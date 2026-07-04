use leptos::prelude::*;
#[cfg(feature = "ssr")]
use leptos_meta::MetaTags;
use leptos_meta::{provide_meta_context, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

use crate::pages::{
    admin::AdminRoutes, auth::LoginPage, guides::GuidesPage, home::HomePage, host::HostRoutes,
};

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Title text="Instant Space Rust" />
        <Stylesheet id="main-css" href="/style/main.css" />
        <Router>
            <Routes fallback=|| view! { <main class="page"><h1>"Not found"</h1></main> }>
                <Route path=path!("/") view=HomePage />
                <Route path=path!("/login") view=LoginPage />
                <Route path=path!("/my-spaces") view=HostRoutes />
                <Route path=path!("/guides") view=GuidesPage />
                <Route path=path!("/admin") view=AdminRoutes />
            </Routes>
        </Router>
    }
}

#[cfg(feature = "ssr")]
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="zh-CN" data-instant-ssr="leptos">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <link href="https://unpkg.com/maplibre-gl@5.0.0/dist/maplibre-gl.css" rel="stylesheet" />
                <script src="https://unpkg.com/maplibre-gl@5.0.0/dist/maplibre-gl.js"></script>
                <AutoReload options=options.clone() />
                <HydrationScripts options=options.clone() />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}
