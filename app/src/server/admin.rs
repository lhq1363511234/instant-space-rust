use instant_domain::admin::AdminStats;
use leptos::prelude::*;

#[server(GetAdminStats, "/inspace/api")]
pub async fn get_admin_stats() -> Result<AdminStats, ServerFnError> {
    #[cfg(feature = "ssr")]
    crate::server::auth::require_admin_user().await?;

    let pool = crate::server::db_pool().await?;

    instant_db::admin::stats(&pool)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))
}
