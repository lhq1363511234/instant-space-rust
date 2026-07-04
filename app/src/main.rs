use axum::Router;
use instant_space_web::app::{shell, App};
use leptos::prelude::LeptosOptions;
use leptos_axum::{generate_route_list, LeptosRoutes};
use std::net::SocketAddr;
use tower_http::services::ServeDir;

const SITE_ADDR: &str = "127.0.0.1:3001";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let addr: SocketAddr = SITE_ADDR.parse()?;
    tracing::info!("listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, build_router().into_make_service()).await?;
    Ok(())
}

fn build_router() -> Router {
    let options = leptos_options();
    let routes = generate_route_list(App);
    let shell_options = options.clone();

    Router::new()
        .nest_service("/pkg", ServeDir::new("target/site/pkg"))
        .nest_service("/style", ServeDir::new("app/style"))
        .leptos_routes(&options, routes, move || shell(shell_options.clone()))
        .with_state(options)
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
        assert!(html.contains("/pkg/instant_space_app.js"));
    }
}
