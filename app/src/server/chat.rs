use instant_domain::chat::{AccessGrant, ChatAccessState, ChatMessage};
use leptos::prelude::*;

#[cfg(feature = "ssr")]
use axum::http::{header::SET_COOKIE, HeaderMap, HeaderValue};
#[cfg(feature = "ssr")]
use instant_auth::{generate_token, verify_password};
#[cfg(feature = "ssr")]
use leptos_axum::ResponseOptions;
#[cfg(feature = "ssr")]
use time::{Duration, OffsetDateTime};
#[cfg(feature = "ssr")]
use uuid::Uuid;

#[server(VerifySpacePassword, "/inspace/api")]
pub async fn verify_space_password(
    space_id: String,
    password: String,
) -> Result<AccessGrant, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let id = Uuid::parse_str(&space_id).map_err(|err| ServerFnError::new(err.to_string()))?;
        let pool = crate::server::db_pool().await?;
        let Some((hash, version)) = instant_db::spaces::space_password_hash(&pool, id)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?
        else {
            return Err(ServerFnError::new("space not found"));
        };
        let ok =
            verify_password(&password, &hash).map_err(|err| ServerFnError::new(err.to_string()))?;
        if !ok {
            return Err(ServerFnError::new("wrong password"));
        }

        let token = generate_token();
        let expires_at = OffsetDateTime::now_utc() + Duration::hours(12);
        instant_db::chat::create_access_session(&pool, id, &token, version, expires_at)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?;
        set_access_cookie(id, &token)?;

        Ok(AccessGrant {
            password_version: version,
            expires_at: expires_at.to_string(),
        })
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("server only"))
    }
}

#[server(CheckSpaceAccess, "/inspace/api")]
pub async fn check_space_access(space_id: String) -> Result<ChatAccessState, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let id = Uuid::parse_str(&space_id).map_err(|_| ServerFnError::new("invalid space id"))?;
        let pool = crate::server::db_pool().await?;
        let Some(meta) = instant_db::spaces::space_access_meta(&pool, id)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?
        else {
            return Err(ServerFnError::new("space not found"));
        };

        let access_version = if meta.is_public {
            Some(meta.password_version)
        } else if let Some(token) = access_cookie(id).await? {
            instant_db::chat::has_valid_access_session(&pool, id, &token)
                .await
                .map_err(|err| ServerFnError::new(err.to_string()))?
        } else {
            None
        };

        Ok(ChatAccessState {
            space_id: meta.id,
            space_name: meta.name_en.unwrap_or(meta.name_zh),
            is_public: meta.is_public,
            allowed: meta.is_public || access_version.is_some(),
            password_version: access_version,
        })
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("server only"))
    }
}

#[server(ListChatMessages, "/inspace/api")]
pub async fn list_chat_messages(space_id: String) -> Result<Vec<ChatMessage>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let id = Uuid::parse_str(&space_id).map_err(|err| ServerFnError::new(err.to_string()))?;
        ensure_chat_access(id, false).await?;
        let pool = crate::server::db_pool().await?;

        instant_db::chat::list_messages(&pool, id)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("server only"))
    }
}

#[server(SendChatMessage, "/inspace/api")]
pub async fn send_chat_message(
    space_id: String,
    body: String,
) -> Result<ChatMessage, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let clean_body = body.trim().to_string();
        if clean_body.is_empty() {
            return Err(ServerFnError::new("message is required"));
        }
        if clean_body.chars().count() > 800 {
            return Err(ServerFnError::new("message is too long"));
        }

        let id = Uuid::parse_str(&space_id).map_err(|err| ServerFnError::new(err.to_string()))?;
        let access = ensure_chat_access(id, true).await?;
        let pool = crate::server::db_pool().await?;
        let sender = crate::server::auth::current_session()
            .await?
            .map(|user| user.name.unwrap_or(user.email))
            .unwrap_or_else(|| "Guest".to_string());

        let message = instant_db::chat::insert_message(
            &pool,
            id,
            sender,
            clean_body,
            access.password_version,
        )
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;
        crate::realtime::publish_message(id, message.clone()).await;
        Ok(message)
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("server only"))
    }
}

#[cfg(feature = "ssr")]
struct ValidChatAccess {
    password_version: i32,
}

#[cfg(feature = "ssr")]
async fn ensure_chat_access(
    space_id: Uuid,
    require_current_password: bool,
) -> Result<ValidChatAccess, ServerFnError> {
    let pool = crate::server::db_pool().await?;
    let Some(meta) = instant_db::spaces::space_access_meta(&pool, space_id)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?
    else {
        return Err(ServerFnError::new("space not found"));
    };

    if meta.is_public {
        return Ok(ValidChatAccess {
            password_version: meta.password_version,
        });
    }

    let Some(token) = access_cookie(space_id).await? else {
        return Err(ServerFnError::new("private space access required"));
    };
    let Some(password_version) =
        instant_db::chat::has_valid_access_session(&pool, space_id, &token)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?
    else {
        return Err(ServerFnError::new("private space access expired"));
    };

    if require_current_password && password_version != meta.password_version {
        return Err(ServerFnError::new(
            "space password changed; please re-enter password",
        ));
    }

    Ok(ValidChatAccess { password_version })
}

#[cfg(feature = "ssr")]
fn access_cookie_name(space_id: Uuid) -> String {
    format!("instant_access_{}", space_id.simple())
}

#[cfg(feature = "ssr")]
fn set_access_cookie(space_id: Uuid, token: &str) -> Result<(), ServerFnError> {
    if let Some(response) = use_context::<ResponseOptions>() {
        let cookie = format!(
            "{}={token}; Path=/; HttpOnly; Secure; SameSite=Lax; Max-Age=43200",
            access_cookie_name(space_id)
        );
        let value =
            HeaderValue::from_str(&cookie).map_err(|err| ServerFnError::new(err.to_string()))?;
        response.insert_header(SET_COOKIE, value);
    }
    Ok(())
}

#[cfg(feature = "ssr")]
async fn access_cookie(space_id: Uuid) -> Result<Option<String>, ServerFnError> {
    let headers: HeaderMap = leptos_axum::extract().await?;
    let Some(cookie) = headers.get(axum::http::header::COOKIE) else {
        return Ok(None);
    };
    let cookie = cookie
        .to_str()
        .map_err(|err| ServerFnError::new(err.to_string()))?;
    let name = access_cookie_name(space_id);

    Ok(cookie.split(';').find_map(|part| {
        let (cookie_name, value) = part.trim().split_once('=')?;
        (cookie_name == name).then(|| value.to_string())
    }))
}
