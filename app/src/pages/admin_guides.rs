use leptos::prelude::*;

use crate::components::admin_nav::AdminNav;
use crate::i18n::{t, use_i18n, Locale};
use crate::server::auth::current_session;
use crate::server::guides::{list_admin_guides, set_guide_status_admin};
use instant_domain::guides::{GuideStatus, GuideSummary};

#[component]
pub fn AdminGuidesPage() -> impl IntoView {
    let locale = use_i18n().locale;
    let reload = RwSignal::new(0u32);
    let session = Resource::new(
        || (),
        |_| async move { current_session().await.ok().flatten() },
    );
    let guides = Resource::new(
        move || reload.get(),
        |_| async move { list_admin_guides().await.unwrap_or_default() },
    );

    view! {
        <main class="page admin-layout">
            <Suspense fallback=move || view! { <p>{move || t(locale.get(), "正在检查管理员权限", "Checking admin access")}</p> }>
                {move || Suspend::new(async move {
                    let user = session.await;
                    if !user.as_ref().is_some_and(|u| u.role.is_admin()) {
                        return view! {
                            <section class="form">
                                <h1>{move || t(locale.get(), "需要管理员登录", "Admin sign-in required")}</h1>
                                <a class="button button-primary" href="/inspace/login">
                                    {move || t(locale.get(), "去登录", "Go to sign in")}
                                </a>
                            </section>
                        }.into_any();
                    }
                    view! {
                        <AdminNav />
                        <section>
                            <h1>{move || t(locale.get(), "攻略管理", "Guide Management")}</h1>
                            <p>{move || t(locale.get(), "管理全站攻略：发布、取消发布、归档、编辑。", "Manage every guide: publish, unpublish, archive, edit.")}</p>
                            <Suspense fallback=move || view! { <div class="space-list-skeleton"><span></span><span></span></div> }>
                                {move || Suspend::new(async move {
                                    let items = guides.await;
                                    view! { <AdminGuideList items=items reload=reload /> }
                                })}
                            </Suspense>
                        </section>
                    }.into_any()
                })}
            </Suspense>
        </main>
    }
}

#[component]
fn AdminGuideList(items: Vec<GuideSummary>, reload: RwSignal<u32>) -> impl IntoView {
    let locale = use_i18n().locale;
    let query = RwSignal::new(String::new());
    let status_filter = RwSignal::new(String::new());

    if items.is_empty() {
        return view! {
            <section class="empty-state">
                <strong>{move || t(locale.get(), "没有攻略", "No guides")}</strong>
            </section>
        }
        .into_any();
    }

    let all = items.clone();
    let filtered = move || {
        let q = query.get().trim().to_lowercase();
        let sf = status_filter.get();
        all.iter()
            .filter(|g| {
                if !sf.is_empty() && status_key(g.status) != sf {
                    return false;
                }
                if q.is_empty() {
                    return true;
                }
                g.title_zh.to_lowercase().contains(&q)
                    || g.title_en.as_deref().unwrap_or("").to_lowercase().contains(&q)
                    || guide_location(g).to_lowercase().contains(&q)
            })
            .cloned()
            .collect::<Vec<_>>()
    };

    view! {
        <div class="admin-filter-bar">
            <input
                type="text"
                placeholder=move || t(locale.get(), "搜索标题或位置", "Search title or location")
                prop:value=move || query.get()
                on:input=move |ev| query.set(event_target_value(&ev))
            />
            <select
                prop:value=move || status_filter.get()
                on:change=move |ev| status_filter.set(event_target_value(&ev))
            >
                <option value="">{move || t(locale.get(), "全部状态", "All statuses")}</option>
                <option value="draft">{move || t(locale.get(), "草稿", "Draft")}</option>
                <option value="published">{move || t(locale.get(), "已发布", "Published")}</option>
                <option value="archived">{move || t(locale.get(), "已归档", "Archived")}</option>
            </select>
        </div>
        <section class="admin-guide-grid" aria-label="All guides">
            <For
                each=move || filtered()
                key=|g| format!("{}-{:?}", g.id, g.status)
                children=move |g| view! { <AdminGuideCard guide=g reload=reload /> }
            />
        </section>
    }
    .into_any()
}

#[component]
fn AdminGuideCard(guide: GuideSummary, reload: RwSignal<u32>) -> impl IntoView {
    let locale = use_i18n().locale;
    let message = RwSignal::new(None::<String>);
    let error = RwSignal::new(None::<String>);

    let title = if guide.title_zh.trim().is_empty() {
        guide.title_en.clone().unwrap_or_default()
    } else {
        guide.title_zh.clone()
    };
    let loc = guide_location(&guide);
    let status = guide.status;
    let guide_id = guide.id.to_string();
    let edit_href = format!("/inspace/admin/guides/{}/edit", guide.id);

    let run = move |result: Result<GuideSummary, ServerFnError>, ok: &str| match result {
        Ok(_) => {
            error.set(None);
            message.set(Some(ok.to_string()));
            reload.update(|n| *n += 1);
        }
        Err(err) => {
            message.set(None);
            error.set(Some(err.to_string()));
        }
    };

    let set_status = Action::new({
        let id = guide_id.clone();
        move |next: &GuideStatus| {
            let id = id.clone();
            let next = *next;
            async move { set_guide_status_admin(id, next).await }
        }
    });

    Effect::new(move |_| {
        if let Some(result) = set_status.value().get() {
            run(result, t(locale.get(), "已更新状态", "Status updated"));
        }
    });

    view! {
        <article class="admin-guide-card">
            <header>
                <strong>{title}</strong>
                <span class=move || format!("guide-status-badge status-{}", status_key(status))>
                    {status_label(status, locale.get())}
                </span>
            </header>
            <p class="muted">{if loc.is_empty() { "—".to_string() } else { loc }}</p>
            <div class="admin-guide-actions">
                {move || (status != GuideStatus::Published).then(|| view! {
                    <button class="button button-secondary-light" type="button"
                        on:click=move |_| { set_status.dispatch(GuideStatus::Published); }>
                        {move || t(locale.get(), "发布", "Publish")}
                    </button>
                })}
                {move || (status == GuideStatus::Published).then(|| view! {
                    <button class="button button-secondary-light" type="button"
                        on:click=move |_| { set_status.dispatch(GuideStatus::Draft); }>
                        {move || t(locale.get(), "取消发布", "Unpublish")}
                    </button>
                })}
                {move || (status != GuideStatus::Archived).then(|| view! {
                    <button class="button button-danger-light" type="button"
                        on:click=move |_| { set_status.dispatch(GuideStatus::Archived); }>
                        {move || t(locale.get(), "归档", "Archive")}
                    </button>
                })}
                <a class="button button-secondary-light" href=edit_href>
                    {move || t(locale.get(), "编辑", "Edit")}
                </a>
            </div>
            {move || message.get().map(|m| view! { <p class="form-success">{m}</p> })}
            {move || error.get().map(|e| view! { <p class="form-error">{e}</p> })}
        </article>
    }
}

fn status_key(status: GuideStatus) -> &'static str {
    match status {
        GuideStatus::Draft => "draft",
        GuideStatus::Published => "published",
        GuideStatus::Archived => "archived",
    }
}

fn status_label(status: GuideStatus, locale: Locale) -> &'static str {
    match status {
        GuideStatus::Draft => t(locale, "草稿", "Draft"),
        GuideStatus::Published => t(locale, "已发布", "Published"),
        GuideStatus::Archived => t(locale, "已归档", "Archived"),
    }
}

fn guide_location(guide: &GuideSummary) -> String {
    [
        Some(guide.province.as_str()),
        Some(guide.city.as_str()),
        guide.district.as_deref(),
        guide.spot_name.as_deref(),
    ]
    .into_iter()
    .flatten()
    .filter(|v| !v.trim().is_empty())
    .collect::<Vec<_>>()
    .join(" / ")
}
