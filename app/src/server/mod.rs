pub mod admin;
pub mod auth;
pub mod chat;
pub mod geo;
pub mod guides;
pub mod site;
pub mod spaces;
pub mod traces;

#[cfg(feature = "ssr")]
use leptos::prelude::ServerFnError;
#[cfg(feature = "ssr")]
use std::sync::OnceLock;

#[cfg(feature = "ssr")]
static DB_POOL: OnceLock<sqlx::PgPool> = OnceLock::new();

#[cfg(feature = "ssr")]
pub async fn db_pool() -> Result<sqlx::PgPool, ServerFnError> {
    if let Some(pool) = DB_POOL.get() {
        return Ok(pool.clone());
    }

    let database_url =
        std::env::var("DATABASE_URL").map_err(|err| ServerFnError::new(err.to_string()))?;
    let pool = instant_db::connect(&database_url)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;

    let _ = DB_POOL.set(pool.clone());
    Ok(DB_POOL.get().cloned().unwrap_or(pool))
}
