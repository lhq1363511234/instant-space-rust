use instant_domain::admin::AdminStats;
use leptos::prelude::*;

#[server(GetAdminStats, "/api")]
pub async fn get_admin_stats() -> Result<AdminStats, ServerFnError> {
    let database_url =
        std::env::var("DATABASE_URL").map_err(|err| ServerFnError::new(err.to_string()))?;
    let pool = instant_db::connect(&database_url)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;

    instant_db::admin::stats(&pool)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))
}
