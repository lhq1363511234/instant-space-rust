use leptos::prelude::*;

#[component]
pub fn AdminNav() -> impl IntoView {
    view! {
        <nav class="admin-nav">
            <a href="/admin">"Dashboard"</a>
            <a href="/admin/spaces">"Spaces"</a>
            <a href="/admin/guides">"Guides"</a>
            <a href="/admin/templates">"Templates"</a>
            <a href="/admin/resident-applications">"Resident"</a>
        </nav>
    }
}
