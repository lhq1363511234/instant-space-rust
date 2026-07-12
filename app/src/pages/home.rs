use leptos::prelude::*;

use crate::components::map_home::MapHome;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <main class="app-shell">
            <MapHome />
        </main>
    }
}
