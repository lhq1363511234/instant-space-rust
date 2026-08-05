use instant_domain::spaces::SpaceType;
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

use crate::{
    components::{
        space_experience_modal::OpenSpaceLink,
        space_form::{provide_create_space_modal, use_create_space_modal},
    },
    i18n::{localize_optional, t, use_i18n},
    pages::space::SpacePanel,
    server::spaces::{
        list_space_filter_cities, list_space_filter_countries, list_space_filter_districts,
        list_space_filter_provinces, list_space_filter_spots, list_space_page, SpaceMarker,
        SpacePageResult,
    },
};

const PAGE_SIZE: i32 = 20;

/// Scalable, server-paginated discovery. The page intentionally keeps every
/// result compact; detailed content belongs to the Space page.
#[component]
pub fn ExplorePage() -> impl IntoView {
    let locale = use_i18n().locale;
    let query_map = use_query_map();
    let initial_query = query_map.with_untracked(|params| params.get("q").unwrap_or_default());
    let query_input = RwSignal::new(initial_query.clone());
    let applied_query = RwSignal::new(initial_query);
    let selected_type = RwSignal::new(None::<SpaceType>);
    let selected_country = RwSignal::new(None::<String>);
    let selected_province = RwSignal::new(None::<String>);
    let selected_city = RwSignal::new(None::<String>);
    let selected_district = RwSignal::new(None::<String>);
    let selected_spot = RwSignal::new(None::<String>);
    let page = RwSignal::new(1i32);
    let modal = use_create_space_modal().unwrap_or_else(provide_create_space_modal);

    let countries = Resource::new(
        || (),
        |_| async move { list_space_filter_countries().await.unwrap_or_default() },
    );
    let provinces = Resource::new(
        move || selected_country.get(),
        |country| async move {
            list_space_filter_provinces(country)
                .await
                .unwrap_or_default()
        },
    );
    let cities = Resource::new(
        move || (selected_country.get(), selected_province.get()),
        |(country, province)| async move {
            list_space_filter_cities(country, province)
                .await
                .unwrap_or_default()
        },
    );

    let districts = Resource::new(
        move || {
            (
                selected_country.get(),
                selected_province.get(),
                selected_city.get(),
            )
        },
        |(country, province, city)| async move {
            list_space_filter_districts(country, province, city)
                .await
                .unwrap_or_default()
        },
    );
    let spots = Resource::new(
        move || {
            (
                selected_country.get(),
                selected_province.get(),
                selected_city.get(),
                selected_district.get(),
            )
        },
        |(country, province, city, district)| async move {
            list_space_filter_spots(country, province, city, district)
                .await
                .unwrap_or_default()
        },
    );

    Effect::new(move |previous: Option<()>| {
        selected_type.track();
        selected_country.track();
        selected_province.track();
        selected_city.track();
        selected_district.track();
        selected_spot.track();
        applied_query.track();
        if previous.is_some() {
            page.set(1);
        }
    });

    let spaces = Resource::new(
        move || {
            (
                applied_query.get(),
                selected_type.get(),
                selected_country.get(),
                selected_province.get(),
                selected_city.get(),
                selected_district.get(),
                selected_spot.get(),
                page.get(),
            )
        },
        |(query, kind, country, province, city, district, spot, page)| async move {
            let query = (!query.trim().is_empty()).then_some(query);
            list_space_page(
                query, kind, country, province, city, district, spot, page, PAGE_SIZE,
            )
            .await
        },
    );

    view! {
        <main id="main-content" class="page explore-page explore-directory">
            <header class="explore-directory-head directory-hero">
                <div class="directory-hero-copy">
                    <p class="eyebrow">{move || t(locale.get(), "地点索引", "Place index")}</p>
                    <h1>{move || t(locale.get(), "从一个地点，进入它的空间。", "Start with a place. Enter its Space.")}</h1>
                    <p>{move || t(locale.get(), "搜索你正要去、曾经到过或真正熟悉的地方。空间里保存攻略、现场讨论，以及后来的人仍能读到的地点记忆。", "Search for somewhere you are visiting, have been to, or know well. Each Space keeps practical guides, on-site discussion, and memories tied to that place.")}</p>
                </div>
                <div class="explore-head-actions directory-hero-actions">
                    <a class="button button-secondary" href="/inspace/map">{move || t(locale.get(), "在地图上寻找", "Find on the map")}</a>
                    <button class="button button-primary" type="button" on:click=move |_| modal.open.set(true)>{move || t(locale.get(), "为熟悉的地点建空间", "Create a Space for a place")}</button>
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
                        <span>{move || t(locale.get(), "先输入一个地点", "Start with a place")}</span>
                        <div class="explore-search-row">
                            <input
                                type="search"
                                prop:value=move || query_input.get()
                                placeholder=move || t(locale.get(), "例如：南昌、滕王阁、公司楼下的公园", "e.g. Nanchang, Tengwang Pavilion, the park near work")
                                on:input=move |ev| query_input.set(event_target_value(&ev))
                            />
                            <button class="button button-primary" type="submit">{move || t(locale.get(), "搜索", "Search")}</button>
                        </div>
                    </label>
                    <div class="explore-filter-line">
                        <span>{move || t(locale.get(), "先选类型，再按地点逐级缩小", "Choose a type, then narrow by place")}</span>
                        <div class="filter-chip-row">{space_type_buttons(locale, selected_type, page)}</div>
                    </div>
                    <div class="explore-location-filters">
                        <select
                            aria-label=move || t(locale.get(), "国家或地区", "Country or region")
                            prop:value=move || selected_country.get().unwrap_or_default()
                            on:change=move |ev| {
                                let value = event_target_value(&ev);
                                selected_country.set((!value.is_empty()).then_some(value));
                                selected_province.set(None);
                                selected_city.set(None);
                                selected_district.set(None);
                                selected_spot.set(None);
                            }
                        >
                            <option value="">{move || t(locale.get(), "全部国家 / 地区", "All countries / regions")}</option>
                            <Suspense fallback=move || view! { <option>{move || t(locale.get(), "加载中", "Loading")}</option> }>
                                {move || Suspend::new(async move {
                                    let items = countries.await;
                                    view! { <For each=move || items.clone() key=|item| item.clone() children=move |item| view! { <option value=item.clone()>{item.clone()}</option> } /> }
                                })}
                            </Suspense>
                        </select>
                        <select
                            aria-label=move || t(locale.get(), "省份或地区", "Province or region")
                            prop:value=move || selected_province.get().unwrap_or_default()
                            on:change=move |ev| {
                                let value = event_target_value(&ev);
                                selected_province.set((!value.is_empty()).then_some(value));
                                selected_city.set(None);
                                selected_district.set(None);
                                selected_spot.set(None);
                            }
                        >
                            <option value="">{move || t(locale.get(), "全部省份 / 地区", "All provinces / regions")}</option>
                            <Suspense fallback=move || view! { <option>{move || t(locale.get(), "加载中", "Loading")}</option> }>
                                {move || Suspend::new(async move {
                                    let items = provinces.await;
                                    view! { <For each=move || items.clone() key=|item| item.clone() children=move |item| view! { <option value=item.clone()>{item.clone()}</option> } /> }
                                })}
                            </Suspense>
                        </select>
                        <select
                            aria-label=move || t(locale.get(), "城市", "City")
                            prop:value=move || selected_city.get().unwrap_or_default()
                            on:change=move |ev| {
                                let value = event_target_value(&ev);
                                selected_city.set((!value.is_empty()).then_some(value));
                                selected_district.set(None);
                                selected_spot.set(None);
                            }
                        >
                            <option value="">{move || t(locale.get(), "全部城市", "All cities")}</option>
                            <Suspense fallback=move || view! { <option>{move || t(locale.get(), "加载中", "Loading")}</option> }>
                                {move || Suspend::new(async move {
                                    let items = cities.await;
                                    view! { <For each=move || items.clone() key=|item| item.clone() children=move |item| view! { <option value=item.clone()>{item.clone()}</option> } /> }
                                })}
                            </Suspense>
                        </select>
                        <select
                            aria-label=move || t(locale.get(), "区域", "District")
                            prop:value=move || selected_district.get().unwrap_or_default()
                            on:change=move |ev| {
                                let value = event_target_value(&ev);
                                selected_district.set((!value.is_empty()).then_some(value));
                                selected_spot.set(None);
                            }
                        >
                            <option value="">{move || t(locale.get(), "全部区域", "All districts")}</option>
                            <Suspense fallback=move || view! { <option>{move || t(locale.get(), "加载中", "Loading")}</option> }>
                                {move || Suspend::new(async move {
                                    let items = districts.await;
                                    view! { <For each=move || items.clone() key=|item| item.clone() children=move |item| view! { <option value=item.clone()>{item.clone()}</option> } /> }
                                })}
                            </Suspense>
                        </select>
                        <select
                            aria-label=move || t(locale.get(), "具体地点", "Exact place")
                            prop:value=move || selected_spot.get().unwrap_or_default()
                            on:change=move |ev| {
                                let value = event_target_value(&ev);
                                selected_spot.set((!value.is_empty()).then_some(value));
                            }
                        >
                            <option value="">{move || t(locale.get(), "全部地点", "All places")}</option>
                            <Suspense fallback=move || view! { <option>{move || t(locale.get(), "加载中", "Loading")}</option> }>
                                {move || Suspend::new(async move {
                                    let items = spots.await;
                                    view! { <For each=move || items.clone() key=|item| item.clone() children=move |item| view! { <option value=item.clone()>{item.clone()}</option> } /> }
                                })}
                            </Suspense>
                        </select>
                        <button
                            class="explore-clear-button"
                            type="button"
                            on:click=move |_| {
                                query_input.set(String::new());
                                applied_query.set(String::new());
                                selected_type.set(None);
                                selected_country.set(None);
                                selected_province.set(None);
                                selected_city.set(None);
                                selected_district.set(None);
                                selected_spot.set(None);
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
                format!("找到 {total} 个空间，正在看第 {first} 至 {last} 个")
            } else {
                format!("{total} Spaces, showing {first} to {last}")
            }}</p>
            <span>{move || t(locale.get(), "先看地点，再决定是否进入", "Scan the place first, then decide whether to enter")}</span>
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
    let space_id = space.id.clone();
    let title_zh = space.name_zh.clone();
    let title_en = space.name_en.clone();
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
            <OpenSpaceLink space_id=space_id initial_panel=SpacePanel::Wall class="space-directory-link">
                <div class="space-directory-main">
                    <div class="space-directory-title-line">
                        <h2>{move || localize_optional(locale.get(), &title_zh, title_en.as_deref())}</h2>
                        <span class="space-type-label">{move || t(locale.get(), kind_zh, kind_en)}</span>
                    </div>
                    <p class="space-directory-location">{move || if location.is_empty() { t(locale.get(), "地点信息待补充", "Location details pending").to_string() } else { location.clone() }}</p>
                </div>
                <div class="space-directory-status">
                    <span>{move || if is_public { t(locale.get(), "公开", "Public") } else { t(locale.get(), "需访问码", "Access code") }}</span>
                    {(online_count > 0).then(|| view! { <span>{move || if locale.get() == crate::i18n::Locale::Zh { format!("{online_count} 人在线") } else { format!("{online_count} online") }}</span> })}
                </div>
                <span class="space-directory-enter" aria-hidden="true">{move || t(locale.get(), "进入", "Open")}</span>
            </OpenSpaceLink>
        </article>
    }
}
