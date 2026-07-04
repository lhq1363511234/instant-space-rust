use instant_domain::spaces::SpaceType;
use instant_map_ui::{MapProjection, MapStyle};
use leptos::prelude::*;

use crate::{
    components::private_verify::PrivateVerify,
    server::spaces::{list_spaces, SpaceMarker},
};

#[component]
pub fn MapHome() -> impl IntoView {
    let query = RwSignal::new(String::new());
    let selected_type = RwSignal::new(None::<SpaceType>);
    let selected_space = RwSignal::new(None::<SpaceMarker>);
    let map_style = RwSignal::new(MapStyle::Road);
    let map_projection = RwSignal::new(MapProjection::Flat2d);
    let spaces = Resource::new(
        move || (query.get(), selected_type.get()),
        |(q, space_type)| async move {
            let value = if q.trim().is_empty() { None } else { Some(q) };
            list_spaces(value, space_type).await.unwrap_or_default()
        },
    );

    Effect::new(move |_| {
        instant_map_ui::mount("map", MapStyle::Road, MapProjection::Flat2d);
    });

    Effect::new(move |_| {
        query.get();
        selected_type.get();
        selected_space.set(None);
    });

    view! {
        <section class="map-layout">
            <div id="map" class="map-canvas" aria-label="Instant Space map">
                <div class="map-loading" aria-live="polite">
                    <span class="map-loading-dot" aria-hidden="true"></span>
                    <span>"Loading real map tiles"</span>
                </div>
            </div>

            <div class="map-vignette" aria-hidden="true"></div>

            <section class="map-filter-panel" aria-label="Space filters">
                <div class="map-filter-heading">
                    <div>
                        <p class="eyebrow">"Instant Space"</p>
                        <h1>"Explore live spaces"</h1>
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
                        aria-pressed=move || aria_pressed(selected_type.get().is_none())
                        on:click=move |_| selected_type.set(None)
                    >
                        "All"
                    </button>
                    <button
                        type="button"
                        class=move || filter_chip_class(selected_type.get() == Some(SpaceType::Scenic))
                        aria-pressed=move || aria_pressed(selected_type.get() == Some(SpaceType::Scenic))
                        on:click=move |_| selected_type.set(Some(SpaceType::Scenic))
                    >
                        "Scenic"
                    </button>
                    <button
                        type="button"
                        class=move || filter_chip_class(selected_type.get() == Some(SpaceType::Food))
                        aria-pressed=move || aria_pressed(selected_type.get() == Some(SpaceType::Food))
                        on:click=move |_| selected_type.set(Some(SpaceType::Food))
                    >
                        "Food"
                    </button>
                    <button
                        type="button"
                        class=move || filter_chip_class(selected_type.get() == Some(SpaceType::Park))
                        aria-pressed=move || aria_pressed(selected_type.get() == Some(SpaceType::Park))
                        on:click=move |_| selected_type.set(Some(SpaceType::Park))
                    >
                        "Park"
                    </button>
                    <button
                        type="button"
                        class=move || filter_chip_class(selected_type.get() == Some(SpaceType::Transit))
                        aria-pressed=move || aria_pressed(selected_type.get() == Some(SpaceType::Transit))
                        on:click=move |_| selected_type.set(Some(SpaceType::Transit))
                    >
                        "Transit"
                    </button>
                    <button
                        type="button"
                        class=move || filter_chip_class(selected_type.get() == Some(SpaceType::Event))
                        aria-pressed=move || aria_pressed(selected_type.get() == Some(SpaceType::Event))
                        on:click=move |_| selected_type.set(Some(SpaceType::Event))
                    >
                        "Event"
                    </button>
                </div>
            </section>

            <section class="map-guide-card" aria-label="Map guide">
                <p class="eyebrow">"Live map"</p>
                <p>"Switch between flat map and 3D globe. Select a marker or open the space drawer."</p>
            </section>

            <div class="map-controls" aria-label="Map controls">
                <div class="map-style-switcher" aria-label="Map style">
                    <button
                        type="button"
                        class=move || map_style_class(map_style.get() == MapStyle::Road)
                        aria-label="Switch map to roadmap"
                        aria-pressed=move || aria_pressed(map_style.get() == MapStyle::Road)
                        on:click=move |_| {
                            map_style.set(MapStyle::Road);
                            instant_map_ui::set_style("map", MapStyle::Road);
                        }
                    >
                        "Road"
                    </button>
                    <button
                        type="button"
                        class=move || map_style_class(map_style.get() == MapStyle::Dark)
                        aria-label="Switch map to dark"
                        aria-pressed=move || aria_pressed(map_style.get() == MapStyle::Dark)
                        on:click=move |_| {
                            map_style.set(MapStyle::Dark);
                            instant_map_ui::set_style("map", MapStyle::Dark);
                        }
                    >
                        "Dark"
                    </button>
                </div>

                <div class="map-projection-switcher" aria-label="Map projection">
                    <button
                        type="button"
                        class=move || map_projection_class(map_projection.get() == MapProjection::Flat2d)
                        aria-label="Switch to 2D map"
                        aria-pressed=move || aria_pressed(map_projection.get() == MapProjection::Flat2d)
                        on:click=move |_| {
                            map_projection.set(MapProjection::Flat2d);
                            instant_map_ui::set_projection("map", MapProjection::Flat2d);
                        }
                    >
                        "2D Map"
                    </button>
                    <button
                        type="button"
                        class=move || map_projection_class(map_projection.get() == MapProjection::Globe3d)
                        aria-label="Switch to 3D globe"
                        aria-pressed=move || aria_pressed(map_projection.get() == MapProjection::Globe3d)
                        on:click=move |_| {
                            map_projection.set(MapProjection::Globe3d);
                            instant_map_ui::set_projection("map", MapProjection::Globe3d);
                        }
                    >
                        "3D Globe"
                    </button>
                </div>

                <div class="map-zoom-controls" aria-label="Map zoom">
                    <button
                        type="button"
                        aria-label="Zoom out"
                        on:click=move |_| instant_map_ui::zoom_out("map")
                    >
                        "-"
                    </button>
                    <button
                        type="button"
                        aria-label="Zoom in"
                        on:click=move |_| instant_map_ui::zoom_in("map")
                    >
                        "+"
                    </button>
                </div>
            </div>

            <details class="space-drawer" open=true>
                <summary>
                    <span>"Spaces"</span>
                    <Suspense fallback=move || view! { <span class="drawer-count">"..."</span> }>
                        {move || Suspend::new(async move {
                            let items = spaces.await;
                            view! { <span class="drawer-count">{items.len()}</span> }
                        })}
                    </Suspense>
                </summary>

                <Suspense fallback=move || view! { <SpaceListSkeleton /> }>
                    {move || Suspend::new(async move {
                        let items = spaces.await;
                        view! { <SpaceResults items=items selected_space=selected_space /> }
                    })}
                </Suspense>
            </details>

            {move || {
                if let Some(space) = selected_space.get() {
                    view! {
                        <SpaceDetailDrawer
                            space=space
                            on_close=Callback::new(move |_| selected_space.set(None))
                        />
                    }
                        .into_any()
                } else {
                    view! { <span class="space-detail-empty" aria-hidden="true"></span> }.into_any()
                }
            }}
        </section>
    }
}

#[component]
fn SpaceResults(
    items: Vec<SpaceMarker>,
    selected_space: RwSignal<Option<SpaceMarker>>,
) -> impl IntoView {
    let sync_items = items.clone();
    let list_items = items.clone();
    let on_select = Callback::new(move |space: SpaceMarker| {
        instant_map_ui::focus_point("map", space.lng, space.lat);
        selected_space.set(Some(space));
    });

    view! {
        <MapMarkerSync spaces=sync_items />
        {if list_items.is_empty() {
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
                        each=move || list_items.clone()
                        key=|space| space.id.clone()
                        children=move |space| {
                            let select = on_select;
                            view! { <SpaceCard space=space on_select=select /> }
                        }
                    />
                </ul>
            }
                .into_any()
        }}
    }
}

#[component]
fn MapMarkerSync(spaces: Vec<SpaceMarker>) -> impl IntoView {
    Effect::new(move |_| {
        if let Ok(points_json) = serde_json::to_string(&spaces) {
            instant_map_ui::sync_points("map", &points_json);
        }
    });
}

#[component]
fn SpaceCard(space: SpaceMarker, on_select: Callback<SpaceMarker>) -> impl IntoView {
    let is_public = space.is_public;
    let selectable_space = space.clone();
    let space_id = space.id.clone();
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
                    <button
                        class="space-card-action"
                        type="button"
                        data-space-select=space_id
                        on:click=move |_| on_select.run(selectable_space.clone())
                    >
                        {space.name_zh}
                    </button>
                    <p>{location}</p>
                </div>
                <div class="space-card-meta">
                    <span class="space-badge">{type_label}</span>
                    <span class=status_class>{status_label}</span>
                    <span class="space-count">{space.online_count}" online"</span>
                </div>
            </article>
        </li>
    }
}

#[component]
fn SpaceDetailDrawer(space: SpaceMarker, on_close: Callback<()>) -> impl IntoView {
    let is_public = space.is_public;
    let space_id = space.id.clone();
    let space_name = space.name_zh.clone();
    let location = location_label(&space);
    let type_label = space_type_label(&space.space_type);
    let status_label = if is_public {
        "Public space"
    } else {
        "Private space"
    };
    let status_class = if is_public {
        "space-badge space-badge-public"
    } else {
        "space-badge space-badge-private"
    };

    view! {
        <aside class="space-detail-drawer" aria-label="Space detail">
            <div class="space-detail-head">
                <div>
                    <p class="eyebrow">{type_label}</p>
                    <h2>{space.name_zh}</h2>
                </div>
                <button
                    type="button"
                    class="drawer-close"
                    aria-label="Close space detail"
                    on:click=move |_| on_close.run(())
                >
                    "x"
                </button>
            </div>

            <p class="space-detail-location">{location}</p>

            <div class="space-detail-stats">
                <span class=status_class>{status_label}</span>
                <span class="space-count">{space.online_count}" online"</span>
                <span class="space-coordinates">{format!("{:.5}, {:.5}", space.lat, space.lng)}</span>
            </div>

            <div class="space-detail-actions">
                <a class="button button-primary" href={format!("/spaces/{}", space.id)}>
                    "Open space"
                </a>
                <button
                    type="button"
                    class="button button-secondary"
                    on:click=move |_| instant_map_ui::focus_point("map", space.lng, space.lat)
                >
                    "Center map"
                </button>
            </div>

            {(!is_public).then(move || view! {
                <PrivateVerify
                    space_id=space_id.clone()
                    space_name=space_name.clone()
                />
            })}
        </aside>
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

fn aria_pressed(is_active: bool) -> &'static str {
    if is_active {
        "true"
    } else {
        "false"
    }
}

fn map_style_class(is_active: bool) -> &'static str {
    if is_active {
        "map-style-button is-active"
    } else {
        "map-style-button"
    }
}

fn map_projection_class(is_active: bool) -> &'static str {
    if is_active {
        "map-projection-button is-active"
    } else {
        "map-projection-button"
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
