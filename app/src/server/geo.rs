use instant_domain::locations::{GeoMatch, GeoOption};
use leptos::prelude::*;

#[server(ListGeoCountries, "/inspace/api")]
pub async fn list_geo_countries() -> Result<Vec<GeoOption>, ServerFnError> {
    let pool = crate::server::db_pool().await?;
    instant_db::geo::countries(&pool)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))
}

#[server(ListGeoRegions, "/inspace/api")]
pub async fn list_geo_regions(country: String) -> Result<Vec<GeoOption>, ServerFnError> {
    if country.trim().is_empty() {
        return Ok(Vec::new());
    }
    let pool = crate::server::db_pool().await?;
    instant_db::geo::regions(&pool, country)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))
}

#[server(ListGeoCities, "/inspace/api")]
pub async fn list_geo_cities(
    country: String,
    region: Option<String>,
) -> Result<Vec<GeoOption>, ServerFnError> {
    if country.trim().is_empty() {
        return Ok(Vec::new());
    }
    let pool = crate::server::db_pool().await?;
    instant_db::geo::cities(&pool, country, region)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))
}

#[server(ListGeoDistricts, "/inspace/api")]
pub async fn list_geo_districts(
    country: String,
    region: Option<String>,
    city: Option<String>,
) -> Result<Vec<GeoOption>, ServerFnError> {
    if country.trim().is_empty() {
        return Ok(Vec::new());
    }
    let pool = crate::server::db_pool().await?;
    instant_db::geo::districts(&pool, country, region, city)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))
}

#[server(ReverseGeoPoint, "/inspace/api")]
pub async fn reverse_geo_point(lat: f64, lng: f64) -> Result<Option<GeoMatch>, ServerFnError> {
    if !lat.is_finite() || !lng.is_finite() {
        return Err(ServerFnError::new("invalid coordinates"));
    }
    let pool = crate::server::db_pool().await?;
    instant_db::geo::nearest_place(&pool, lat, lng)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))
}


#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PlaceCenter {
    pub lng: f64,
    pub lat: f64,
    pub zoom: f64,
}

#[server(ResolvePlaceCenter, "/inspace/api")]
pub async fn resolve_place_center(
    country: Option<String>,
    region: Option<String>,
    city: Option<String>,
) -> Result<Option<PlaceCenter>, ServerFnError> {
    let pool = crate::server::db_pool().await?;
    let found = instant_db::geo::place_center(&pool, country, region, city)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;
    Ok(found.map(|(lng, lat, zoom)| PlaceCenter { lng, lat, zoom }))
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GeoCapital {
    pub country: String,
    pub capital: String,
    pub lng: f64,
    pub lat: f64,
    pub zoom: f64,
}

#[server(ListGeoCapitals, "/inspace/api")]
pub async fn list_geo_capitals() -> Result<Vec<GeoCapital>, ServerFnError> {
    let pool = crate::server::db_pool().await?;
    let rows = instant_db::geo::list_capitals(&pool)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;
    Ok(rows
        .into_iter()
        .map(|(country, capital, lng, lat, zoom)| GeoCapital {
            country,
            capital,
            lng,
            lat,
            zoom,
        })
        .collect())
}
