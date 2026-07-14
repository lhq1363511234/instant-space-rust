use leptos::prelude::*;
#[cfg(feature = "ssr")]
use leptos_meta::MetaTags;
use leptos_meta::{provide_meta_context, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

use crate::components::{
    header::Header,
    space_form::{provide_create_space_modal, CreateSpaceModalHost},
};
use crate::pages::{
    admin::AdminRoutes,
    auth::LoginPage,
    guides::{GuideDetailPage, GuideEditorPage, GuidesPage},
    home::HomePage,
    host::HostRoutes,
    space::SpacePage,
};

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    crate::i18n::provide_i18n();
    crate::app_state::provide_app_refresh_state();
    provide_create_space_modal();

    view! {
        <Title text="Instant Space Rust" />
        <Stylesheet id="main-css" href="/style/main.css?v=20260712-home-hero-fix-v46" />
        <Router>
            <Header />
            <Routes fallback=|| view! { <main class="page"><h1>"Not found"</h1></main> }>
                <Route path=path!("/") view=HomePage />
                <Route path=path!("/inspace") view=HomePage />
                <Route path=path!("/login") view=LoginPage />
                <Route path=path!("/inspace/login") view=LoginPage />
                <Route path=path!("/my-spaces") view=HostRoutes />
                <Route path=path!("/inspace/my-spaces") view=HostRoutes />
                <Route path=path!("/spaces/:space_id") view=SpacePage />
                <Route path=path!("/inspace/spaces/:space_id") view=SpacePage />
                <Route path=path!("/guides") view=GuidesPage />
                <Route path=path!("/inspace/guides") view=GuidesPage />
                <Route path=path!("/guides/new") view=GuideEditorPage />
                <Route path=path!("/inspace/guides/new") view=GuideEditorPage />
                <Route path=path!("/admin/guides/new") view=GuideEditorPage />
                <Route path=path!("/inspace/admin/guides/new") view=GuideEditorPage />
                <Route path=path!("/guides/:guide_id/edit") view=GuideEditorPage />
                <Route path=path!("/inspace/guides/:guide_id/edit") view=GuideEditorPage />
                <Route path=path!("/admin/guides/:guide_id/edit") view=GuideEditorPage />
                <Route path=path!("/inspace/admin/guides/:guide_id/edit") view=GuideEditorPage />
                <Route path=path!("/guides/:guide_id") view=GuideDetailPage />
                <Route path=path!("/inspace/guides/:guide_id") view=GuideDetailPage />
                <Route path=path!("/admin") view=AdminRoutes />
                <Route path=path!("/inspace/admin") view=AdminRoutes />
            </Routes>
            <CreateSpaceModalHost />
        </Router>
    }
}

#[cfg(feature = "ssr")]
pub fn shell(options: LeptosOptions) -> impl IntoView {
    // Native map bootstrap (no WASM required): mounts #map + opens map for ?country/?map.
    let map_boot = include_str!("map_boot.js");
    let capitals_boot = include_str!("geo_capitals_boot.js");

    view! {
        <!DOCTYPE html>
        <html lang="zh-CN" data-instant-ssr="leptos">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <link href="/vendor/maplibre-gl/maplibre-gl.css" rel="stylesheet" />
                <script src="/vendor/maplibre-gl/maplibre-gl.js"></script>
                <script>{capitals_boot}</script>
                <script>{map_boot}</script>
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
