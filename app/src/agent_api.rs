use axum::{
    extract::{ConnectInfo, Path, Query},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, patch},
    Json, Router,
};
use instant_auth::{generate_password_code, hash_password, verify_password};
use instant_domain::{
    guides::{GuideSection, GuideStatus},
    spaces::SpaceType,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::net::SocketAddr;
use uuid::Uuid;

use crate::server::db_pool;

const DEFAULT_PAGE_SIZE: i64 = 50;
const MAX_PAGE_SIZE: i64 = 100;
const KEY_PREFIX_LEN: usize = 16;

pub fn router() -> Router<leptos::prelude::LeptosOptions> {
    Router::<leptos::prelude::LeptosOptions>::new()
        .route("/api/spaces", get(list_spaces).post(create_space))
        .route("/api/spaces/:id", patch(update_space))
        .route("/api/guides", get(list_guides).post(create_guide))
        .route("/api/guides/:id", patch(update_guide))
}

#[derive(Debug, Serialize)]
struct ApiErrorBody {
    error: ApiErrorDetail,
}

#[derive(Debug, Serialize)]
struct ApiErrorDetail {
    code: &'static str,
    message: String,
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: String,
}

impl ApiError {
    fn new(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
        }
    }
    fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, "bad_request", message)
    }
    fn forbidden(message: impl Into<String>) -> Self {
        Self::new(StatusCode::FORBIDDEN, "forbidden", message)
    }
    fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, "not_found", message)
    }
    fn internal(err: impl std::fmt::Display) -> Self {
        tracing::error!("agent api error: {err}");
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "internal server error",
        )
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ApiErrorBody {
                error: ApiErrorDetail {
                    code: self.code,
                    message: self.message,
                },
            }),
        )
            .into_response()
    }
}

type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug, Clone)]
struct Principal {
    key_id: Uuid,
    user_id: Uuid,
    user_email: String,
}

async fn authenticate(headers: &HeaderMap, required_scope: &str) -> ApiResult<Principal> {
    let raw = headers
        .get("x-inspace-api-key")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .or_else(|| {
            headers
                .get(axum::http::header::AUTHORIZATION)
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.strip_prefix("Bearer "))
                .map(str::trim)
                .filter(|v| !v.is_empty())
        })
        .ok_or_else(|| {
            ApiError::new(
                StatusCode::UNAUTHORIZED,
                "missing_api_key",
                "API key required",
            )
        })?;

    if raw.len() < KEY_PREFIX_LEN {
        return Err(ApiError::new(
            StatusCode::UNAUTHORIZED,
            "invalid_api_key",
            "invalid API key",
        ));
    }
    let prefix = &raw[..KEY_PREFIX_LEN];
    let pool = db_pool().await.map_err(ApiError::internal)?;
    let record = instant_db::agent_api::find_active_key_by_prefix(&pool, prefix)
        .await
        .map_err(ApiError::internal)?
        .ok_or_else(|| {
            ApiError::new(
                StatusCode::UNAUTHORIZED,
                "invalid_api_key",
                "invalid API key",
            )
        })?;

    let valid = verify_password(raw, &record.key_hash).map_err(ApiError::internal)?;
    if !valid {
        return Err(ApiError::new(
            StatusCode::UNAUTHORIZED,
            "invalid_api_key",
            "invalid API key",
        ));
    }
    let allowed = record
        .scopes
        .iter()
        .any(|scope| scope == "*" || scope == required_scope);
    if !allowed {
        return Err(ApiError::forbidden(format!(
            "scope required: {required_scope}"
        )));
    }
    let used = instant_db::agent_api::recent_request_count(&pool, record.id)
        .await
        .map_err(ApiError::internal)?;
    if used >= i64::from(record.rate_limit_per_minute) {
        return Err(ApiError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "rate_limited",
            "rate limit exceeded",
        ));
    }

    Ok(Principal {
        key_id: record.id,
        user_id: record.user_id,
        user_email: record.user_email,
    })
}

async fn audit(
    principal: &Principal,
    method: &str,
    path: &str,
    status: StatusCode,
    target_type: Option<&str>,
    target_id: Option<&str>,
    remote: Option<SocketAddr>,
) {
    let Ok(pool) = db_pool().await else { return };
    let remote = remote.map(|value| value.ip().to_string());
    if let Err(err) = instant_db::agent_api::record_request(
        &pool,
        principal.key_id,
        principal.user_id,
        method,
        path,
        status.as_u16(),
        target_type,
        target_id,
        remote.as_deref(),
    )
    .await
    {
        tracing::error!("agent api audit failed: {err}");
    }
}

#[derive(Debug, Deserialize)]
struct PageQuery {
    q: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
}

fn page_bounds(query: &PageQuery) -> (i64, i64) {
    (
        query
            .limit
            .unwrap_or(DEFAULT_PAGE_SIZE)
            .clamp(1, MAX_PAGE_SIZE),
        query.offset.unwrap_or(0).max(0),
    )
}

#[derive(Debug, Serialize)]
struct ListResponse<T> {
    items: Vec<T>,
    limit: i64,
    offset: i64,
}

#[derive(Debug, Serialize)]
struct AgentSpace {
    id: Uuid,
    name_zh: String,
    name_en: Option<String>,
    country: Option<String>,
    province: Option<String>,
    city: Option<String>,
    district: Option<String>,
    spot_name: Option<String>,
    address_line: Option<String>,
    lat: f64,
    lng: f64,
    space_type: String,
    custom_type: Option<String>,
    description_zh: Option<String>,
    description_en: Option<String>,
    tag_zh: Option<String>,
    tag_en: Option<String>,
    is_public: bool,
    status: String,
}

async fn list_spaces(
    headers: HeaderMap,
    Query(query): Query<PageQuery>,
    connect: Option<ConnectInfo<SocketAddr>>,
) -> ApiResult<Json<ListResponse<AgentSpace>>> {
    let principal = authenticate(&headers, "spaces:read").await?;
    let (limit, offset) = page_bounds(&query);
    let q = query
        .q
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty());
    let pool = db_pool().await.map_err(ApiError::internal)?;
    let rows = sqlx::query(
        r#"
        SELECT id, name_zh, name_en, country, province, city, district, spot_name,
               address_line, lat, lng, space_type::text AS space_type, custom_type,
               description_zh, description_en, tag_zh, tag_en, is_public,
               status::text AS status
        FROM spaces
        WHERE (host_user_id = $1 OR creator_id = $1)
          AND status <> 'archived'
          AND ($2::text IS NULL OR concat_ws(' ', name_zh, name_en, country, province, city, district, spot_name, address_line) ILIKE '%' || $2 || '%')
        ORDER BY updated_at DESC, id
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(principal.user_id)
    .bind(q)
    .bind(limit)
    .bind(offset)
    .fetch_all(&pool)
    .await
    .map_err(ApiError::internal)?;

    let items = rows
        .into_iter()
        .map(|row| -> Result<AgentSpace, sqlx::Error> {
            Ok(AgentSpace {
                id: row.try_get("id")?,
                name_zh: row.try_get("name_zh")?,
                name_en: row.try_get("name_en")?,
                country: row.try_get("country")?,
                province: row.try_get("province")?,
                city: row.try_get("city")?,
                district: row.try_get("district")?,
                spot_name: row.try_get("spot_name")?,
                address_line: row.try_get("address_line")?,
                lat: row.try_get("lat")?,
                lng: row.try_get("lng")?,
                space_type: row.try_get("space_type")?,
                custom_type: row.try_get("custom_type")?,
                description_zh: row.try_get("description_zh")?,
                description_en: row.try_get("description_en")?,
                tag_zh: row.try_get("tag_zh")?,
                tag_en: row.try_get("tag_en")?,
                is_public: row.try_get("is_public")?,
                status: row.try_get("status")?,
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(ApiError::internal)?;
    audit(
        &principal,
        "GET",
        "/api/spaces",
        StatusCode::OK,
        None,
        None,
        connect.map(|v| v.0),
    )
    .await;
    Ok(Json(ListResponse {
        items,
        limit,
        offset,
    }))
}

fn default_duration() -> i32 {
    24 * 30
}
fn default_public() -> bool {
    true
}

#[derive(Debug, Deserialize)]
struct CreateSpaceBody {
    name_zh: String,
    name_en: Option<String>,
    country: Option<String>,
    province: String,
    city: String,
    district: Option<String>,
    spot_name: Option<String>,
    address_line: Option<String>,
    lat: f64,
    lng: f64,
    space_type: SpaceType,
    custom_type: Option<String>,
    description_zh: Option<String>,
    description_en: Option<String>,
    tag_zh: Option<String>,
    tag_en: Option<String>,
    #[serde(default = "default_public")]
    is_public: bool,
    #[serde(default = "default_duration")]
    duration_hours: i32,
}

#[derive(Debug, Serialize)]
struct CreatedSpaceResponse {
    id: Uuid,
    name_zh: String,
    password: String,
    hotspot_name: String,
}

async fn create_space(
    headers: HeaderMap,
    connect: Option<ConnectInfo<SocketAddr>>,
    Json(body): Json<CreateSpaceBody>,
) -> ApiResult<(StatusCode, Json<CreatedSpaceResponse>)> {
    let principal = authenticate(&headers, "spaces:write").await?;
    if body.name_zh.trim().is_empty()
        || body.province.trim().is_empty()
        || body.city.trim().is_empty()
    {
        return Err(ApiError::bad_request(
            "name_zh, province and city are required",
        ));
    }
    if !body.lat.is_finite()
        || !body.lng.is_finite()
        || !(-90.0..=90.0).contains(&body.lat)
        || !(-180.0..=180.0).contains(&body.lng)
    {
        return Err(ApiError::bad_request("invalid coordinates"));
    }
    if !(1..=24 * 365).contains(&body.duration_hours) {
        return Err(ApiError::bad_request(
            "duration_hours must be between 1 and 8760",
        ));
    }
    let password = generate_password_code();
    let password_hash = hash_password(&password).map_err(ApiError::internal)?;
    let hotspot_name =
        instant_domain::spaces::hotspot_name(&password).map_err(ApiError::internal)?;
    let pool = db_pool().await.map_err(ApiError::internal)?;
    let created = instant_db::spaces::create_host_space(
        &pool,
        instant_db::spaces::CreateSpaceInput {
            name_zh: body.name_zh.trim().to_string(),
            name_en: clean(body.name_en),
            country: clean(body.country),
            province: body.province.trim().to_string(),
            city: body.city.trim().to_string(),
            district: clean(body.district),
            spot_name: clean(body.spot_name),
            address_line: clean(body.address_line),
            lat: body.lat,
            lng: body.lng,
            space_type: body.space_type,
            custom_type: clean(body.custom_type),
            description_zh: clean(body.description_zh),
            description_en: clean(body.description_en),
            tag_zh: clean(body.tag_zh),
            tag_en: clean(body.tag_en),
            is_public: body.is_public,
            duration_hours: body.duration_hours,
            password_hash,
            host_user_id: principal.user_id,
        },
    )
    .await
    .map_err(ApiError::internal)?;
    let id = created.id.to_string();
    audit(
        &principal,
        "POST",
        "/api/spaces",
        StatusCode::CREATED,
        Some("space"),
        Some(&id),
        connect.map(|v| v.0),
    )
    .await;
    Ok((
        StatusCode::CREATED,
        Json(CreatedSpaceResponse {
            id: created.id,
            name_zh: created.name_zh,
            password,
            hotspot_name,
        }),
    ))
}

#[derive(Debug, Default, Deserialize)]
struct UpdateSpaceBody {
    name_zh: Option<String>,
    name_en: Option<String>,
    country: Option<String>,
    province: Option<String>,
    city: Option<String>,
    district: Option<String>,
    spot_name: Option<String>,
    address_line: Option<String>,
    lat: Option<f64>,
    lng: Option<f64>,
    custom_type: Option<String>,
    description_zh: Option<String>,
    description_en: Option<String>,
    tag_zh: Option<String>,
    tag_en: Option<String>,
    host_bio_zh: Option<String>,
    host_bio_en: Option<String>,
    is_public: Option<bool>,
}

async fn update_space(
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    connect: Option<ConnectInfo<SocketAddr>>,
    Json(body): Json<UpdateSpaceBody>,
) -> ApiResult<Json<instant_domain::spaces::SpaceSummary>> {
    let principal = authenticate(&headers, "spaces:write").await?;
    let pool = db_pool().await.map_err(ApiError::internal)?;
    if !instant_db::agent_api::user_manages_space(&pool, principal.user_id, id)
        .await
        .map_err(ApiError::internal)?
    {
        return Err(ApiError::not_found("space not found"));
    }
    let existing = instant_db::spaces::get_space_detail(&pool, id)
        .await
        .map_err(ApiError::internal)?
        .ok_or_else(|| ApiError::not_found("space not found"))?;
    let lat = body.lat.unwrap_or(existing.summary.lat);
    let lng = body.lng.unwrap_or(existing.summary.lng);
    if !lat.is_finite()
        || !lng.is_finite()
        || !(-90.0..=90.0).contains(&lat)
        || !(-180.0..=180.0).contains(&lng)
    {
        return Err(ApiError::bad_request("invalid coordinates"));
    }
    let updated = instant_db::spaces::update_host_space(
        &pool,
        id,
        instant_db::spaces::UpdateSpaceInput {
            name_zh: body.name_zh.unwrap_or(existing.summary.name_zh),
            name_en: body.name_en.or(existing.summary.name_en),
            country: body.country.or(existing.summary.country),
            province: body
                .province
                .or(existing.summary.province)
                .unwrap_or_default(),
            city: body.city.or(existing.summary.city).unwrap_or_default(),
            district: body.district.or(existing.summary.district),
            spot_name: body.spot_name.or(existing.summary.spot_name),
            address_line: body.address_line.or(existing.summary.address_line),
            lat,
            lng,
            is_public: body.is_public.unwrap_or(existing.summary.is_public),
            custom_type: body.custom_type.or(existing.custom_type),
            description_zh: body.description_zh.or(existing.description_zh),
            description_en: body.description_en.or(existing.description_en),
            tag_zh: body.tag_zh.or(existing.tag_zh),
            tag_en: body.tag_en.or(existing.tag_en),
            host_bio_zh: body.host_bio_zh.or(existing.host_bio_zh),
            host_bio_en: body.host_bio_en.or(existing.host_bio_en),
        },
    )
    .await
    .map_err(ApiError::internal)?;
    let sid = id.to_string();
    audit(
        &principal,
        "PATCH",
        "/api/spaces/:id",
        StatusCode::OK,
        Some("space"),
        Some(&sid),
        connect.map(|v| v.0),
    )
    .await;
    Ok(Json(updated))
}

#[derive(Debug, Serialize)]
struct AgentGuide {
    id: Uuid,
    title_zh: String,
    title_en: Option<String>,
    country: Option<String>,
    province: String,
    city: String,
    district: Option<String>,
    spot_name: Option<String>,
    status: String,
    featured: bool,
    space_id: Option<Uuid>,
}

async fn list_guides(
    headers: HeaderMap,
    Query(query): Query<PageQuery>,
    connect: Option<ConnectInfo<SocketAddr>>,
) -> ApiResult<Json<ListResponse<AgentGuide>>> {
    let principal = authenticate(&headers, "guides:read").await?;
    let (limit, offset) = page_bounds(&query);
    let tokens = query.q.as_ref().map(|value| {
        value
            .split_whitespace()
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>()
    });
    let tokens = tokens.filter(|items| !items.is_empty());
    let pool = db_pool().await.map_err(ApiError::internal)?;
    let rows = sqlx::query(
        r#"
        SELECT g.id, g.title_zh, g.title_en, g.country, g.province, g.city, g.district, g.spot_name,
               g.status::text AS status, g.featured, g.space_id
        FROM guides g
        LEFT JOIN spaces s ON s.id = g.space_id
        WHERE (g.author_id = $1 OR s.host_user_id = $1 OR s.creator_id = $1)
          AND (
            $2::text[] IS NULL
            OR NOT EXISTS (
              SELECT 1 FROM unnest($2::text[]) tok
              WHERE concat_ws(
                ' ', g.title_zh, g.title_en, g.country, g.province, g.city,
                g.district, g.spot_name, g.summary_zh, g.summary_en,
                g.content_zh, g.content_en
              ) NOT ILIKE '%' || tok || '%'
            )
          )
        ORDER BY g.updated_at DESC, g.id
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(principal.user_id)
    .bind(tokens)
    .bind(limit)
    .bind(offset)
    .fetch_all(&pool)
    .await
    .map_err(ApiError::internal)?;
    let items = rows
        .into_iter()
        .map(|row| -> Result<AgentGuide, sqlx::Error> {
            Ok(AgentGuide {
                id: row.try_get("id")?,
                title_zh: row.try_get("title_zh")?,
                title_en: row.try_get("title_en")?,
                country: row.try_get("country")?,
                province: row.try_get("province")?,
                city: row.try_get("city")?,
                district: row.try_get("district")?,
                spot_name: row.try_get("spot_name")?,
                status: row.try_get("status")?,
                featured: row.try_get("featured")?,
                space_id: row.try_get("space_id")?,
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(ApiError::internal)?;
    audit(
        &principal,
        "GET",
        "/api/guides",
        StatusCode::OK,
        None,
        None,
        connect.map(|v| v.0),
    )
    .await;
    Ok(Json(ListResponse {
        items,
        limit,
        offset,
    }))
}

fn default_guide_status() -> GuideStatus {
    GuideStatus::Draft
}
#[derive(Debug, Deserialize)]
struct CreateGuideBody {
    title_zh: String,
    title_en: Option<String>,
    summary_zh: Option<String>,
    summary_en: Option<String>,
    content_zh: Option<String>,
    content_en: Option<String>,
    guide_type: Option<String>,
    category: Option<String>,
    province: String,
    city: String,
    district: Option<String>,
    spot_name: Option<String>,
    space_id: Option<Uuid>,
    cover_image_url: Option<String>,
    #[serde(default)]
    images: Vec<String>,
    #[serde(default)]
    sections: Vec<GuideSection>,
    #[serde(default = "default_guide_status")]
    status: GuideStatus,
    #[serde(default)]
    featured: bool,
}

async fn create_guide(
    headers: HeaderMap,
    connect: Option<ConnectInfo<SocketAddr>>,
    Json(body): Json<CreateGuideBody>,
) -> ApiResult<(StatusCode, Json<instant_domain::guides::GuideSummary>)> {
    let principal = authenticate(&headers, "guides:write").await?;
    if body.title_zh.trim().is_empty()
        || body.province.trim().is_empty()
        || body.city.trim().is_empty()
    {
        return Err(ApiError::bad_request(
            "title_zh, province and city are required",
        ));
    }
    let pool = db_pool().await.map_err(ApiError::internal)?;
    if let Some(space_id) = body.space_id {
        if !instant_db::agent_api::user_manages_space(&pool, principal.user_id, space_id)
            .await
            .map_err(ApiError::internal)?
        {
            return Err(ApiError::forbidden(
                "the API key user does not manage this space",
            ));
        }
    }
    let created = instant_db::guides::create_guide_draft(
        &pool,
        instant_db::guides::CreateGuideDraftInput {
            title_zh: body.title_zh.trim().to_string(),
            title_en: clean(body.title_en),
            summary_zh: clean(body.summary_zh),
            summary_en: clean(body.summary_en),
            content_zh: clean(body.content_zh),
            content_en: clean(body.content_en),
            guide_type: clean(body.guide_type).unwrap_or_else(|| "record".to_string()),
            category: clean(body.category),
            province: body.province.trim().to_string(),
            city: body.city.trim().to_string(),
            district: clean(body.district),
            spot_name: clean(body.spot_name),
            author_id: principal.user_id,
            author_name: Some(principal.user_email.clone()),
            space_id: body.space_id,
            cover_image_url: clean(body.cover_image_url),
            images: clean_vec(body.images),
            sections: body.sections,
            status: body.status,
            featured: body.featured,
        },
    )
    .await
    .map_err(ApiError::internal)?;
    let gid = created.id.to_string();
    audit(
        &principal,
        "POST",
        "/api/guides",
        StatusCode::CREATED,
        Some("guide"),
        Some(&gid),
        connect.map(|v| v.0),
    )
    .await;
    Ok((StatusCode::CREATED, Json(created)))
}

#[derive(Debug, Default, Deserialize)]
struct UpdateGuideBody {
    title_zh: Option<String>,
    title_en: Option<String>,
    summary_zh: Option<String>,
    summary_en: Option<String>,
    content_zh: Option<String>,
    content_en: Option<String>,
    guide_type: Option<String>,
    category: Option<String>,
    province: Option<String>,
    city: Option<String>,
    district: Option<String>,
    spot_name: Option<String>,
    space_id: Option<Uuid>,
    cover_image_url: Option<String>,
    images: Option<Vec<String>>,
    sections: Option<Vec<GuideSection>>,
    status: Option<GuideStatus>,
    featured: Option<bool>,
}

async fn update_guide(
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    connect: Option<ConnectInfo<SocketAddr>>,
    Json(body): Json<UpdateGuideBody>,
) -> ApiResult<Json<instant_domain::guides::GuideSummary>> {
    let principal = authenticate(&headers, "guides:write").await?;
    let pool = db_pool().await.map_err(ApiError::internal)?;
    if !instant_db::agent_api::user_manages_guide(&pool, principal.user_id, id)
        .await
        .map_err(ApiError::internal)?
    {
        return Err(ApiError::not_found("guide not found"));
    }
    let existing = instant_db::guides::get_guide(&pool, id)
        .await
        .map_err(ApiError::internal)?
        .ok_or_else(|| ApiError::not_found("guide not found"))?;
    let space_id = body.space_id.or(existing.space_id);
    if let Some(space_id) = space_id {
        if !instant_db::agent_api::user_manages_space(&pool, principal.user_id, space_id)
            .await
            .map_err(ApiError::internal)?
        {
            return Err(ApiError::forbidden(
                "the API key user does not manage this space",
            ));
        }
    }
    let updated = instant_db::guides::update_guide(
        &pool,
        id,
        instant_db::guides::UpdateGuideInput {
            title_zh: body.title_zh.unwrap_or(existing.title_zh),
            title_en: body.title_en.or(existing.title_en),
            summary_zh: body.summary_zh.or(existing.summary_zh),
            summary_en: body.summary_en.or(existing.summary_en),
            content_zh: body.content_zh.or(existing.content_zh),
            content_en: body.content_en.or(existing.content_en),
            guide_type: body.guide_type.unwrap_or(existing.guide_type),
            category: body.category.or(existing.category),
            province: body.province.unwrap_or(existing.province),
            city: body.city.unwrap_or(existing.city),
            district: body.district.or(existing.district),
            spot_name: body.spot_name.or(existing.spot_name),
            space_id,
            cover_image_url: body.cover_image_url.or(existing.cover_image_url),
            images: body.images.map(clean_vec).unwrap_or(existing.images),
            sections: body.sections.unwrap_or(existing.sections),
            status: body.status.unwrap_or(existing.status),
            featured: body.featured.unwrap_or(existing.featured),
        },
    )
    .await
    .map_err(ApiError::internal)?;
    let gid = id.to_string();
    audit(
        &principal,
        "PATCH",
        "/api/guides/:id",
        StatusCode::OK,
        Some("guide"),
        Some(&gid),
        connect.map(|v| v.0),
    )
    .await;
    Ok(Json(updated))
}

fn clean(value: Option<String>) -> Option<String> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}
fn clean_vec(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .collect()
}
