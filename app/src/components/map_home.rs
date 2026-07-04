use leptos::prelude::*;

use crate::server::spaces::list_spaces;

#[component]
pub fn MapHome() -> impl IntoView {
    let query = RwSignal::new(String::new());
    let spaces = Resource::new(
        move || query.get(),
        |q| async move {
            let value = if q.trim().is_empty() { None } else { Some(q) };
            list_spaces(value, None).await.unwrap_or_default()
        },
    );
    Effect::new(move |_| {
        instant_map_ui::mount("map", "https://demotiles.maplibre.org/style.json");
    });

    view! {
        <section class="map-layout">
            <div id="map" class="map-canvas" aria-label="Instant Space map">
                <p class="map-loading">"MapLibre map mounts here."</p>
            </div>
            <aside class="space-panel">
                <label>
                    "搜索空间"
                    <input
                        type="search"
                        aria-label="Search spaces"
                        on:input=move |ev| query.set(event_target_value(&ev))
                    />
                </label>
                <select aria-label="Space type">
                    <option value="">"全部"</option>
                    <option value="scenic">"景点"</option>
                    <option value="food">"美食"</option>
                    <option value="park">"公园"</option>
                    <option value="transit">"交通"</option>
                </select>
                <Suspense fallback=move || view! { <p>"加载空间"</p> }>
                    {move || Suspend::new(async move {
                        let items = spaces.await;
                        view! {
                            <ul class="space-list">
                                <For
                                    each=move || items.clone()
                                    key=|space| space.id.clone()
                                    children=move |space| view! {
                                        <li>
                                            <button type="button">{space.name_zh}</button>
                                        </li>
                                    }
                                />
                            </ul>
                        }
                    })}
                </Suspense>
            </aside>
        </section>
    }
}
