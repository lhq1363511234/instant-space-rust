use instant_domain::spaces::SpaceType;
use leptos::prelude::*;

use crate::{
    components::private_verify::PrivateVerify,
    server::spaces::{list_spaces, SpaceMarker},
};

#[component]
pub fn MapHome() -> impl IntoView {
    let query = RwSignal::new(String::new());
    let selected_type = RwSignal::new(None::<SpaceType>);
    let spaces = Resource::new(
        move || (query.get(), selected_type.get()),
        |(q, space_type)| async move {
            let value = if q.trim().is_empty() { None } else { Some(q) };
            list_spaces(value, space_type).await.unwrap_or_default()
        },
    );

    Effect::new(move |_| {
        instant_map_ui::mount("map", "https://demotiles.maplibre.org/style.json");
    });

    view! {
        <section class="map-layout">
            <div id="map" class="map-canvas" aria-label="Instant Space map">
                <div class="map-loading" aria-live="polite">
                    <span class="map-loading-dot" aria-hidden="true"></span>
                    <span>"Preparing the map"</span>
                </div>
            </div>

            <aside class="explorer-panel" aria-label="Space explorer">
                <div class="explorer-panel-header">
                    <div>
                        <p class="eyebrow">"Explore nearby"</p>
                        <h1>"Find your next space"</h1>
                    </div>
                    <Suspense fallback=move || view! { <span class="result-count">"Loading"</span> }>
                        {move || Suspend::new(async move {
                            let items = spaces.await;
                            view! { <span class="result-count">{format!("{} spaces", items.len())}</span> }
                        })}
                    </Suspense>
                </div>

                <label class="search-control">
                    <span>"Search spaces"</span>
                    <input
                        type="search"
                        aria-label="Search spaces"
                        placeholder="Try Bund, tea, park..."
                        on:input=move |ev| query.set(event_target_value(&ev))
                    />
                </label>

                <div class="filter-row" aria-label="Space type filters">
                    <button
                        type="button"
                        class=move || filter_chip_class(selected_type.get().is_none())
                        aria-pressed=move || selected_type.get().is_none()
                        on:click=move |_| selected_type.set(None)
                    >
                        "All"
                    </button>
                    <button
                        type="button"
                        class=move || filter_chip_class(selected_type.get() == Some(SpaceType::Scenic))
                        aria-pressed=move || selected_type.get() == Some(SpaceType::Scenic)
                        on:click=move |_| selected_type.set(Some(SpaceType::Scenic))
                    >
                        "Scenic"
                    </button>
                    <button
                        type="button"
                        class=move || filter_chip_class(selected_type.get() == Some(SpaceType::Food))
                        aria-pressed=move || selected_type.get() == Some(SpaceType::Food)
                        on:click=move |_| selected_type.set(Some(SpaceType::Food))
                    >
                        "Food"
                    </button>
                    <button
                        type="button"
                        class=move || filter_chip_class(selected_type.get() == Some(SpaceType::Park))
                        aria-pressed=move || selected_type.get() == Some(SpaceType::Park)
                        on:click=move |_| selected_type.set(Some(SpaceType::Park))
                    >
                        "Park"
                    </button>
                    <button
                        type="button"
                        class=move || filter_chip_class(selected_type.get() == Some(SpaceType::Transit))
                        aria-pressed=move || selected_type.get() == Some(SpaceType::Transit)
                        on:click=move |_| selected_type.set(Some(SpaceType::Transit))
                    >
                        "Transit"
                    </button>
                    <button
                        type="button"
                        class=move || filter_chip_class(selected_type.get() == Some(SpaceType::Event))
                        aria-pressed=move || selected_type.get() == Some(SpaceType::Event)
                        on:click=move |_| selected_type.set(Some(SpaceType::Event))
                    >
                        "Event"
                    </button>
                </div>

                <Suspense fallback=move || view! { <SpaceListSkeleton /> }>
                    {move || Suspend::new(async move {
                        let items = spaces.await;
                        if items.is_empty() {
                            view! {
                                <div class="empty-state">
                                    <strong>"No spaces found"</strong>
                                    <span>"Try another keyword or clear the filter."</span>
                                </div>
                            }
                                .into_any()
                        } else {
                            view! {
                                <ul class="space-list" aria-label="Available spaces">
                                    <For
                                        each=move || items.clone()
                                        key=|space| space.id.clone()
                                        children=move |space| view! { <SpaceCard space=space /> }
                                    />
                                </ul>
                            }
                                .into_any()
                        }
                    })}
                </Suspense>
            </aside>
        </section>
    }
}

#[component]
fn SpaceCard(space: SpaceMarker) -> impl IntoView {
    let is_public = space.is_public;
    let space_id = space.id.clone();
    let space_name = space.name_zh.clone();
    let location = location_label(&space);
    let type_label = space_type_label(&space.space_type);
    let status_label = if is_public { "Public" } else { "Private" };
    let status_class = if is_public {
        "space-badge space-badge-public"
    } else {
        "space-badge space-badge-private"
    };

    view! {
        <li>
            <article class="space-card">
                <div class="space-card-main">
                    <button class="space-card-action" type="button">{space.name_zh}</button>
                    <p>{location}</p>
                </div>
                <div class="space-card-meta">
                    <span class="space-badge">{type_label}</span>
                    <span class=status_class>{status_label}</span>
                    <span class="space-count">{space.online_count}" online"</span>
                </div>
                {(!is_public).then(move || view! {
                    <PrivateVerify
                        space_id=space_id.clone()
                        space_name=space_name.clone()
                    />
                })}
            </article>
        </li>
    }
}

#[component]
fn SpaceListSkeleton() -> impl IntoView {
    view! {
        <div class="space-list-skeleton" aria-label="Loading spaces">
            <span></span>
            <span></span>
            <span></span>
        </div>
    }
}

fn filter_chip_class(is_active: bool) -> &'static str {
    if is_active {
        "filter-chip is-active"
    } else {
        "filter-chip"
    }
}

fn space_type_label(space_type: &SpaceType) -> &'static str {
    match space_type {
        SpaceType::Scenic => "Scenic",
        SpaceType::Food => "Food",
        SpaceType::Park => "Park",
        SpaceType::Transit => "Transit",
        SpaceType::Event => "Event",
        SpaceType::Custom => "Custom",
    }
}

fn location_label(space: &SpaceMarker) -> String {
    let mut parts = Vec::new();
    if let Some(city) = &space.city {
        parts.push(city.as_str());
    }
    if let Some(district) = &space.district {
        parts.push(district.as_str());
    }
    if parts.is_empty() {
        space
            .province
            .clone()
            .unwrap_or_else(|| "Location pending".to_string())
    } else {
        parts.join(" / ")
    }
}
