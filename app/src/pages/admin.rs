use leptos::prelude::*;

use crate::components::admin_nav::AdminNav;

#[component]
pub fn AdminRoutes() -> impl IntoView {
    view! {
        <main class="page admin-layout">
            <AdminNav />
            <section>
                <h1>"管理后台"</h1>
                <p>"Dashboard, spaces, guides, templates, and resident applications share this shell."</p>
            </section>
        </main>
    }
}
