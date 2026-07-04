use instant_domain::auth::CurrentUser;
use leptos::prelude::*;

#[cfg(feature = "ssr")]
use axum::http::{header::SET_COOKIE, HeaderValue};
#[cfg(feature = "ssr")]
use instant_auth::{generate_token, hash_password, verify_password};
#[cfg(feature = "ssr")]
use leptos_axum::ResponseOptions;
#[cfg(feature = "ssr")]
use time::{Duration, OffsetDateTime};

#[server(RegisterUser, "/api")]
pub async fn register_user(
    email: String,
    password: String,
    name: Option<String>,
) -> Result<CurrentUser, ServerFnError> {
    let database_url =
        std::env::var("DATABASE_URL").map_err(|err| ServerFnError::new(err.to_string()))?;
    let pool = instant_db::connect(&database_url)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;
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
    })
}

#[server(LoginUser, "/api")]
pub async fn login_user(email: String, password: String) -> Result<CurrentUser, ServerFnError> {
    let database_url =
        std::env::var("DATABASE_URL").map_err(|err| ServerFnError::new(err.to_string()))?;
    let pool = instant_db::connect(&database_url)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;
    let Some((user_id, password_hash)) = instant_db::users::find_user_password_hash(&pool, &email)
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
    }))
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
            format!("instant_session={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age=604800");
        let value =
            HeaderValue::from_str(&cookie).map_err(|err| ServerFnError::new(err.to_string()))?;
        response.insert_header(SET_COOKIE, value);
    }
    Ok(())
}
