use leptos::prelude::*;

use crate::i18n::{localize_optional, t, use_i18n};
use crate::server::guides::{
    list_cities, list_districts, list_guide_countries, list_guide_page, list_provinces, list_spots,
    GuidePageResult,
};

const PAGE_SIZE: i32 = 24;

#[component]
pub fn GuideBrowser() -> impl IntoView {
    let locale = use_i18n().locale;
    let selected_province = RwSignal::new(None::<String>);
    let selected_city = RwSignal::new(None::<String>);
    let selected_district = RwSignal::new(None::<String>);
    let selected_spot = RwSignal::new(None::<String>);
    let selected_country = RwSignal::new(None::<String>);
    let countries = Resource::new(
        || (),
        |_| async move { list_guide_countries().await.unwrap_or_default() },
    );
    let provinces = Resource::new(
        move || selected_country.get(),
        |country| async move { list_provinces(country).await.unwrap_or_default() },
    );
    let cities = Resource::new(
        move || (selected_country.get(), selected_province.get()),
        |(country, province)| async move { list_cities(country, province).await.unwrap_or_default() },
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
            list_districts(country, province, city)
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
            list_spots(country, province, city, district)
                .await
                .unwrap_or_default()
        },
    );
    let query_input = RwSignal::new(String::new());
    let applied_query = RwSignal::new(String::new());
    let page = RwSignal::new(1i32);

    // Any filter change resets to page 1, otherwise a deep page number would
    // survive into a much shorter result set.
    Effect::new(move |previous: Option<()>| {
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

    let guides = Resource::new(
        move || {
            (
                applied_query.get(),
                selected_country.get(),
                selected_province.get(),
                selected_city.get(),
                selected_district.get(),
                selected_spot.get(),
                page.get(),
            )
        },
        |(query, country, province, city, district, spot, page)| async move {
            let query = (!query.trim().is_empty()).then_some(query);
            list_guide_page(
                query, country, province, city, district, spot, page, PAGE_SIZE,
            )
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
            <div class="page-head directory-hero guide-directory-hero">
                <div class="directory-hero-copy">
                    <p class="survey-kicker">{move || t(locale.get(), "地点阅读索引", "Place reading index")}</p>
                    <h1>{move || t(locale.get(), "先选一个地方，再读那里留下的攻略。", "Choose a place, then read what people left there.")}</h1>
                    <p>{move || t(locale.get(), "路线、营业时间、避坑提示和现场经验，都应该跟着真实地点保存，而不是散落在没有上下文的信息流里。", "Routes, opening hours, warnings, and first-hand knowledge should stay attached to the real place, not disappear into a contextless feed.")}</p>
                </div>
                <div class="guide-browser-head-actions directory-hero-actions">
                    <a class="button button-primary" href="/inspace/my-spaces">
                        {move || t(locale.get(), "从我的空间开始写", "Write from My Spaces")}
                    </a>
                    <a class="guide-standalone-link" href="/inspace/guides/new">
                        {move || t(locale.get(), "直接写一篇", "Write directly")}
                    </a>
                </div>
            </div>
            <aside class="guide-writing-rule" aria-label=move || t(locale.get(), "攻略写作说明", "Guide writing guidance")>
                <strong>{move || t(locale.get(), "让攻略留在地点里", "Keep guides with the place")}</strong>
                <p>{move || t(
                    locale.get(),
                    "进入你管理的空间再点「写攻略」，地点会自动带入，读者也能从攻略回到空间，看见简介、故事和现场讨论。",
                    "Open a Space you manage and choose Write guide. The place is attached automatically, and readers can return to its profile, stories, and on-site discussion.",
                )}</p>
            </aside>
            <form
                class="guide-search-panel"
                role="search"
                on:submit=move |ev| {
                    ev.prevent_default();
                    applied_query.set(query_input.get().trim().to_string());
                }
            >
                <label class="guide-search-field">
                    <span>{move || t(locale.get(), "搜索地点或攻略标题", "Search places or guide titles")}</span>
                    <div class="guide-search-row">
                        <input
                            type="search"
                            prop:value=move || query_input.get()
                            placeholder=move || t(locale.get(), "例如：南昌、滕王阁、外滩夜行", "e.g. Nanchang, Tengwang Pavilion, the Bund at night")
                            on:input=move |ev| query_input.set(event_target_value(&ev))
                        />
                        <button class="button button-primary" type="submit">
                            {move || t(locale.get(), "搜索", "Search")}
                        </button>
                    </div>
                </label>
            </form>
            <div class="guide-filter-stack">
                <div class="guide-filter-heading">
                    <div>
                        <strong>{move || t(locale.get(), "沿着地点层级寻找", "Follow the place hierarchy")}</strong>
                        <span>{move || t(locale.get(), "省份、城市、区域、具体地点", "Province, city, district, and exact place")}</span>
                    </div>
                    <button
                        class="guide-clear-button"
                        type="button"
                        on:click=move |_| {
                            query_input.set(String::new());
                            applied_query.set(String::new());
                            selected_country.set(None);
                            selected_province.set(None);
                            selected_city.set(None);
                            selected_district.set(None);
                            selected_spot.set(None);
                            page.set(1);
                        }
                    >
                        {move || t(locale.get(), "清除全部", "Clear all")}
                    </button>
                </div>
                <div class="filter-row">
                <select
                    prop:value=move || selected_country.get().unwrap_or_default()
                    aria-label="Country"
                    on:change=move |ev| {
                        let value = event_target_value(&ev);
                        selected_country.set(if value.is_empty() { None } else { Some(value) });
                        selected_province.set(None);
                        selected_city.set(None);
                        selected_district.set(None);
                        selected_spot.set(None);
                    }
                >
                    <option value="">{move || t(locale.get(), "国家 / 地区", "Country / Region")}</option>
                    <Suspense fallback=move || view! { <option>{move || t(locale.get(), "加载中", "Loading")}</option> }>
                        {move || Suspend::new(async move {
                            let items = countries.await;
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
                    prop:value=move || selected_province.get().unwrap_or_default()
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
                    prop:value=move || selected_city.get().unwrap_or_default()
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
                    prop:value=move || selected_district.get().unwrap_or_default()
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
                    prop:value=move || selected_spot.get().unwrap_or_default()
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
            </div>
            <Suspense fallback=move || view! { <p class="directory-loading">{move || t(locale.get(), "正在加载攻略", "Loading guides")}</p> }>
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
                format!("找到 {total} 篇地点攻略，正在看第 {first} 至 {last} 篇")
            } else {
                format!("{total} place guides, showing {first} to {last}")
            }}</p>
            <span>{move || t(locale.get(), "按地点扫读，进入后再看完整内容", "Scan by place, then open the full record")}</span>
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
                            <span class="guide-list-location">{location}</span>
                            <span class="guide-list-enter">{move || t(locale.get(), "阅读", "Read")}</span>
                        </a>
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
