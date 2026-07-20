#![recursion_limit = "512"]

use axum::{extract::Query, http::StatusCode, routing::get, Json, Router};
use instant_domain::locations::GeoMatch;
use instant_space_web::app::{shell, App};
use leptos::prelude::LeptosOptions;
use leptos_axum::{generate_route_list, LeptosRoutes};
use serde::Deserialize;
use std::net::SocketAddr;
use std::time::Duration;
use tower_http::services::ServeDir;

const SITE_ADDR: &str = "127.0.0.1:3001";
/// How often the background task promotes expired spaces.
const EXPIRY_SWEEP_INTERVAL: Duration = Duration::from_secs(60);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    // Single source of truth for the schema: run embedded migrations before
    // the service starts serving, so code and database never drift apart.
    let pool = instant_space_web::server::db_pool()
        .await
        .map_err(|err| anyhow::anyhow!(err.to_string()))?;
    instant_db::run_migrations(&pool).await?;
    tracing::info!("migrations applied");

    // Promote any already-stale active spaces once at boot, then keep sweeping.
    match instant_db::spaces::expire_stale_spaces(&pool).await {
        Ok(count) if count > 0 => tracing::info!("expired {count} stale spaces at startup"),
        Ok(_) => {}
        Err(err) => tracing::error!("startup space expiry failed: {err}"),
    }
    spawn_expiry_task(pool.clone());

    let addr: SocketAddr = SITE_ADDR.parse()?;
    tracing::info!("listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, build_router().into_make_service()).await?;
    Ok(())
}

/// Background loop that transitions `active` spaces past their `expires_at`
/// into `expired`, keeping the map and explore list consistent without
/// per-request checks.
fn spawn_expiry_task(pool: sqlx::PgPool) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(EXPIRY_SWEEP_INTERVAL);
        // Skip the immediate first tick; startup already ran one sweep.
        ticker.tick().await;
        loop {
            ticker.tick().await;
            match instant_db::spaces::expire_stale_spaces(&pool).await {
                Ok(count) if count > 0 => tracing::info!("expired {count} stale spaces"),
                Ok(_) => {}
                Err(err) => tracing::error!("space expiry sweep failed: {err}"),
            }
        }
    });
}

fn build_router() -> Router {
    let options = leptos_options();
    let routes = generate_route_list(App);
    let shell_options = options.clone();

    Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route("/geo/reverse", get(reverse_geo))
        .nest_service("/pkg", ServeDir::new("target/site/pkg"))
        .nest_service("/style", ServeDir::new("app/style"))
        .nest_service("/vendor", ServeDir::new("app/vendor"))
        .leptos_routes(&options, routes, move || shell(shell_options.clone()))
        .with_state(options)
}

/// Liveness: the process is up and serving. Does not touch the database.
async fn health() -> &'static str {
    "ok"
}

/// Readiness: the process can serve real traffic, i.e. the database is
/// reachable. Returns 503 when the pool cannot answer a trivial query.
async fn ready() -> (StatusCode, &'static str) {
    let Ok(pool) = instant_space_web::server::db_pool().await else {
        return (StatusCode::SERVICE_UNAVAILABLE, "db pool unavailable");
    };
    match sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&pool)
        .await
    {
        Ok(_) => (StatusCode::OK, "ready"),
        Err(_) => (StatusCode::SERVICE_UNAVAILABLE, "db query failed"),
    }
}

#[derive(Debug, Deserialize)]
struct ReverseGeoQuery {
    lat: f64,
    lng: f64,
}

async fn reverse_geo(
    Query(query): Query<ReverseGeoQuery>,
) -> Result<Json<Option<GeoMatch>>, (StatusCode, String)> {
    if !query.lat.is_finite() || !query.lng.is_finite() {
        return Err((StatusCode::BAD_REQUEST, "invalid coordinates".to_string()));
    }

    let pool = instant_space_web::server::db_pool()
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    let location = instant_db::geo::nearest_place(&pool, query.lat, query.lng)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    Ok(Json(location))
}

fn leptos_options() -> LeptosOptions {
    LeptosOptions::builder()
        .output_name("instant_space_app")
        .site_addr(SITE_ADDR.parse::<SocketAddr>().expect("valid site address"))
        .hash_files(false)
        .build()
}

#[cfg(test)]
mod tests {
    use axum::{
        body::{to_bytes, Body},
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn root_route_is_served_by_leptos_ssr_shell() {
        let response = super::build_router()
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
        let html = String::from_utf8(body.to_vec()).unwrap();

        assert!(html.to_ascii_lowercase().contains("<!doctype html>"));
        assert!(html.contains("Instant Space Rust"));
        assert!(html.contains("data-instant-ssr=\"leptos\""));
        assert!(html.contains("/vendor/maplibre-gl/maplibre-gl.js"));
        assert!(html.contains("/vendor/maplibre-gl/maplibre-gl.css"));
        assert!(!html.contains("unpkg.com/maplibre-gl"));
        assert!(html.contains("/pkg/instant_space_app.js"));
    }

    #[tokio::test]
    async fn health_endpoint_is_liveness_only() {
        let response = super::build_router()
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
