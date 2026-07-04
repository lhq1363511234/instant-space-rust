use leptos::prelude::*;

use crate::components::{header::Header, map_home::MapHome};

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <main class="app-shell">
            <Header />
            <MapHome />
        </main>
    }
}
