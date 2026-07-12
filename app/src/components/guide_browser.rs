use leptos::prelude::*;

use crate::i18n::{localize_optional, t, use_i18n};
use crate::server::guides::{list_cities, list_districts, list_guides, list_provinces, list_spots};

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
    let guides = Resource::new(
        move || {
            (
                selected_province.get(),
                selected_city.get(),
                selected_district.get(),
                selected_spot.get(),
            )
        },
        |(province, city, district, spot)| async move {
            list_guides(province, city, district, spot)
                .await
                .unwrap_or_default()
        },
    );

    view! {
        <section class="guide-browser">
            <div class="page-head">
                <div>
                    <h1>{move || t(locale.get(), "全球攻略", "Global Guides")}</h1>
                    <p>{move || t(locale.get(), "浏览已发布攻略，或提交新的结构化攻略草稿。", "Browse published guides or submit a structured draft.")}</p>
                </div>
                <a class="button button-primary" href="/inspace/guides/new">
                    {move || t(locale.get(), "提交攻略", "Submit guide")}
                </a>
            </div>
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
                    let items = guides.await;
                    view! {
                        <ul class="guide-list">
                            <For
                                each=move || items.clone()
                                key=|guide| guide.id
                                children=move |guide| {
                                    let title_zh = guide.title_zh.clone();
                                    let title_en = guide.title_en.clone();
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
                                }
                            />
                        </ul>
                    }
                })}
            </Suspense>
        </section>
    }
}
