use instant_domain::guides::{GuideStatus, GuideSummary};
use leptos::prelude::*;

use crate::components::admin_nav::AdminNav;
use crate::i18n::{t, use_i18n, Locale};
use crate::server::auth::current_session;
use crate::server::guides::{list_admin_guides, set_guide_status_admin};

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
        <main id="main-content" class="page admin-layout admin-guides-page">
            <Suspense fallback=move || view! { <p>{move || t(locale.get(), "正在检查管理员权限", "Checking admin access")}</p> }>
                {move || Suspend::new(async move {
                    let user = session.await;
                    if !user.as_ref().is_some_and(|u| u.role.is_admin()) {
                        return view! {
                            <section class="form"><h1>{move || t(locale.get(), "需要管理员登录", "Admin sign-in required")}</h1><a class="button button-primary" href="/inspace/login">{move || t(locale.get(), "去登录", "Go to sign in")}</a></section>
                        }.into_any();
                    }
                    view! {
                        <AdminNav />
                        <section class="admin-guide-workspace">
                            <header class="admin-guide-head">
                                <div><p class="eyebrow">"GUIDE OPERATIONS"</p><h1>{move || t(locale.get(), "空间攻略管理", "Space guide management")}</h1><p>{move || t(locale.get(), "管理攻略从草稿到发布的全过程，并清楚看到它属于哪个真实空间。攻略是讨论的共同上下文，不是孤立文章。", "Manage guides from draft to publication and see which real Space each guide belongs to. A guide is shared context for discussion, not an isolated article.")}</p></div>
                                <a class="button button-primary" href="/inspace/admin/guides/new">{move || t(locale.get(), "新建空间攻略", "New space guide")}</a>
                            </header>
                            <Suspense fallback=move || view! { <div class="space-list-skeleton"><span></span><span></span></div> }>
                                {move || Suspend::new(async move { view! { <AdminGuideList items=guides.await reload=reload /> } })}
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

    let total = items.len();
    let published = items
        .iter()
        .filter(|g| g.status == GuideStatus::Published)
        .count();
    let drafts = items
        .iter()
        .filter(|g| g.status == GuideStatus::Draft)
        .count();
    let archived = items
        .iter()
        .filter(|g| g.status == GuideStatus::Archived)
        .count();

    let all = items.clone();
    let filtered = Memo::new(move |_| {
        let q = query.get().trim().to_lowercase();
        let sf = status_filter.get();
        all.iter()
            .filter(|guide| {
                if !sf.is_empty() && status_key(guide.status) != sf {
                    return false;
                }
                q.is_empty()
                    || guide.title_zh.to_lowercase().contains(&q)
                    || guide
                        .title_en
                        .as_deref()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains(&q)
                    || guide_location(guide).to_lowercase().contains(&q)
            })
            .cloned()
            .collect::<Vec<_>>()
    });

    view! {
        <div class="admin-guide-stats" aria-label=move || t(locale.get(), "攻略统计", "Guide statistics")>
            <article><span>{move || t(locale.get(), "全部攻略", "All guides")}</span><strong>{total}</strong></article>
            <article><span>{move || t(locale.get(), "已发布", "Published")}</span><strong>{published}</strong></article>
            <article><span>{move || t(locale.get(), "待完善草稿", "Drafts")}</span><strong>{drafts}</strong></article>
            <article><span>{move || t(locale.get(), "已归档", "Archived")}</span><strong>{archived}</strong></article>
        </div>
        <div class="admin-guide-toolbar">
            <input type="search" placeholder=move || t(locale.get(), "搜索攻略标题、地点或空间", "Search title, place, or Space") prop:value=move || query.get() on:input=move |ev| query.set(event_target_value(&ev)) />
            <select aria-label=move || t(locale.get(), "攻略状态", "Guide status") prop:value=move || status_filter.get() on:change=move |ev| status_filter.set(event_target_value(&ev))>
                <option value="">{move || t(locale.get(), "全部状态", "All statuses")}</option><option value="draft">{move || t(locale.get(), "草稿", "Draft")}</option><option value="published">{move || t(locale.get(), "已发布", "Published")}</option><option value="archived">{move || t(locale.get(), "已归档", "Archived")}</option>
            </select>
        </div>
        <div class="admin-guide-table" role="table" aria-label=move || t(locale.get(), "空间攻略列表", "Space guide list")>
            <div class="admin-guide-row admin-guide-row-head" role="row"><span>{move || t(locale.get(), "攻略", "Guide")}</span><span>{move || t(locale.get(), "所属空间", "Space")}</span><span>{move || t(locale.get(), "状态", "Status")}</span><span>{move || t(locale.get(), "操作", "Actions")}</span></div>
            <Show when=move || !filtered.get().is_empty() fallback=move || view! { <div class="empty-state"><strong>{move || t(locale.get(), "没有符合条件的攻略", "No matching guides")}</strong></div> }>
                <For each=move || filtered.get() key=|guide| format!("{}-{:?}", guide.id, guide.status) children=move |guide| view! { <AdminGuideRow guide=guide reload=reload /> } />
            </Show>
        </div>
        <aside class="admin-guide-workflow"><strong>{move || t(locale.get(), "讨论沉淀工作流", "Discussion-to-guide workflow")}</strong><span>{move || t(locale.get(), "空间讨论 → 标记有效回答 → 主理人整理 → 攻略版本审核 → 发布", "Space discussion → mark useful answer → host edits → review guide version → publish")}</span></aside>
    }
}

#[component]
fn AdminGuideRow(guide: GuideSummary, reload: RwSignal<u32>) -> impl IntoView {
    let locale = use_i18n().locale;
    let message = RwSignal::new(None::<String>);
    let error = RwSignal::new(None::<String>);
    let title = if guide.title_zh.trim().is_empty() {
        guide.title_en.clone().unwrap_or_default()
    } else {
        guide.title_zh.clone()
    };
    let location = guide_location(&guide);
    let status = guide.status;
    let guide_id = guide.id.to_string();
    let edit_href = format!("/inspace/admin/guides/{}/edit", guide.id);
    let preview_href = format!("/inspace/guides/{}", guide.id);
    let space_href = guide.space_id.map(|id| format!("/inspace/spaces/{id}"));

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
            match result {
                Ok(_) => {
                    error.set(None);
                    message.set(Some(
                        t(locale.get(), "状态已更新", "Status updated").to_string(),
                    ));
                    reload.update(|value| *value += 1);
                }
                Err(err) => {
                    message.set(None);
                    error.set(Some(err.to_string()));
                }
            }
        }
    });

    view! {
        <article class="admin-guide-row" role="row">
            <div class="admin-guide-title-cell"><strong>{title}</strong><span>{if location.is_empty() { "—".to_string() } else { location.clone() }}</span></div>
            <div class="admin-guide-space-cell">{match space_href { Some(href) => view! { <a href=href>{move || t(locale.get(), "查看所属空间", "View linked Space")}</a><span>{move || t(locale.get(), "攻略与在线讨论共享此地点上下文", "Guide and live room share this place context")}</span> }.into_any(), None => view! { <strong>{move || t(locale.get(), "未绑定空间", "Not linked")}</strong><span>{move || t(locale.get(), "可编辑时绑定到真实地点", "Link it to a real place while editing")}</span> }.into_any() }}</div>
            <span class=format!("guide-status-badge status-{}", status_key(status))>{status_label(status, locale.get())}</span>
            <div class="admin-guide-actions">
                {move || (status == GuideStatus::Published).then(|| view! { <a class="button button-secondary-light" href=preview_href.clone()>{move || t(locale.get(), "预览", "Preview")}</a> })}
                {move || (status != GuideStatus::Published).then(|| view! { <button class="button button-secondary-light" type="button" disabled=move || set_status.pending().get() on:click=move |_| { set_status.dispatch(GuideStatus::Published); }>{move || t(locale.get(), "发布", "Publish")}</button> })}
                {move || (status == GuideStatus::Published).then(|| view! { <button class="button button-secondary-light" type="button" disabled=move || set_status.pending().get() on:click=move |_| { set_status.dispatch(GuideStatus::Draft); }>{move || t(locale.get(), "取消发布", "Unpublish")}</button> })}
                {move || (status == GuideStatus::Archived).then(|| view! { <button class="button button-secondary-light" type="button" disabled=move || set_status.pending().get() on:click=move |_| { set_status.dispatch(GuideStatus::Draft); }>{move || t(locale.get(), "恢复", "Restore")}</button> })}
                {move || (status != GuideStatus::Archived).then(|| view! { <button class="button button-danger-light" type="button" disabled=move || set_status.pending().get() on:click=move |_| { set_status.dispatch(GuideStatus::Archived); }>{move || t(locale.get(), "归档", "Archive")}</button> })}
                <a class="button button-primary" href=edit_href>{move || t(locale.get(), "编辑", "Edit")}</a>
            </div>
            {move || message.get().map(|value| view! { <p class="form-success" role="status">{value}</p> })}
            {move || error.get().map(|value| view! { <p class="form-error" role="alert">{value}</p> })}
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
    .filter(|value| !value.trim().is_empty())
    .collect::<Vec<_>>()
    .join(" / ")
}
