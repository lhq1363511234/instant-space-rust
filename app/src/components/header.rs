use leptos::prelude::*;

#[component]
pub fn Header() -> impl IntoView {
    view! {
        <header class="topbar" aria-label="Instant Space navigation">
            <a href="/" class="brand" aria-label="Instant Space home">
                <span class="brand-mark" aria-hidden="true"></span>
                <span>"Instant Space"</span>
            </a>
            <nav class="primary-nav" aria-label="Primary">
                <a href="/" class="nav-link is-active">"Explore"</a>
                <a href="/guides" class="nav-link">"Guides"</a>
                <a href="/my-spaces" class="nav-link nav-link-primary">"Create Space"</a>
                <a href="/admin" class="nav-link nav-link-muted">"Admin"</a>
            </nav>
        </header>
    }
}
