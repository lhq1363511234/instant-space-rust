//! Phase 4 media asset upload.
//!
//! Guide covers and section images were URL-only; hosts and admins had to host
//! pictures somewhere else. This module adds a small first-party upload
//! endpoint (authenticated, size/mime restricted) that stores files on disk
//! under `uploads/` and serves them back at `/uploads/<file>`.
//!
//! Browser requests go to `/inspace/api/media/upload` (nginx proxies the full
//! URI to this service); direct/local calls may use `/api/media/upload`. Both
//! routes are registered so curl against 127.0.0.1:3001 works too.

use axum::{
    extract::{DefaultBodyLimit, Multipart},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use serde::Serialize;
use uuid::Uuid;

pub fn router() -> Router<leptos::prelude::LeptosOptions> {
    Router::<leptos::prelude::LeptosOptions>::new()
        .route("/api/media/upload", post(upload_media))
        .route("/inspace/api/media/upload", post(upload_media))
        .layer(DefaultBodyLimit::max(MAX_UPLOAD_BYTES as usize))
}

/// Cap a single upload at 10 MB; big enough for photos, small enough to keep
/// disk and memory sane on this host.
const MAX_UPLOAD_BYTES: u64 = 10 * 1024 * 1024;

const ALLOWED_MIME: &[(&str, &str)] = &[
    ("image/jpeg", "jpg"),
    ("image/png", "png"),
    ("image/webp", "webp"),
    ("image/gif", "gif"),
    ("image/avif", "avif"),
];

#[derive(Debug, Serialize)]
struct UploadResponse {
    url: String,
}

#[derive(Debug, Serialize)]
struct UploadError {
    error: String,
}

async fn upload_media(
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, Response> {
    // Authenticate with the same HttpOnly session cookie the rest of the app
    // uses. Any signed-in user may upload; they are only attached to guides
    // they own or manage.
    let Some(user) = authenticated_user(&headers).await? else {
        return Err(api_error(StatusCode::UNAUTHORIZED, "login required"));
    };

    let mut uploaded: Option<(String, Vec<u8>)> = None; // (ext, bytes)
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| api_error(StatusCode::BAD_REQUEST, "malformed multipart body"))?
    {
        let mime = field
            .content_type()
            .map(ToOwned::to_owned)
            .unwrap_or_default();
        let Some((_, ext)) = ALLOWED_MIME.iter().find(|(allowed, _)| *allowed == mime) else {
            return Err(api_error(
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "only jpeg/png/webp/gif/avif images are allowed",
            ));
        };
        let data = field
            .bytes()
            .await
            .map_err(|_| api_error(StatusCode::BAD_REQUEST, "failed to read upload"))?;
        if data.is_empty() || data.len() > MAX_UPLOAD_BYTES as usize {
            return Err(api_error(
                StatusCode::PAYLOAD_TOO_LARGE,
                "file must be between 1 byte and 10 MB",
            ));
        }
        uploaded = Some((ext.to_string(), data.to_vec()));
        // Keep only the first file field; ignore extra fields.
        break;
    }

    let Some((ext, raw)) = uploaded else {
        return Err(api_error(StatusCode::BAD_REQUEST, "no file field present"));
    };

    let filename = format!("{}.{}", Uuid::new_v4().simple(), ext);
    let dir = std::path::Path::new("uploads");
    std::fs::create_dir_all(dir).map_err(|_| {
        api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "cannot create uploads dir",
        )
    })?;
    let path = dir.join(&filename);
    std::fs::write(&path, &raw)
        .map_err(|_| api_error(StatusCode::INTERNAL_SERVER_ERROR, "cannot persist upload"))?;

    tracing::info!("media uploaded by {}: {}", user.email, filename);
    Ok(Json(UploadResponse {
        url: format!("/inspace/uploads/{filename}"),
    }))
}

async fn authenticated_user(
    headers: &HeaderMap,
) -> Result<Option<instant_domain::auth::CurrentUser>, Response> {
    let Some(cookie) = headers
        .get(axum::http::header::COOKIE)
        .and_then(|value| value.to_str().ok())
    else {
        return Ok(None);
    };
    let token = cookie.split(';').find_map(|part| {
        let (name, value) = part.trim().split_once('=')?;
        (name == "instant_session").then(|| value.to_string())
    });
    let Some(token) = token else { return Ok(None) };
    let pool = crate::server::db_pool()
        .await
        .map_err(|_| api_error(StatusCode::SERVICE_UNAVAILABLE, "database unavailable"))?;
    instant_db::users::current_user_by_token(&pool, &token)
        .await
        .map_err(|_| api_error(StatusCode::INTERNAL_SERVER_ERROR, "session lookup failed"))
}

fn api_error(status: StatusCode, message: &str) -> Response {
    (
        status,
        Json(UploadError {
            error: message.to_string(),
        }),
    )
        .into_response()
}
