use axum::{response::Html, routing::get, Router};
use std::net::SocketAddr;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let app = Router::new()
        .route("/", get(home))
        .route("/guides", get(guides))
        .route("/admin", get(admin))
        .nest_service("/style", ServeDir::new("app/style"));

    let addr: SocketAddr = "127.0.0.1:3001".parse()?;
    tracing::info!("listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

async fn home() -> Html<&'static str> {
    Html(
        r#"<!doctype html>
<html lang="zh-CN">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <link href="https://unpkg.com/maplibre-gl@4.7.1/dist/maplibre-gl.css" rel="stylesheet" />
    <link href="/style/main.css" rel="stylesheet" />
    <script src="https://unpkg.com/maplibre-gl@4.7.1/dist/maplibre-gl.js"></script>
    <title>Instant Space Rust</title>
  </head>
  <body>
    <main class="app-shell">
      <header class="topbar">
        <a href="/" class="brand">Instant Space</a>
        <nav>
          <a href="/guides">&#23548;&#35272;</a>
          <a href="/admin">Admin</a>
        </nav>
      </header>
      <section class="map-layout">
        <div id="map" class="map-canvas" aria-label="Instant Space map"></div>
        <aside class="space-panel">
          <label>&#25628;&#32034;&#31354;&#38388; <input type="search" aria-label="Search spaces" /></label>
          <ul class="space-list">
            <li><button type="button">&#22806;&#28393;</button></li>
            <li>
              <button type="button">&#31169;&#23494;&#33590;&#23460;</button>
              <form class="private-entry" aria-label="Private space verification">
                <label>Password <input type="password" aria-label="Private space password" /></label>
                <button type="button">Enter chat</button>
              </form>
            </li>
          </ul>
        </aside>
      </section>
    </main>
  </body>
</html>"#,
    )
}

async fn guides() -> Html<&'static str> {
    Html(
        r#"<!doctype html>
<html lang="zh-CN">
  <head><meta charset="utf-8" /><link href="/style/main.css" rel="stylesheet" /><title>&#23548;&#35272;</title></head>
  <body><main class="page"><h1>&#23548;&#35272;</h1><select aria-label="Province"><option>&#19978;&#28023;&#24066;</option></select></main></body>
</html>"#,
    )
}

async fn admin() -> Html<&'static str> {
    Html(
        r#"<!doctype html>
<html lang="zh-CN">
  <head><meta charset="utf-8" /><link href="/style/main.css" rel="stylesheet" /><title>&#31649;&#29702;&#21518;&#21488;</title></head>
  <body><main class="page admin-layout"><h1>&#31649;&#29702;&#21518;&#21488;</h1><nav class="admin-nav"><a href="/admin/spaces">Spaces</a><a href="/admin/guides">Guides</a><a href="/admin/templates">Templates</a><a href="/admin/resident-applications">Resident</a></nav></main></body>
</html>"#,
    )
}
