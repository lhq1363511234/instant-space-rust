use leptos::prelude::*;

use crate::i18n::{localize_optional, t, use_i18n};
use crate::server::guides::{
    list_cities, list_districts, list_guide_page, list_provinces, list_spots, GuidePageResult,
};

const PAGE_SIZE: i32 = 24;

#[component]
pub fn GuideBrowser() -> impl IntoView {
    let locale = use_i18n().locale;
    let selected_province = RwSignal::new(None::<String>);
    let selected_city = RwSignal::new(None::<String>);
    let selected_district = RwSignal::new(None::<String>);
    let selected_spot = RwSignal::new(None::<String>);
    let provinces = Resource::new(
        || (),
        |_| async move { list_provinces().await.unwrap_or_default() },
    );
    let cities = Resource::new(
        move || selected_province.get(),
        |province| async move {
            match province {
                Some(province) => list_cities(province).await.unwrap_or_default(),
                None => Vec::new(),
            }
        },
    );
    let districts = Resource::new(
        move || (selected_province.get(), selected_city.get()),
        |(province, city)| async move {
            match (province, city) {
                (Some(province), Some(city)) => {
                    list_districts(province, city).await.unwrap_or_default()
                }
                _ => Vec::new(),
            }
        },
    );
    let spots = Resource::new(
        move || {
            (
                selected_province.get(),
                selected_city.get(),
                selected_district.get(),
            )
        },
        |(province, city, district)| async move {
            match (province, city) {
                (Some(province), Some(city)) => list_spots(province, city, district)
                    .await
                    .unwrap_or_default(),
                _ => Vec::new(),
            }
        },
    );
    let query_input = RwSignal::new(String::new());
    let applied_query = RwSignal::new(String::new());
    let page = RwSignal::new(1i32);

    // Any filter change resets to page 1, otherwise a deep page number would
    // survive into a much shorter result set.
    Effect::new(move |previous: Option<()>| {
        selected_province.track();
        selected_city.track();
        selected_district.track();
        selected_spot.track();
        applied_query.track();
        if previous.is_some() {
            page.set(1);
        }
    });

    let guides = Resource::new(
        move || {
            (
                applied_query.get(),
                selected_province.get(),
                selected_city.get(),
                selected_district.get(),
                selected_spot.get(),
                page.get(),
            )
        },
        |(query, province, city, district, spot, page)| async move {
            let query = (!query.trim().is_empty()).then_some(query);
            list_guide_page(query, province, city, district, spot, page, PAGE_SIZE)
                .await
                .unwrap_or(GuidePageResult {
                    items: Vec::new(),
                    total: 0,
                    page: 1,
                    page_size: PAGE_SIZE,
                    total_pages: 1,
                })
        },
    );

    view! {
        <section class="guide-browser">
            <div class="page-head">
                <div>
                    <p class="survey-kicker">{move || t(locale.get(), "全部记录", "All records")}</p>
                    <h1>{move || t(locale.get(), "按目的地浏览攻略", "Browse guides by destination")}</h1>
                    <p>{move || t(locale.get(), "按省市、区域和地点筛选，找到路线、美食和避坑提示。", "Filter by region and place to find routes, food, and practical warnings.")}</p>
                </div>
                <div class="guide-browser-head-actions">
                    <a class="button button-primary" href="/inspace/my-spaces">
                        {move || t(locale.get(), "去我的空间写攻略", "Write from My Spaces")}
                    </a>
                    <a class="button button-secondary-light" href="/inspace/guides/new">
                        {move || t(locale.get(), "写独立攻略", "Write standalone")}
                    </a>
                </div>
            </div>
            <p class="guide-browser-note">
                {move || t(
                    locale.get(),
                    "攻略最好写在空间里：进入一个空间点「写攻略」，地点信息会自动带入，写完的攻略会挂在那个真实地点下。",
                    "Guides work best inside a Space: open a Space, click Write guide, and the place details are carried over so the guide stays attached to that real location.",
                )}
            </p>
            <form
                class="guide-search-panel"
                role="search"
                on:submit=move |ev| {
                    ev.prevent_default();
                    applied_query.set(query_input.get().trim().to_string());
                }
            >
                <label class="guide-search-field">
                    <span>{move || t(locale.get(), "搜索攻略标题或地点", "Search guide titles or places")}</span>
                    <div class="guide-search-row">
                        <input
                            type="search"
                            prop:value=move || query_input.get()
                            placeholder=move || t(locale.get(), "例如：京都、卢浮宫、外滩", "e.g. Kyoto, Louvre, the Bund")
                            on:input=move |ev| query_input.set(event_target_value(&ev))
                        />
                        <button class="button button-primary" type="submit">
                            {move || t(locale.get(), "搜索", "Search")}
                        </button>
                    </div>
                </label>
            </form>
            <div class="filter-row">
                <select
                    aria-label="Province"
                    on:change=move |ev| {
                        let value = event_target_value(&ev);
                        selected_province.set(if value.is_empty() { None } else { Some(value) });
                        selected_city.set(None);
                        selected_district.set(None);
                        selected_spot.set(None);
                    }
                >
                    <option value="">{move || t(locale.get(), "省份 / 地区", "Province / Region")}</option>
                    <Suspense fallback=move || view! { <option>{move || t(locale.get(), "加载中", "Loading")}</option> }>
                        {move || Suspend::new(async move {
                            let items = provinces.await;
                            view! {
                                <For
                                    each=move || items.clone()
                                    key=|item| item.province.clone()
                                    children=move |item| view! {
                                        <option value=item.province.clone()>{item.province.clone()}</option>
                                    }
                                />
                            }
                        })}
                    </Suspense>
                </select>
                <select
                    aria-label="City"
                    on:change=move |ev| {
                        let value = event_target_value(&ev);
                        selected_city.set(if value.is_empty() { None } else { Some(value) });
                        selected_district.set(None);
                        selected_spot.set(None);
                    }
                >
                    <option value="">{move || t(locale.get(), "城市", "City")}</option>
                    <Suspense fallback=move || view! { <option>{move || t(locale.get(), "加载中", "Loading")}</option> }>
                        {move || Suspend::new(async move {
                            let items = cities.await;
                            view! {
                                <For
                                    each=move || items.clone()
                                    key=|item| item.clone()
                                    children=move |item| view! {
                                        <option value=item.clone()>{item.clone()}</option>
                                    }
                                />
                            }
                        })}
                    </Suspense>
                </select>
                <select
                    aria-label="District"
                    on:change=move |ev| {
                        let value = event_target_value(&ev);
                        selected_district.set(if value.is_empty() { None } else { Some(value) });
                        selected_spot.set(None);
                    }
                >
                    <option value="">{move || t(locale.get(), "区域", "District")}</option>
                    <Suspense fallback=move || view! { <option>{move || t(locale.get(), "加载中", "Loading")}</option> }>
                        {move || Suspend::new(async move {
                            let items = districts.await;
                            view! {
                                <For
                                    each=move || items.clone()
                                    key=|item| item.clone()
                                    children=move |item| view! {
                                        <option value=item.clone()>{item.clone()}</option>
                                    }
                                />
                            }
                        })}
                    </Suspense>
                </select>
                <select
                    aria-label="Spot"
                    on:change=move |ev| {
                        let value = event_target_value(&ev);
                        selected_spot.set(if value.is_empty() { None } else { Some(value) });
                    }
                >
                    <option value="">{move || t(locale.get(), "地点", "Spot")}</option>
                    <Suspense fallback=move || view! { <option>{move || t(locale.get(), "加载中", "Loading")}</option> }>
                        {move || Suspend::new(async move {
                            let items = spots.await;
                            view! {
                                <For
                                    each=move || items.clone()
                                    key=|item| item.clone()
                                    children=move |item| view! {
                                        <option value=item.clone()>{item.clone()}</option>
                                    }
                                />
                            }
                        })}
                    </Suspense>
                </select>
            </div>
            <Suspense fallback=move || view! { <p>{move || t(locale.get(), "正在加载攻略", "Loading guides")}</p> }>
                {move || Suspend::new(async move {
                    let result = guides.await;
                    view! { <GuideResults result=result page=page /> }
                })}
            </Suspense>
        </section>
    }
}

fn pagination_window(current: i32, total: i32) -> Vec<i32> {
    let start = (current - 2).max(1);
    let end = (start + 4).min(total);
    let start = (end - 4).max(1);
    (start..=end).collect()
}

#[component]
fn GuideResults(result: GuidePageResult, page: RwSignal<i32>) -> impl IntoView {
    let locale = use_i18n().locale;
    let current = result.page;
    let total_pages = result.total_pages;
    let total = result.total;
    // The server clamps out-of-range pages; mirror that back so the controls
    // match what is actually on screen.
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
                <strong>{move || t(locale.get(), "没有找到匹配的攻略", "No matching guides")}</strong>
                <span>{move || t(locale.get(), "换一个地点或清除筛选条件。", "Try another place or clear the filters.")}</span>
            </div>
        }
        .into_any();
    }

    let page_numbers = pagination_window(current, total_pages);
    view! {
        <div class="directory-result-bar" aria-live="polite">
            <p>{move || if locale.get() == crate::i18n::Locale::Zh {
                format!("共 {total} 篇攻略，当前显示 {first}–{last}")
            } else {
                format!("{total} guides · showing {first}–{last}")
            }}</p>
            <span>{move || t(locale.get(), "按精选与更新时间排序", "Ordered by featured and recency")}</span>
        </div>
        <ul class="guide-list">
            {result.items.into_iter().map(|guide| {
                let title_zh = guide.title_zh.clone();
                let title_en = guide.title_en.clone();
                // The title already carries the spot name, so the row only
                // needs the wider geography to disambiguate.
                let location = format!("{} / {}", guide.province, guide.city);
                let href = format!("/inspace/guides/{}", guide.id);
                let edit_href = format!("/inspace/guides/{}/edit", guide.id);
                let can_edit = guide.can_edit;
                view! {
                    <li>
                        <a class="guide-list-link" href=href>
                            <strong>{move || localize_optional(locale.get(), &title_zh, title_en.as_deref())}</strong>
                        </a>
                        <span>{location}</span>
                        {can_edit.then(|| view! {
                            <div class="guide-list-actions">
                                <a class="button button-secondary-light" href=edit_href>{move || t(locale.get(), "编辑", "Edit")}</a>
                            </div>
                        })}
                    </li>
                }
            }).collect_view()}
        </ul>
        <nav class="directory-pagination" aria-label=move || t(locale.get(), "攻略列表分页", "Guide list pagination")>
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
