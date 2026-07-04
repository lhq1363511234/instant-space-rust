use leptos::prelude::*;

use crate::components::admin_nav::AdminNav;
use crate::server::admin::get_admin_stats;

#[component]
pub fn AdminRoutes() -> impl IntoView {
    let stats = Resource::new(|| (), |_| async move { get_admin_stats().await.ok() });

    view! {
        <main class="page admin-layout">
            <AdminNav />
            <section>
                <h1>"管理后台"</h1>
                <p>"Dashboard, spaces, guides, templates, and resident applications share this shell."</p>
                <Suspense fallback=move || view! { <p>"加载统计"</p> }>
                    {move || Suspend::new(async move {
                        let stats = stats.await;
                        view! {
                            <div class="stats-grid">
                                <article>
                                    <strong>"Spaces"</strong>
                                    <span>{stats.as_ref().map(|s| s.spaces_count).unwrap_or_default()}</span>
                                </article>
                                <article>
                                    <strong>"Guides"</strong>
                                    <span>{stats.as_ref().map(|s| s.guides_count).unwrap_or_default()}</span>
                                </article>
                                <article>
                                    <strong>"Users"</strong>
                                    <span>{stats.as_ref().map(|s| s.users_count).unwrap_or_default()}</span>
                                </article>
                                <article>
                                    <strong>"Resident"</strong>
                                    <span>{stats.as_ref().map(|s| s.pending_resident_applications).unwrap_or_default()}</span>
                                </article>
                            </div>
                        }
                    })}
                </Suspense>
            </section>
        </main>
    }
}
