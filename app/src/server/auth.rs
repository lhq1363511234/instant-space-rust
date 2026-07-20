use instant_domain::auth::{CurrentUser, UserRole};
use leptos::prelude::*;

#[cfg(feature = "ssr")]
use axum::http::{header::SET_COOKIE, HeaderMap, HeaderValue};
#[cfg(feature = "ssr")]
use instant_auth::{generate_token, hash_password, verify_password};
#[cfg(feature = "ssr")]
use leptos_axum::ResponseOptions;
#[cfg(feature = "ssr")]
use time::{Duration, OffsetDateTime};

#[server(RegisterUser, "/inspace/api")]
pub async fn register_user(
    email: String,
    password: String,
    name: Option<String>,
) -> Result<CurrentUser, ServerFnError> {
    let pool = crate::server::db_pool().await?;
    let password_hash =
        hash_password(&password).map_err(|err| ServerFnError::new(err.to_string()))?;
    let user = instant_db::users::create_user(&pool, &email, name.as_deref(), &password_hash)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;
    create_session_cookie(&pool, user.id).await?;

    Ok(CurrentUser {
        id: user.id,
        email: user.email,
        name: user.name,
        role: user.role,
    })
}

#[server(LoginUser, "/inspace/api")]
pub async fn login_user(email: String, password: String) -> Result<CurrentUser, ServerFnError> {
    let pool = crate::server::db_pool().await?;
    let Some((user_id, password_hash, role)) =
        instant_db::users::find_user_password_hash(&pool, &email)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?
    else {
        return Err(ServerFnError::new("invalid credentials"));
    };
    let ok = verify_password(&password, &password_hash)
        .map_err(|err| ServerFnError::new(err.to_string()))?;
    if !ok {
        return Err(ServerFnError::new("invalid credentials"));
    }
    create_session_cookie(&pool, user_id).await?;
    let user = instant_db::users::current_user_by_id(&pool, user_id)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;

    Ok(user.unwrap_or(CurrentUser {
        id: user_id,
        email,
        name: None,
        role,
    }))
}

#[server(CurrentSession, "/inspace/api")]
pub async fn current_session() -> Result<Option<CurrentUser>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let Some(token) = session_token().await? else {
            return Ok(None);
        };
        let pool = crate::server::db_pool().await?;
        instant_db::users::current_user_by_token(&pool, &token)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))
    }

    #[cfg(not(feature = "ssr"))]
    {
        Ok(None)
    }
}

#[server(LogoutUser, "/inspace/api")]
pub async fn logout_user() -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        if let Some(token) = session_token().await? {
            let pool = crate::server::db_pool().await?;
            instant_db::users::delete_session_by_token(&pool, &token)
                .await
                .map_err(|err| ServerFnError::new(err.to_string()))?;
        }
        clear_session_cookie()?;
    }

    Ok(())
}

pub fn role_label(role: &UserRole) -> &'static str {
    match role {
        UserRole::User => "user",
        UserRole::Admin => "admin",
        UserRole::SuperAdmin => "super_admin",
    }
}

#[cfg(feature = "ssr")]
async fn create_session_cookie(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
) -> Result<(), ServerFnError> {
    let token = generate_token();
    let expires_at = OffsetDateTime::now_utc() + Duration::days(7);
    instant_db::users::create_session(pool, user_id, &token, expires_at)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;
    set_session_cookie(&token)?;
    Ok(())
}

#[cfg(feature = "ssr")]
fn set_session_cookie(token: &str) -> Result<(), ServerFnError> {
    if let Some(response) = use_context::<ResponseOptions>() {
        let cookie =
            format!("instant_session={token}; Path=/; HttpOnly; Secure; SameSite=Lax; Max-Age=604800");
        let value =
            HeaderValue::from_str(&cookie).map_err(|err| ServerFnError::new(err.to_string()))?;
        response.insert_header(SET_COOKIE, value);
    }
    Ok(())
}

#[cfg(feature = "ssr")]
fn clear_session_cookie() -> Result<(), ServerFnError> {
    if let Some(response) = use_context::<ResponseOptions>() {
        let cookie = "instant_session=; Path=/; HttpOnly; Secure; SameSite=Lax; Max-Age=0";
        let value =
            HeaderValue::from_str(cookie).map_err(|err| ServerFnError::new(err.to_string()))?;
        response.insert_header(SET_COOKIE, value);
    }
    Ok(())
}

#[cfg(feature = "ssr")]
pub async fn require_admin_user() -> Result<CurrentUser, ServerFnError> {
    let Some(user) = current_session().await? else {
        return Err(ServerFnError::new("admin login required"));
    };

    if user.role.is_admin() {
        Ok(user)
    } else {
        Err(ServerFnError::new("admin permission required"))
    }
}

#[cfg(feature = "ssr")]
async fn session_token() -> Result<Option<String>, ServerFnError> {
    let headers: HeaderMap = leptos_axum::extract().await?;
    let Some(cookie) = headers.get(axum::http::header::COOKIE) else {
        return Ok(None);
    };
    let cookie = cookie
        .to_str()
        .map_err(|err| ServerFnError::new(err.to_string()))?;

    Ok(cookie.split(';').find_map(|part| {
        let (name, value) = part.trim().split_once('=')?;
        (name == "instant_session").then(|| value.to_string())
    }))
}
