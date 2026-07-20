use leptos::prelude::*;

use crate::components::admin_nav::AdminNav;
use crate::i18n::{t, use_i18n};
use crate::server::auth::current_session;
use crate::server::spaces::{
    archive_my_space_template, close_my_space, delete_my_space, list_admin_spaces,
    reactivate_my_space, SpaceMarker,
};

#[component]
pub fn AdminSpacesPage() -> impl IntoView {
    let locale = use_i18n().locale;
    let reload = RwSignal::new(0u32);
    let session = Resource::new(
        || (),
        |_| async move { current_session().await.ok().flatten() },
    );
    let spaces = Resource::new(
        move || reload.get(),
        |_| async move { list_admin_spaces().await.unwrap_or_default() },
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
                            <h1>{move || t(locale.get(), "空间管理", "Space Management")}</h1>
                            <p>{move || t(locale.get(), "管理全站空间，包括已归档/删除的空间。", "Manage every space, including archived/deleted ones.")}</p>
                            <Suspense fallback=move || view! { <div class="space-list-skeleton"><span></span><span></span></div> }>
                                {move || Suspend::new(async move {
                                    let items = spaces.await;
                                    view! { <AdminSpaceList items=items reload=reload /> }
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
fn AdminSpaceList(items: Vec<SpaceMarker>, reload: RwSignal<u32>) -> impl IntoView {
    let locale = use_i18n().locale;
    let query = RwSignal::new(String::new());
    let status_filter = RwSignal::new(String::new());

    if items.is_empty() {
        return view! {
            <section class="empty-state">
                <strong>{move || t(locale.get(), "没有空间", "No spaces")}</strong>
            </section>
        }
        .into_any();
    }

    let all = items.clone();
    let filtered = move || {
        let q = query.get().trim().to_lowercase();
        let sf = status_filter.get();
        all.iter()
            .filter(|s| {
                if !sf.is_empty() && s.status != sf {
                    return false;
                }
                if q.is_empty() {
                    return true;
                }
                s.name_zh.to_lowercase().contains(&q)
                    || s.name_en.as_deref().unwrap_or("").to_lowercase().contains(&q)
                    || location_label(s).to_lowercase().contains(&q)
            })
            .cloned()
            .collect::<Vec<_>>()
    };

    view! {
        <div class="admin-filter-bar">
            <input
                type="text"
                placeholder=move || t(locale.get(), "搜索名称或位置", "Search name or location")
                prop:value=move || query.get()
                on:input=move |ev| query.set(event_target_value(&ev))
            />
            <select
                prop:value=move || status_filter.get()
                on:change=move |ev| status_filter.set(event_target_value(&ev))
            >
                <option value="">{move || t(locale.get(), "全部状态", "All statuses")}</option>
                <option value="active">{move || t(locale.get(), "活跃", "Active")}</option>
                <option value="expired">{move || t(locale.get(), "过期", "Expired")}</option>
                <option value="closed">{move || t(locale.get(), "已关闭", "Closed")}</option>
                <option value="archived">{move || t(locale.get(), "已删除", "Archived")}</option>
                <option value="template">{move || t(locale.get(), "模板", "Template")}</option>
            </select>
        </div>
        <section class="my-space-grid" aria-label="All spaces">
            <For
                each=move || filtered()
                key=|space| format!("{}-{}", space.id, space.status)
                children=move |space| view! { <AdminSpaceCard space=space reload=reload /> }
            />
        </section>
    }
    .into_any()
}

#[component]
fn AdminSpaceCard(space: SpaceMarker, reload: RwSignal<u32>) -> impl IntoView {
    let locale = use_i18n().locale;
    let message = RwSignal::new(None::<String>);
    let error = RwSignal::new(None::<String>);
    let confirm_delete = RwSignal::new(false);

    let name = space.name_zh.clone();
    let loc = location_label(&space);
    let status = space.status.clone();
    let status_badge = status.clone();
    let is_public = space.is_public;
    let space_id = space.id.clone();

    let run = move |result: Result<SpaceMarker, ServerFnError>, ok: &str| {
        match result {
            Ok(_) => {
                error.set(None);
                message.set(Some(ok.to_string()));
                reload.update(|n| *n += 1);
            }
            Err(err) => {
                message.set(None);
                error.set(Some(err.to_string()));
            }
        }
    };

    let close = Action::new({
        let id = space_id.clone();
        move |_: &()| {
            let id = id.clone();
            async move { close_my_space(id).await }
        }
    });
    let reactivate = Action::new({
        let id = space_id.clone();
        move |_: &()| {
            let id = id.clone();
            async move { reactivate_my_space(id).await }
        }
    });
    let archive = Action::new({
        let id = space_id.clone();
        move |_: &()| {
            let id = id.clone();
            async move { archive_my_space_template(id).await }
        }
    });
    let delete = Action::new({
        let id = space_id.clone();
        move |_: &()| {
            let id = id.clone();
            async move { delete_my_space(id).await }
        }
    });

    Effect::new(move |_| {
        if let Some(result) = close.value().get() {
            run(result, "已关闭");
        }
    });
    Effect::new(move |_| {
        if let Some(result) = reactivate.value().get() {
            run(result, "已重新激活");
        }
    });
    Effect::new(move |_| {
        if let Some(result) = archive.value().get() {
            run(result, "已归档为模板");
        }
    });
    Effect::new(move |_| {
        if let Some(result) = delete.value().get() {
            run(result, "已删除");
        }
    });

    view! {
        <article class="my-space-card">
            <header>
                <strong>{name}</strong>
                <span class=move || format!("space-status-badge status-{}", status_badge)>
                    {status_label(&status_badge, locale.get())}
                </span>
            </header>
            <p class="muted">{if loc.is_empty() { "—".to_string() } else { loc }}</p>
            <p class="muted">
                {move || if is_public {
                    t(locale.get(), "公开", "Public")
                } else {
                    t(locale.get(), "私密", "Private")
                }}
            </p>
            <div class="admin-space-actions">
                <button class="button button-secondary-light" type="button" on:click=move |_| { close.dispatch(()); }>
                    {move || t(locale.get(), "关闭", "Close")}
                </button>
                <button class="button button-secondary-light" type="button" on:click=move |_| { reactivate.dispatch(()); }>
                    {move || t(locale.get(), "重新激活", "Reactivate")}
                </button>
                <button class="button button-secondary-light" type="button" on:click=move |_| { archive.dispatch(()); }>
                    {move || t(locale.get(), "归档模板", "Archive template")}
                </button>
                {move || if confirm_delete.get() {
                    view! {
                        <button class="button button-danger" type="button" on:click=move |_| { delete.dispatch(()); confirm_delete.set(false); }>
                            {move || t(locale.get(), "确认删除", "Confirm delete")}
                        </button>
                    }.into_any()
                } else {
                    view! {
                        <button class="button button-danger-light" type="button" on:click=move |_| { confirm_delete.set(true); }>
                            {move || t(locale.get(), "删除", "Delete")}
                        </button>
                    }.into_any()
                }}
            </div>
            {move || message.get().map(|m| view! { <p class="form-success">{m}</p> })}
            {move || error.get().map(|e| view! { <p class="form-error">{e}</p> })}
        </article>
    }
}

fn status_label(status: &str, locale: crate::i18n::Locale) -> &'static str {
    match status {
        "active" => t(locale, "活跃", "Active"),
        "expired" => t(locale, "过期", "Expired"),
        "closed" => t(locale, "已关闭", "Closed"),
        "archived" => t(locale, "已删除", "Archived"),
        "template" => t(locale, "模板", "Template"),
        _ => t(locale, "未知", "Unknown"),
    }
}

fn location_label(space: &SpaceMarker) -> String {
    [
        space.country.as_deref(),
        space.province.as_deref(),
        space.city.as_deref(),
        space.district.as_deref(),
        space.spot_name.as_deref(),
        space.address_line.as_deref(),
    ]
    .into_iter()
    .flatten()
    .filter(|value| !value.trim().is_empty())
    .collect::<Vec<_>>()
    .join(" / ")
}
