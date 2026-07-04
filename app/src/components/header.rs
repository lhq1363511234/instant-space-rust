use leptos::prelude::*;

#[component]
pub fn Header() -> impl IntoView {
    view! {
        <header class="topbar">
            <a href="/" class="brand">"Instant Space"</a>
            <nav>
                <a href="/guides">"导览"</a>
                <a href="/my-spaces">"创建空间"</a>
                <a href="/admin">"Admin"</a>
            </nav>
        </header>
    }
}
