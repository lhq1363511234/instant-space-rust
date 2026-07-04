use instant_domain::{guides::GuideSummary, locations::LocationNode};
use leptos::prelude::*;

#[server(ListGuides, "/api")]
pub async fn list_guides(province: Option<String>) -> Result<Vec<GuideSummary>, ServerFnError> {
    let database_url =
        std::env::var("DATABASE_URL").map_err(|err| ServerFnError::new(err.to_string()))?;
    let pool = instant_db::connect(&database_url)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;

    instant_db::guides::list_published_guides(&pool, province)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))
}

#[server(ListProvinces, "/api")]
pub async fn list_provinces() -> Result<Vec<LocationNode>, ServerFnError> {
    let database_url =
        std::env::var("DATABASE_URL").map_err(|err| ServerFnError::new(err.to_string()))?;
    let pool = instant_db::connect(&database_url)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;

    instant_db::locations::provinces(&pool)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))
}
