use instant_domain::spaces::SpaceType;
use leptos::prelude::*;

use crate::components::admin_nav::AdminNav;
use crate::i18n::{t, use_i18n};
use crate::pages::host::ManageSpaceModal;
use crate::server::auth::current_session;
use crate::server::spaces::{
    archive_my_space_template, close_my_space, delete_my_space, list_admin_space_page,
    reactivate_my_space, SpaceMarker, SpacePageResult,
};

#[component]
pub fn AdminSpacesPage() -> impl IntoView {
    let locale = use_i18n().locale;
    let reload = RwSignal::new(0u32);
    let query = RwSignal::new(String::new());
    let status_filter = RwSignal::new("managed".to_string());
    let type_filter = RwSignal::new(String::new());
    let page = RwSignal::new(1i32);
    let session = Resource::new(
        || (),
        |_| async move { current_session().await.ok().flatten() },
    );
    let spaces = Resource::new(
        move || {
            (
                reload.get(),
                query.get(),
                status_filter.get(),
                type_filter.get(),
                page.get(),
            )
        },
        |(_, query, status, type_filter, page)| async move {
            let space_type = parse_space_type(&type_filter);
            list_admin_space_page(
                optional_string(query),
                optional_string(status),
                space_type,
                page,
                20,
            )
            .await
            .unwrap_or(SpacePageResult {
                items: Vec::new(),
                total: 0,
                page,
                page_size: 20,
                total_pages: 1,
            })
        },
    );

    view! {
        <main id="main-content" class="page admin-layout">
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
                            <div class="page-head admin-page-head">
                                <div>
                                    <p class="eyebrow">"SPACE OPERATIONS"</p>
                                    <h1>{move || t(locale.get(), "空间管理", "Space Management")}</h1>
                                    <p>{move || t(locale.get(), "先找到要处理的空间，再编辑、管理攻略或调整状态。列表按页加载，不会一次塞进一万个空间。", "Find the Space you need, then edit it, manage its guides, or change its status. Results are paginated instead of loading ten thousand rows at once.")}</p>
                                </div>
                            </div>
                            <Suspense fallback=move || view! { <div class="space-list-skeleton"><span></span><span></span></div> }>
                                {move || Suspend::new(async move {
                                    let result = spaces.await;
                                    view! { <AdminSpaceList result=result query=query status_filter=status_filter type_filter=type_filter page=page reload=reload /> }
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
fn AdminSpaceList(
    result: SpacePageResult,
    query: RwSignal<String>,
    status_filter: RwSignal<String>,
    type_filter: RwSignal<String>,
    page: RwSignal<i32>,
    reload: RwSignal<u32>,
) -> impl IntoView {
    let locale = use_i18n().locale;
    let first = if result.total == 0 {
        0
    } else {
        (result.page - 1) * result.page_size + 1
    };
    let last = if result.total == 0 {
        0
    } else {
        first + result.items.len() as i32 - 1
    };
    let current_page = result.page;
    let total_pages = result.total_pages;
    let items = result.items;

    view! {
        <div class="admin-filter-bar admin-space-filter-bar">
            <label>
                <span>{move || t(locale.get(), "搜索空间", "Search Spaces")}</span>
                <input
                    type="search"
                    placeholder=move || t(locale.get(), "名称、城市、地点或地址", "Name, city, place, or address")
                    prop:value=move || query.get()
                    on:input=move |ev| {
                        query.set(event_target_value(&ev));
                        page.set(1);
                    }
                />
            </label>
            <label>
                <span>{move || t(locale.get(), "状态", "Status")}</span>
                <select
                    prop:value=move || status_filter.get()
                    on:change=move |ev| {
                        status_filter.set(event_target_value(&ev));
                        page.set(1);
                    }
                >
                    <option value="managed">{move || t(locale.get(), "未删除空间", "Managed spaces")}</option>
                    <option value="active">{move || t(locale.get(), "活跃", "Active")}</option>
                    <option value="expired">{move || t(locale.get(), "已过期", "Expired")}</option>
                    <option value="closed">{move || t(locale.get(), "已关闭", "Closed")}</option>
                    <option value="template">{move || t(locale.get(), "模板", "Template")}</option>
                    <option value="archived">{move || t(locale.get(), "已删除", "Deleted")}</option>
                    <option value="">{move || t(locale.get(), "全部状态", "All statuses")}</option>
                </select>
            </label>
            <label>
                <span>{move || t(locale.get(), "类型", "Type")}</span>
                <select
                    prop:value=move || type_filter.get()
                    on:change=move |ev| {
                        type_filter.set(event_target_value(&ev));
                        page.set(1);
                    }
                >
                    <option value="">{move || t(locale.get(), "全部类型", "All types")}</option>
                    <option value="scenic">{move || t(locale.get(), "景点", "Scenic")}</option>
                    <option value="food">{move || t(locale.get(), "餐饮", "Food")}</option>
                    <option value="park">{move || t(locale.get(), "公园", "Park")}</option>
                    <option value="transit">{move || t(locale.get(), "交通", "Transit")}</option>
                    <option value="event">{move || t(locale.get(), "活动", "Event")}</option>
                    <option value="custom">{move || t(locale.get(), "其他", "Custom")}</option>
                </select>
            </label>
        </div>
        <div class="admin-list-summary">
            <strong>{move || format!("{}–{}", first, last)}</strong>
            <span>{move || format!("{} {}", t(locale.get(), "条，共", "of"), result.total)}</span>
        </div>
        {if items.is_empty() {
            view! {
                <section class="empty-state">
                    <strong>{move || t(locale.get(), "没有符合条件的空间", "No Spaces match these filters")}</strong>
                    <span>{move || t(locale.get(), "换一个关键词，或切换状态筛选。", "Try another search or status filter.")}</span>
                </section>
            }.into_any()
        } else {
            view! {
                <section class="admin-space-table" aria-label=move || t(locale.get(), "空间列表", "Space list")>
                    <For
                        each=move || items.clone()
                        key=|space| format!("{}-{}", space.id, space.status)
                        children=move |space| view! { <AdminSpaceRow space=space reload=reload /> }
                    />
                </section>
            }.into_any()
        }}
        <nav class="admin-pagination" aria-label=move || t(locale.get(), "空间分页", "Space pagination")>
            <button class="button button-secondary-light" type="button" disabled={current_page <= 1} on:click=move |_| page.set(1)>
                {move || t(locale.get(), "首页", "First")}
            </button>
            <button class="button button-secondary-light" type="button" disabled={current_page <= 1} on:click=move |_| page.update(|value| *value = (*value - 1).max(1))>
                {move || t(locale.get(), "上一页", "Previous")}
            </button>
            <span>{move || format!("{} / {}", current_page, total_pages)}</span>
            <button class="button button-secondary-light" type="button" disabled={current_page >= total_pages} on:click=move |_| page.update(|value| *value += 1)>
                {move || t(locale.get(), "下一页", "Next")}
            </button>
            <button class="button button-secondary-light" type="button" disabled={current_page >= total_pages} on:click=move |_| page.set(total_pages)>
                {move || t(locale.get(), "末页", "Last")}
            </button>
        </nav>
    }
}

#[component]
fn AdminSpaceRow(space: SpaceMarker, reload: RwSignal<u32>) -> impl IntoView {
    let locale = use_i18n().locale;
    let message = RwSignal::new(None::<String>);
    let error = RwSignal::new(None::<String>);
    let confirm_delete = RwSignal::new(false);
    let manage_open = RwSignal::new(false);
    let name = space.name_zh.clone();
    let english_name = space.name_en.clone();
    let location = location_label(&space);
    let status = space.status.clone();
    let space_id = space.id.clone();
    let status_for_buttons = status.clone();

    let handle = move |result: Result<SpaceMarker, ServerFnError>, success: &str| match result {
        Ok(_) => {
            error.set(None);
            message.set(Some(success.to_string()));
            reload.update(|n| *n += 1);
        }
        Err(err) => {
            message.set(None);
            error.set(Some(err.to_string()));
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
            handle(result, "已关闭");
        }
    });
    Effect::new(move |_| {
        if let Some(result) = reactivate.value().get() {
            handle(result, "已恢复");
        }
    });
    Effect::new(move |_| {
        if let Some(result) = archive.value().get() {
            handle(result, "已归档为模板");
        }
    });
    Effect::new(move |_| {
        if let Some(result) = delete.value().get() {
            handle(result, "已移入已删除");
        }
    });

    view! {
        <article class="admin-space-row">
            <div class="admin-space-main">
                <div class="admin-space-title-line">
                    <a href=format!("/inspace/spaces/{}", space.id)><strong>{name}</strong></a>
                    <span class=move || format!("space-status-badge status-{}", status)>{status_label(&status, locale.get())}</span>
                </div>
                <p class="admin-space-location">{if location.is_empty() { "—".to_string() } else { location }}</p>
                <p class="admin-space-meta">
                    <span>{space_type_label(&space.space_type, locale.get())}</span>
                    <span>{if space.is_public { t(locale.get(), "公开", "Public") } else { t(locale.get(), "私密", "Private") }}</span>
                    <span>{english_name.unwrap_or_default()}</span>
                </p>
            </div>
            <div class="admin-space-actions">
                <button class="button button-secondary-light" type="button" on:click=move |_| manage_open.set(true)>
                    {move || t(locale.get(), "编辑空间", "Edit")}
                </button>
                <a class="button button-secondary-light" href=format!("/inspace/admin/guides?space_id={}", space.id)>
                    {move || t(locale.get(), "管理攻略", "Guides")}
                </a>
                {if status_for_buttons != "active" && status_for_buttons != "expired" {
                    view! { <button class="button button-secondary-light" type="button" on:click=move |_| { reactivate.dispatch(()); }>{move || t(locale.get(), "恢复", "Restore")}</button> }.into_any()
                } else {
                    view! { <button class="button button-secondary-light" type="button" on:click=move |_| { close.dispatch(()); }>{move || t(locale.get(), "关闭", "Close")}</button> }.into_any()
                }}
                {move || if confirm_delete.get() {
                    view! { <button class="button button-danger" type="button" on:click=move |_| { confirm_delete.set(false); delete.dispatch(()); }>{move || t(locale.get(), "确认移入已删除", "Confirm delete")}</button> }.into_any()
                } else {
                    view! { <button class="button button-danger-light" type="button" on:click=move |_| confirm_delete.set(true)>{move || t(locale.get(), "移入已删除", "Delete")}</button> }.into_any()
                }}
            </div>
            {move || message.get().map(|value| view! { <p class="form-success">{value}</p> })}
            {move || error.get().map(|value| view! { <p class="form-error">{value}</p> })}
        </article>
        {move || if manage_open.get() { view! { <ManageSpaceModal space=space.clone() open=manage_open /> }.into_any() } else { view! { <span aria-hidden="true"></span> }.into_any() }}
    }
}

fn optional_string(value: String) -> Option<String> {
    let value = value.trim().to_string();
    (!value.is_empty()).then_some(value)
}

fn parse_space_type(value: &str) -> Option<SpaceType> {
    match value {
        "scenic" => Some(SpaceType::Scenic),
        "food" => Some(SpaceType::Food),
        "park" => Some(SpaceType::Park),
        "transit" => Some(SpaceType::Transit),
        "event" => Some(SpaceType::Event),
        "custom" => Some(SpaceType::Custom),
        _ => None,
    }
}

fn status_label(status: &str, locale: crate::i18n::Locale) -> &'static str {
    match status {
        "active" => t(locale, "活跃", "Active"),
        "expired" => t(locale, "过期", "Expired"),
        "closed" => t(locale, "已关闭", "Closed"),
        "archived" => t(locale, "已删除", "Deleted"),
        "template" => t(locale, "模板", "Template"),
        _ => t(locale, "未知", "Unknown"),
    }
}

fn space_type_label(space_type: &SpaceType, locale: crate::i18n::Locale) -> &'static str {
    match space_type {
        SpaceType::Scenic => t(locale, "景点", "Scenic"),
        SpaceType::Food => t(locale, "餐饮", "Food"),
        SpaceType::Park => t(locale, "公园", "Park"),
        SpaceType::Transit => t(locale, "交通", "Transit"),
        SpaceType::Event => t(locale, "活动", "Event"),
        SpaceType::Custom => t(locale, "其他", "Custom"),
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
