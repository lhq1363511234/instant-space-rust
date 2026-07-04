use leptos::prelude::*;

use crate::components::guide_browser::GuideBrowser;

#[component]
pub fn GuidesPage() -> impl IntoView {
    view! {
        <main class="page">
            <GuideBrowser />
        </main>
    }
}
