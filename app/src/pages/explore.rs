use instant_domain::spaces::SpaceType;
use leptos::prelude::*;

use crate::{
    components::space_form::{provide_create_space_modal, use_create_space_modal},
    i18n::{localize_optional, t, use_i18n},
    server::spaces::{list_space_page, SpaceMarker, SpacePageResult},
};

const PAGE_SIZE: i32 = 20;

/// Scalable, server-paginated discovery. The page intentionally keeps every
/// result compact; detailed content belongs to the Space page.
#[component]
pub fn ExplorePage() -> impl IntoView {
    let locale = use_i18n().locale;
    let query_input = RwSignal::new(String::new());
    let applied_query = RwSignal::new(String::new());
    let selected_type = RwSignal::new(None::<SpaceType>);
    let page = RwSignal::new(1i32);
    let modal = use_create_space_modal().unwrap_or_else(provide_create_space_modal);

    let spaces = Resource::new(
        move || (applied_query.get(), selected_type.get(), page.get()),
        |(query, kind, page)| async move {
            let query = (!query.trim().is_empty()).then_some(query);
            list_space_page(query, kind, page, PAGE_SIZE).await
        },
    );

    view! {
        <main id="main-content" class="page explore-page explore-directory">
            <header class="explore-directory-head">
                <div>
                    <p class="eyebrow">{move || t(locale.get(), "空间探索", "Space discovery")}</p>
                    <h1>{move || t(locale.get(), "找到你要去的空间", "Find the Space you need")}</h1>
                    <p>{move || t(locale.get(), "搜索地点或空间名称，列表只保留判断所需的信息；攻略、讨论和分享都在空间内部。", "Search by place or Space name. The list keeps only decision-making details; guides, discussion, and sharing live inside each Space.")}</p>
                </div>
                <div class="explore-head-actions">
                    <a class="button button-secondary" href="/inspace/map">{move || t(locale.get(), "地图模式", "Map view")}</a>
                    <button class="button button-primary" type="button" on:click=move |_| modal.open.set(true)>{move || t(locale.get(), "创建空间", "Create Space")}</button>
                </div>
            </header>

            <section class="explore-content" aria-label=move || t(locale.get(), "空间目录", "Space directory")>
                <form
                    class="explore-search-panel"
                    role="search"
                    on:submit=move |ev| {
                        ev.prevent_default();
                        applied_query.set(query_input.get().trim().to_string());
                        page.set(1);
                    }
                >
                    <label class="explore-search-field">
                        <span>{move || t(locale.get(), "搜索地点或空间", "Search places or Spaces")}</span>
                        <div class="explore-search-row">
                            <input
                                type="search"
                                prop:value=move || query_input.get()
                                placeholder=move || t(locale.get(), "输入名称、城市、区域或具体地点", "Name, city, district, or place")
                                on:input=move |ev| query_input.set(event_target_value(&ev))
                            />
                            <button class="button button-primary" type="submit">{move || t(locale.get(), "搜索", "Search")}</button>
                        </div>
                    </label>
                    <div class="explore-filter-line">
                        <span>{move || t(locale.get(), "类型", "Type")}</span>
                        <div class="filter-chip-row">{space_type_buttons(locale, selected_type, page)}</div>
                        <button
                            class="explore-clear-button"
                            type="button"
                            on:click=move |_| {
                                query_input.set(String::new());
                                applied_query.set(String::new());
                                selected_type.set(None);
                                page.set(1);
                            }
                        >
                            {move || t(locale.get(), "清除筛选", "Clear filters")}
                        </button>
                    </div>
                </form>

                <Suspense fallback=move || view! { <div class="directory-loading">{move || t(locale.get(), "正在查找空间…", "Finding Spaces…")}</div> }>
                    {move || Suspend::new(async move {
                        match spaces.await {
                            Ok(result) => view! { <ExploreResults result=result page=page /> }.into_any(),
                            Err(_) => view! {
                                <div class="directory-empty" role="alert">
                                    <strong>{move || t(locale.get(), "空间列表暂时无法加载", "Spaces could not be loaded")}</strong>
                                    <span>{move || t(locale.get(), "请稍后重试，或切换到地图模式。", "Try again later or switch to map view.")}</span>
                                </div>
                            }.into_any(),
                        }
                    })}
                </Suspense>
            </section>
        </main>
    }
}

fn space_type_buttons(
    locale: RwSignal<crate::i18n::Locale>,
    selected: RwSignal<Option<SpaceType>>,
    page: RwSignal<i32>,
) -> impl IntoView {
    let types = [
        (None, "全部", "All"),
        (Some(SpaceType::Scenic), "景点", "Scenic"),
        (Some(SpaceType::Food), "美食", "Food"),
        (Some(SpaceType::Park), "公园", "Park"),
        (Some(SpaceType::Transit), "交通", "Transit"),
        (Some(SpaceType::Event), "活动", "Events"),
        (Some(SpaceType::Custom), "其他", "Other"),
    ];
    types
        .into_iter()
        .map(|(kind, zh, en)| {
            let class_kind = kind.clone();
            let click_kind = kind.clone();
            view! {
                <button
                    type="button"
                    class=move || if selected.get() == class_kind { "filter-chip is-active" } else { "filter-chip" }
                    aria-pressed=move || selected.get() == kind
                    on:click=move |_| {
                        selected.set(click_kind.clone());
                        page.set(1);
                    }
                >
                    {move || t(locale.get(), zh, en)}
                </button>
            }
        })
        .collect_view()
}

#[component]
fn ExploreResults(result: SpacePageResult, page: RwSignal<i32>) -> impl IntoView {
    let locale = use_i18n().locale;
    let current = result.page;
    let total_pages = result.total_pages;
    let total = result.total;
    // The server clamps out-of-range pages; mirror that back into the signal so
    // the pagination controls stay in sync with what is actually rendered.
    if page.get_untracked() != current {
        page.set(current);
    }
    let first = if total == 0 {
        0
    } else {
        i64::from((current - 1) * result.page_size + 1)
    };
    let last = (i64::from(current * result.page_size)).min(total);

    if result.items.is_empty() {
        return view! {
            <div class="directory-empty">
                <strong>{move || t(locale.get(), "没有找到匹配的空间", "No matching Spaces")}</strong>
                <span>{move || t(locale.get(), "换一个地点名称或清除类型筛选。", "Try another place name or clear the type filter.")}</span>
            </div>
        }
        .into_any();
    }

    let page_numbers = pagination_window(current, total_pages);
    view! {
        <div class="directory-result-bar" aria-live="polite">
            <p>{move || if locale.get() == crate::i18n::Locale::Zh {
                format!("共 {total} 个空间，当前显示 {first}–{last}")
            } else {
                format!("{total} Spaces · showing {first}–{last}")
            }}</p>
            <span>{move || t(locale.get(), "按推荐与创建时间排序", "Ordered by relevance and recency")}</span>
        </div>
        <div class="space-directory-list">
            {result.items.into_iter().map(|space| view! { <SpaceRow space=space /> }).collect_view()}
        </div>
        <nav class="directory-pagination" aria-label=move || t(locale.get(), "空间列表分页", "Space list pagination")>
            <button
                type="button"
                class="pagination-control"
                disabled={current <= 1}
                on:click=move |_| page.set(1)
            >
                {move || t(locale.get(), "首页", "First")}
            </button>
            <button
                type="button"
                class="pagination-control"
                disabled={current <= 1}
                on:click=move |_| page.update(|value| *value = (*value - 1).max(1))
            >
                {move || t(locale.get(), "上一页", "Previous")}
            </button>
            <div class="pagination-pages">
                {page_numbers.into_iter().map(|number| view! {
                    <button
                        type="button"
                        class=if number == current { "pagination-page is-current" } else { "pagination-page" }
                        aria-current=(number == current).then_some("page")
                        on:click=move |_| page.set(number)
                    >{number}</button>
                }).collect_view()}
            </div>
            <button
                type="button"
                class="pagination-control"
                disabled={current >= total_pages}
                on:click=move |_| page.update(|value| *value = (*value + 1).min(total_pages))
            >
                {move || t(locale.get(), "下一页", "Next")}
            </button>
            <button
                type="button"
                class="pagination-control"
                disabled={current >= total_pages}
                on:click=move |_| page.set(total_pages)
            >
                {move || t(locale.get(), "末页", "Last")}
            </button>
        </nav>
    }
    .into_any()
}

fn pagination_window(current: i32, total: i32) -> Vec<i32> {
    let start = (current - 2).max(1);
    let end = (start + 4).min(total);
    let start = (end - 4).max(1);
    (start..=end).collect()
}

#[component]
fn SpaceRow(space: SpaceMarker) -> impl IntoView {
    let locale = use_i18n().locale;
    let href = format!("/inspace/spaces/{}", space.id);
    let title_zh = space.name_zh.clone();
    let title_en = space.name_en.clone();
    let aria_title_zh = title_zh.clone();
    let aria_title_en = title_en.clone();
    let location = [
        space.country.clone(),
        space.province.clone(),
        space.city.clone(),
        space.district.clone(),
        space.spot_name.clone(),
    ]
    .into_iter()
    .flatten()
    .filter(|value| !value.trim().is_empty())
    .fold(Vec::<String>::new(), |mut values, value| {
        if !values.contains(&value) {
            values.push(value);
        }
        values
    });
    let location = location
        .into_iter()
        .rev()
        .take(3)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join(" · ");
    let (kind_zh, kind_en) = match space.space_type {
        SpaceType::Scenic => ("景点", "Scenic"),
        SpaceType::Food => ("美食", "Food"),
        SpaceType::Park => ("公园", "Park"),
        SpaceType::Transit => ("交通", "Transit"),
        SpaceType::Event => ("活动", "Event"),
        SpaceType::Custom => ("其他", "Other"),
    };
    let is_public = space.is_public;
    let online_count = space.online_count;

    view! {
        <article class="space-directory-row">
            <a class="space-directory-link" href=href aria-label=move || format!("{}：{}", t(locale.get(), "进入空间", "Open Space"), localize_optional(locale.get(), &aria_title_zh, aria_title_en.as_deref()))>
                <div class="space-directory-main">
                    <div class="space-directory-title-line">
                        <h2>{move || localize_optional(locale.get(), &title_zh, title_en.as_deref())}</h2>
                        <span class="space-type-label">{move || t(locale.get(), kind_zh, kind_en)}</span>
                    </div>
                    <p class="space-directory-location">{if location.is_empty() { "—".to_string() } else { location }}</p>
                </div>
                <div class="space-directory-status">
                    <span>{move || if is_public { t(locale.get(), "公开", "Public") } else { t(locale.get(), "需访问码", "Access code") }}</span>
                    {(online_count > 0).then(|| view! { <span>{format!("{online_count} online")}</span> })}
                </div>
                <span class="space-directory-enter" aria-hidden="true">"→"</span>
            </a>
        </article>
    }
}
