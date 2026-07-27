use leptos::prelude::*;

use crate::{
    app_state::use_app_refresh_state,
    components::map_home::{MapMarkerSync, SpaceDetailDrawer},
    i18n::{t, use_i18n},
    server::spaces::{list_spaces, SpaceMarker},
};

/// Dedicated map workspace. This is the only component that creates MapLibre.
#[component]
pub fn MapWorkspace() -> impl IntoView {
    let locale = use_i18n().locale;
    let selected_space = RwSignal::new(None::<SpaceMarker>);
    let refresh = use_app_refresh_state();

    // Spaces created via the map picker must show up as markers here.
    let spaces = Resource::new(
        move || {
            (
                refresh.spaces.get(),
                refresh.dest_country.get(),
                refresh.dest_province.get(),
                refresh.dest_city.get(),
                refresh.dest_confirmed.get(),
            )
        },
        |(_refresh, country, province, city, confirmed)| async move {
            let (country, province, city) = if confirmed {
                (country, province, city)
            } else {
                (String::new(), String::new(), String::new())
            };
            list_spaces(
                None,
                None,
                nonempty(country),
                nonempty(province),
                nonempty(city),
            )
            .await
            .unwrap_or_default()
        },
    );

    Effect::new(move |_| {
        instant_map_ui::mount("map", refresh.map_style.get(), refresh.map_projection.get());
    });
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::{closure::Closure, JsCast};
        let retry = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
            instant_map_ui::mount(
                "map",
                refresh.map_style.get_untracked(),
                refresh.map_projection.get_untracked(),
            )
        });
        let _ = web_sys::window().unwrap().add_event_listener_with_callback(
            "instant-space-hydrated",
            retry.as_ref().unchecked_ref(),
        );
        retry.forget();
    }
    on_cleanup(move || instant_map_ui::destroy("map"));
    view! {
        <main id="main-content" class="map-workspace">
            <div id="map" class="map-canvas" aria-label=move || t(locale.get(), "地图探索工作区", "Map exploration workspace")>
                <div class="map-loading" aria-live="polite"><span class="map-loading-dot" aria-hidden="true"></span><span>{move || t(locale.get(), "正在加载地图…", "Loading map…")}</span></div>
            </div>
            <section class="map-workspace-bar" aria-label=move || t(locale.get(), "地图状态", "Map status")>
                <div>
                    <p class="eyebrow">"inspace"</p>
                    <h1>{move || t(locale.get(), "地图探索", "Map exploration")}</h1>
                    <Suspense fallback=|| ()>
                        {move || Suspend::new(async move {
                            let count = spaces.await.len();
                            view! {
                                <p class="map-workspace-count">
                                    {move || format!("{} {}", count, t(locale.get(), "个空间在地图上", "spaces on the map"))}
                                </p>
                            }
                        })}
                    </Suspense>
                </div>
            </section>

            <Suspense fallback=|| ()>
                {move || Suspend::new(async move {
                    let items = spaces.await;
                    view! { <MapMarkerSync spaces=items selected_space=selected_space /> }
                })}
            </Suspense>

            {move || selected_space.get().map(|space| view! {
                <SpaceDetailDrawer
                    space=space
                    on_close=Callback::new(move |_| selected_space.set(None))
                />
            })}
        </main>
    }
}

fn nonempty(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}
