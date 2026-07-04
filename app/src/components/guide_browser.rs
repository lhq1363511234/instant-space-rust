use leptos::prelude::*;

use crate::server::guides::{list_guides, list_provinces};

#[component]
pub fn GuideBrowser() -> impl IntoView {
    let selected_province = RwSignal::new(None::<String>);
    let provinces = Resource::new(
        || (),
        |_| async move { list_provinces().await.unwrap_or_default() },
    );
    let guides = Resource::new(
        move || selected_province.get(),
        |province| async move { list_guides(province).await.unwrap_or_default() },
    );

    view! {
        <section class="guide-browser">
            <h1>"导览"</h1>
            <div class="filter-row">
                <select
                    aria-label="Province"
                    on:change=move |ev| {
                        let value = event_target_value(&ev);
                        selected_province.set(if value.is_empty() { None } else { Some(value) });
                    }
                >
                    <option value="">"省份"</option>
                    <Suspense fallback=move || view! { <option>"加载"</option> }>
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
                <select aria-label="City"><option value="">"城市"</option></select>
                <select aria-label="District"><option value="">"区域"</option></select>
                <select aria-label="Spot"><option value="">"地点"</option></select>
            </div>
            <Suspense fallback=move || view! { <p>"加载导览"</p> }>
                {move || Suspend::new(async move {
                    let items = guides.await;
                    view! {
                        <ul class="guide-list">
                            <For
                                each=move || items.clone()
                                key=|guide| guide.id
                                children=move |guide| view! {
                                    <li>
                                        <strong>{guide.title_zh}</strong>
                                        <span>{format!("{} / {}", guide.province, guide.city)}</span>
                                    </li>
                                }
                            />
                        </ul>
                    }
                })}
            </Suspense>
        </section>
    }
}
