use leptos::prelude::*;
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
