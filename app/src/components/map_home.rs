use instant_domain::spaces::SpaceType;
use instant_map_ui::{MapProjection, MapStyle};
use leptos::prelude::*;
use leptos_router::hooks::use_location;

use crate::{
    app_state::use_app_refresh_state,
    components::{
        private_verify::PrivateVerify,
        space_form::{provide_create_space_modal, use_create_space_modal},
        space_share::SpaceSharePanel,
    },
    i18n::{localize_optional, localized_online_count, localized_space_count, t, use_i18n, Locale},
    server::geo::{list_geo_countries, resolve_place_center},
    server::guides::list_space_guides,
    server::spaces::{list_spaces, SpaceMarker},
};

#[component]
pub fn MapHome() -> impl IntoView {
    let locale = use_i18n().locale;
    let query = RwSignal::new(String::new());
    let selected_type = RwSignal::new(None::<SpaceType>);
    let selected_space = RwSignal::new(None::<SpaceMarker>);
    let refresh = use_app_refresh_state();
    let explorer_open = refresh.explorer_open;
    let hero_open = refresh.hero_open;
    // Shared destination scope (also shown in hamburger menu after confirm).
    let filter_country = refresh.dest_country;
    let filter_province = refresh.dest_province;
    let filter_city = refresh.dest_city;
    let dest_confirmed = refresh.dest_confirmed;
    let location = use_location();

    // URL is the source of truth for destination confirm (works even if WASM click fails).
    // /inspace?country=Japan  => close hero, load that country map
    // /inspace?map=1          => pure map
    // /inspace?explore=1      => open explorer
    // /inspace?home=1         => show hero (only if no country/map override)
    Effect::new(move |_| {
        let search = location.search.get();
        let path = location.pathname.get();
        let on_home = path == "/inspace" || path == "/" || path == "/inspace/";
        let on_map_route = path == "/map" || path == "/inspace/map";
        if !on_home && !on_map_route {
            return;
        }

        let country_q = query_param(&search, "country");
        let wants_map = search.contains("map=1") || search.contains("map=true");
        let wants_explore = search.contains("explore=1") || search.contains("explore=true");
        let wants_home = search.contains("home=1") || search.contains("home=true");

        if on_map_route {
            explorer_open.set(false);
            hero_open.set(false);
            instant_map_ui::reveal("map");
            return;
        }

        if let Some(country) = country_q {
            let mut country = country.trim().to_string();
            // Taiwan, Hong Kong and Macao are part of China, not countries.
            if country.eq_ignore_ascii_case("taiwan")
                || country == "台湾"
                || country == "台灣"
                || country.eq_ignore_ascii_case("hong kong")
                || country == "香港"
                || country.eq_ignore_ascii_case("macao")
                || country.eq_ignore_ascii_case("macau")
                || country == "澳门"
                || country == "澳門"
            {
                country = "China".to_string();
            }
            filter_country.set(country.clone());
            filter_province.set(String::new());
            filter_city.set(String::new());
            dest_confirmed.set(true);
            explorer_open.set(false);
            hero_open.set(false);
            crate::app_state::refresh_spaces();
            instant_map_ui::reveal("map");
            if let Some((lng, lat, zoom)) = country_center_fallback(&country) {
                instant_map_ui::focus_view("map", lng, lat, zoom);
            }
            // refine with geo API
            let country_for_map = country.clone();
            leptos::task::spawn_local(async move {
                if let Ok(Some(center)) =
                    resolve_place_center(nonempty(country_for_map.clone()), None, None).await
                {
                    instant_map_ui::focus_view("map", center.lng, center.lat, center.zoom);
                }
            });
            return;
        }

        if wants_map {
            explorer_open.set(false);
            hero_open.set(false);
            instant_map_ui::reveal("map");
            return;
        }

        if wants_explore {
            hero_open.set(false);
            explorer_open.set(true);
            instant_map_ui::reveal("map");
            return;
        }

        if wants_home {
            explorer_open.set(false);
            hero_open.set(true);
        }
    });

    // Scoped space load: filters empty => gradual global; set => only that region.
    let spaces = Resource::new(
        move || {
            (
                query.get(),
                selected_type.get(),
                refresh.spaces.get(),
                filter_country.get(),
                filter_province.get(),
                filter_city.get(),
                dest_confirmed.get(),
            )
        },
        |(q, space_type, _refresh, country, province, city, confirmed)| async move {
            // Before confirm: still allow global browse but prefer not blocking first paint.
            let country = if confirmed { country } else { String::new() };
            let province = if confirmed { province } else { String::new() };
            let city = if confirmed { city } else { String::new() };
            let value = if q.trim().is_empty() { None } else { Some(q) };
            list_spaces(
                value,
                space_type,
                nonempty(country),
                nonempty(province),
                nonempty(city),
            )
            .await
            .unwrap_or_default()
        },
    );

    Effect::new(move |_| {
        instant_map_ui::mount("map", MapStyle::Road, MapProjection::Flat2d);
    });
    // When intro closes, force map canvas resize (MapLibre blank-canvas fix).
    Effect::new(move |_| {
        if !hero_open.get() {
            instant_map_ui::reveal("map");
        }
    });
    on_cleanup(move || instant_map_ui::destroy("map"));

    Effect::new(move |_| {
        query.get();
        selected_type.get();
        filter_country.get();
        filter_province.get();
        filter_city.get();
        selected_space.set(None);
    });

    view! {
        <section class="map-layout">
            <div id="map" class="map-canvas" aria-label="inspace map">
                <div class="map-loading" aria-live="polite">
                    <span class="map-loading-dot" aria-hidden="true"></span>
                    <span>{move || t(locale.get(), "正在加载真实地图瓦片", "Loading real map tiles")}</span>
                </div>
            </div>

            <div class="map-vignette" aria-hidden="true"></div>

            {move || if hero_open.get() {
                view! {
                    <HomeHero
                        query=query
                        selected_type=selected_type
                        explorer_open=explorer_open
                        hero_open=hero_open
                        filter_country=filter_country
                        filter_province=filter_province
                        filter_city=filter_city
                        dest_confirmed=dest_confirmed
                    />
                }.into_any()
            } else {
                view! { <span class="home-hero-placeholder" aria-hidden="true"></span> }.into_any()
            }}

            {move || if explorer_open.get() {
                view! {
                    <section class="map-filter-panel" aria-label="Space filters">
                        <div class="map-filter-heading">
                            <div>
                                <p class="eyebrow">"inspace"</p>
                                <h1>{move || t(locale.get(), "探索旅行空间", "Explore travel spaces")}</h1>
                                {move || {
                                    if dest_confirmed.get() {
                                        let label = crate::app_state::destination_label(
                                            &filter_country.get(),
                                            &filter_province.get(),
                                            &filter_city.get(),
                                        );
                                        if label.is_empty() {
                                            view! { <span class="dest-scope-chip muted">{move || t(locale.get(), "全局", "Global")}</span> }.into_any()
                                        } else {
                                            view! { <span class="dest-scope-chip">{label}</span> }.into_any()
                                        }
                                    } else {
                                        view! { <span class="dest-scope-chip muted">{move || t(locale.get(), "未选定目的地（全局）", "No destination (global)")}</span> }.into_any()
                                    }
                                }}
                            </div>
                            <div class="panel-heading-actions">
                                <Suspense fallback=move || view! { <span class="result-count">{move || t(locale.get(), "加载中", "Loading")}</span> }>
                                    {move || Suspend::new(async move {
                                        let items = spaces.await;
                                        let count = items.len();
                                        view! { <span class="result-count">{move || localized_space_count(locale.get(), count)}</span> }
                                    })}
                                </Suspense>
                                <button
                                    type="button"
                                    class="panel-close"
                                    aria-label=move || t(locale.get(), "收起探索面板", "Close explorer panel")
                                    on:click=move |_| { explorer_open.set(false); }
                                >
                                    "×"
                                </button>
                            </div>
                        </div>

                        <label class="search-control">
                            <span>{move || t(locale.get(), "搜索空间", "Search spaces")}</span>
                            <input
                                type="search"
                                aria-label="Search spaces"
                                placeholder=move || t(locale.get(), "试试：外滩、茶室、公园...", "Try Bund, tea, park...")
                                prop:value=move || query.get()
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
                                {move || t(locale.get(), "全部", "All")}
                            </button>
                            <button
                                type="button"
                                class=move || filter_chip_class(selected_type.get() == Some(SpaceType::Scenic))
                                aria-pressed=move || aria_pressed(selected_type.get() == Some(SpaceType::Scenic))
                                on:click=move |_| selected_type.set(Some(SpaceType::Scenic))
                            >
                                {move || t(locale.get(), "景点", "Scenic")}
                            </button>
                            <button
                                type="button"
                                class=move || filter_chip_class(selected_type.get() == Some(SpaceType::Food))
                                aria-pressed=move || aria_pressed(selected_type.get() == Some(SpaceType::Food))
                                on:click=move |_| selected_type.set(Some(SpaceType::Food))
                            >
                                {move || t(locale.get(), "美食", "Food")}
                            </button>
                            <button
                                type="button"
                                class=move || filter_chip_class(selected_type.get() == Some(SpaceType::Park))
                                aria-pressed=move || aria_pressed(selected_type.get() == Some(SpaceType::Park))
                                on:click=move |_| selected_type.set(Some(SpaceType::Park))
                            >
                                {move || t(locale.get(), "公园", "Park")}
                            </button>
                            <button
                                type="button"
                                class=move || filter_chip_class(selected_type.get() == Some(SpaceType::Transit))
                                aria-pressed=move || aria_pressed(selected_type.get() == Some(SpaceType::Transit))
                                on:click=move |_| selected_type.set(Some(SpaceType::Transit))
                            >
                                {move || t(locale.get(), "交通", "Transit")}
                            </button>
                            <button
                                type="button"
                                class=move || filter_chip_class(selected_type.get() == Some(SpaceType::Event))
                                aria-pressed=move || aria_pressed(selected_type.get() == Some(SpaceType::Event))
                                on:click=move |_| selected_type.set(Some(SpaceType::Event))
                            >
                                {move || t(locale.get(), "活动", "Event")}
                            </button>
                        </div>

                        <section class="explorer-space-results" aria-label="Explore space results">
                            <div class="explorer-space-results-head">
                                <span>{move || t(locale.get(), "空间列表", "Spaces")}</span>
                                <Suspense fallback=move || view! { <span class="drawer-count">"..."</span> }>
                                    {move || Suspend::new(async move {
                                        let items = spaces.await;
                                        view! { <span class="drawer-count">{items.len()}</span> }
                                    })}
                                </Suspense>
                            </div>

                            <Suspense fallback=move || view! { <SpaceListSkeleton /> }>
                                {move || Suspend::new(async move {
                                    let items = spaces.await;
                                    view! { <SpaceResults items=items selected_space=selected_space /> }
                                })}
                            </Suspense>
                        </section>
                    </section>
                }.into_any()
            } else {
                view! { <span class="explorer-reopen-placeholder" aria-hidden="true"></span> }.into_any()
            }}

            {move || if explorer_open.get() {
                view! {
                    <section class="map-guide-card" aria-label="Map guide">
                        <p class="eyebrow">{move || t(locale.get(), "旅行地图", "Travel map")}</p>
                        <p>{move || t(locale.get(), "点选地图标记或列表中的空间，查看攻略与详情。", "Select a map marker or list item to view guides and details.")}</p>
                    </section>
                }.into_any()
            } else {
                view! { <span class="map-guide-card-placeholder" aria-hidden="true"></span> }.into_any()
            }}

            <Suspense fallback=|| ()>
                {move || Suspend::new(async move {
                    let items = spaces.await;
                    view! { <MapMarkerSync spaces=items selected_space=selected_space /> }
                })}
            </Suspense>

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
fn HomeHero(
    query: RwSignal<String>,
    selected_type: RwSignal<Option<SpaceType>>,
    explorer_open: RwSignal<bool>,
    hero_open: RwSignal<bool>,
    filter_country: RwSignal<String>,
    filter_province: RwSignal<String>,
    filter_city: RwSignal<String>,
    dest_confirmed: RwSignal<bool>,
) -> impl IntoView {
    let locale = use_i18n().locale;
    let create_modal = use_create_space_modal().unwrap_or_else(provide_create_space_modal);
    let countries = RwSignal::new(fallback_countries());
    let countries_ready = RwSignal::new(false);

    Effect::new(move |_| {
        leptos::task::spawn_local(async move {
            match list_geo_countries().await {
                Ok(items) if !items.is_empty() => {
                    countries.set(items);
                    countries_ready.set(true);
                }
                Ok(_) | Err(_) => {
                    countries_ready.set(true);
                }
            }
        });
    });

    let open_explorer = move || {
        hero_open.set(false);
        explorer_open.set(true);
        instant_map_ui::reveal("map");
    };

    view! {
        <section class="home-hero-card home-hero-card--vision" aria-labelledby="home-hero-title">
            <button
                type="button"
                class="home-hero-close"
                aria-label=move || t(locale.get(), "关闭首页，回到地图", "Close home and show map")
                on:click=move |_| {
                    hero_open.set(false);
                    explorer_open.set(false);
                    instant_map_ui::reveal("map");
                }
            >
                "×"
            </button>

            <div class="home-hero-content">
                <div class="home-hero-kicker">
                    <span class="home-hero-live">
                        <i aria-hidden="true"></i>
                        {move || t(locale.get(), "全球旅行攻略与共享体验空间平台", "Global travel guide and shared experience spaces")}
                    </span>
                </div>

                <h1 id="home-hero-title">
                    {move || t(
                        locale.get(),
                        "发现全球旅行攻略空间，扫码进入真实地点的共享体验。",
                        "Discover travel guide spaces and scan into shared experiences at real places.",
                    )}
                </h1>
                <p class="home-hero-copy">
                    {move || t(
                        locale.get(),
                        "每一个真实旅行地点，都可以有一个可通过地图、链接或二维码进入的数字攻略空间。先找地点，再进入 Space 看攻略、社群与现场提醒。",
                        "Every real travel place can have a digital guide space reached from a map, link, or QR code. Find the place first, then enter its Space for guides, community, and on-site tips.",
                    )}
                </p>

                <form
                    class="home-hero-search"
                    role="search"
                    aria-label=move || t(locale.get(), "搜索旅行空间", "Search travel spaces")
                    on:submit=move |ev| {
                        ev.prevent_default();
                        open_explorer();
                    }
                >
                    <span class="home-hero-search-icon" aria-hidden="true">"⌕"</span>
                    <input
                        type="search"
                        aria-label=move || t(locale.get(), "搜索旅行空间", "Search travel spaces")
                        placeholder=move || t(locale.get(), "搜真实地点：外滩、故宫、京都咖啡馆、夜市", "Search real places: Bund, Palace Museum, Kyoto cafe, night market")
                        prop:value=move || query.get()
                        on:input=move |ev| query.set(event_target_value(&ev))
                    />
                    <button type="submit" class="button button-primary">
                        {move || t(locale.get(), "搜索空间", "Search spaces")}
                    </button>
                </form>

                <div class="home-hero-proof" aria-label=move || t(locale.get(), "产品基线", "Product baseline")>
                    <span class="proof-chip proof-a">{move || t(locale.get(), "地图默认显示 Space", "Map shows Spaces first")}</span>
                    <span class="proof-chip proof-b">{move || t(locale.get(), "Guide 是空间下的攻略资产", "Guides live under Spaces")}</span>
                    <span class="proof-chip proof-c">{move || t(locale.get(), "QR / Link 直达空间详情", "QR / Link opens Space detail")}</span>
                    <span class="proof-chip proof-d">{move || t(locale.get(), "未来接入 3D 共享体验", "3D shared experience next")}</span>
                </div>

                <div class="home-hero-intents" aria-label=move || t(locale.get(), "快速入口", "Quick entries")>
                    <button type="button" class="intent-scenic" on:click=move |_| {
                        selected_type.set(Some(SpaceType::Scenic));
                        open_explorer();
                    }>
                        <span class="intent-icon intent-icon-scenic" aria-hidden="true"></span>
                        <span class="intent-text"><strong>{move || t(locale.get(), "景点空间", "Scenic Spaces")}</strong><span>{move || t(locale.get(), "路线、拍照点、避坑", "Routes, photo spots, tips")}</span></span>
                    </button>
                    <button type="button" class="intent-food" on:click=move |_| {
                        selected_type.set(Some(SpaceType::Food));
                        open_explorer();
                    }>
                        <span class="intent-icon intent-icon-food" aria-hidden="true"></span>
                        <span class="intent-text"><strong>{move || t(locale.get(), "美食空间", "Food Spaces")}</strong><span>{move || t(locale.get(), "餐馆、夜市、咖啡馆", "Restaurants, markets, cafes")}</span></span>
                    </button>
                    <button type="button" class="intent-city" on:click=move |_| {
                        selected_type.set(None);
                        open_explorer();
                    }>
                        <span class="intent-icon intent-icon-city" aria-hidden="true"></span>
                        <span class="intent-text"><strong>{move || t(locale.get(), "城市/街区", "Cities / districts")}</strong><span>{move || t(locale.get(), "从地图进入真实地点", "Enter real places from map")}</span></span>
                    </button>
                </div>

                <div class="home-hero-steps" aria-label=move || t(locale.get(), "使用流程", "How it works")>
                    <article class="home-hero-step step-a">
                        <strong>"1"</strong>
                        <div><b>{move || t(locale.get(), "找真实地点", "Find a real place")}</b><span>{move || t(locale.get(), "地图点位代表 Space，不是普通文章。", "Map markers represent Spaces, not generic posts.")}</span></div>
                    </article>
                    <article class="home-hero-step step-b">
                        <strong>"2"</strong>
                        <div><b>{move || t(locale.get(), "读空间攻略", "Read Space guides")}</b><span>{move || t(locale.get(), "路线、美食、交通、Tips 归在同一空间。", "Routes, food, transit, and tips stay under one Space.")}</span></div>
                    </article>
                    <article class="home-hero-step step-c">
                        <strong>"3"</strong>
                        <div><b>{move || t(locale.get(), "分享入口", "Share the entry")}</b><span>{move || t(locale.get(), "二维码或链接发给朋友，扫码进入空间详情。", "Send a QR/link so friends land on the Space detail.")}</span></div>
                    </article>
                </div>

                <section class="home-destination-guide" aria-label=move || t(locale.get(), "你想去哪", "Where do you want to go")>
                    <div class="home-destination-head">
                        <strong>{move || t(locale.get(), "你想去哪？", "Where do you want to go?")}</strong>
                        <p>{move || t(locale.get(), "先选国家聚焦地图；也可以直接浏览全球 Space。", "Pick a country to focus the map, or browse global Spaces directly.")}</p>
                    </div>
                    <form
                        class="home-destination-form"
                        method="get"
                        action="/inspace"
                        on:submit=move |ev| {
                            let country = filter_country.get().trim().to_string();
                            filter_province.set(String::new());
                            filter_city.set(String::new());
                            dest_confirmed.set(true);
                            explorer_open.set(false);
                            hero_open.set(false);
                            crate::app_state::refresh_spaces();
                            instant_map_ui::reveal("map");
                            if let Some((lng, lat, zoom)) = country_center_fallback(&country) {
                                instant_map_ui::focus_view("map", lng, lat, zoom);
                            } else if country.is_empty() {
                                instant_map_ui::focus_view("map", 20.0, 20.0, 1.8);
                            }
                            let country_for_map = country.clone();
                            leptos::task::spawn_local(async move {
                                if let Ok(Some(center)) =
                                    resolve_place_center(nonempty(country_for_map), None, None).await
                                {
                                    instant_map_ui::focus_view("map", center.lng, center.lat, center.zoom);
                                }
                            });
                            let _ = ev;
                        }
                    >
                        <div class="home-destination-grid home-destination-grid-country-only">
                            <label class="field-label">
                                <span>{move || t(locale.get(), "国家", "Country")}</span>
                                <select
                                    name="country"
                                    aria-label=move || t(locale.get(), "选择国家", "Select country")
                                    prop:value=move || filter_country.get()
                                    on:change=move |ev| {
                                        filter_country.set(event_target_value(&ev));
                                        filter_province.set(String::new());
                                        filter_city.set(String::new());
                                        dest_confirmed.set(false);
                                    }
                                >
                                    <option value="">{move || t(locale.get(), "不限，浏览全球", "Any, browse global")}</option>
                                    {move || countries
                                        .get()
                                        .into_iter()
                                        .map(|item| {
                                            let value = item.value.clone();
                                            let label = item.label.clone();
                                            let selected = filter_country.get() == value;
                                            view! { <option value=value selected=selected>{label}</option> }
                                        })
                                        .collect_view()
                                    }
                                </select>
                            </label>
                        </div>
                        <div class="home-destination-actions">
                            <button type="submit" class="button button-primary" id="confirm-destination-btn">
                                {move || t(locale.get(), "确认并看地图", "Confirm & open map")}
                            </button>
                            <a class="button button-secondary" href="/inspace?map=1">
                                {move || t(locale.get(), "直接浏览全球", "Browse world")}
                            </a>
                        </div>
                    </form>
                    {move || {
                        let count = countries.get().len();
                        if countries_ready.get() {
                            view! { <p class="home-destination-status">{format!("{} {}", count, t(locale.get(), "个国家可选", "countries available"))}</p> }.into_any()
                        } else {
                            view! { <p class="home-destination-status">{move || t(locale.get(), "正在同步国家列表…", "Syncing country list…")}</p> }.into_any()
                        }
                    }}
                </section>

                <div class="home-hero-actions" aria-label=move || t(locale.get(), "首屏操作", "Hero actions")>
                    <button type="button" class="button button-secondary" on:click=move |_| {
                        hero_open.set(false);
                        explorer_open.set(false);
                        instant_map_ui::reveal("map");
                    }>
                        {move || t(locale.get(), "看地图", "Open map")}
                    </button>
                    <a class="button button-secondary" href="/inspace/guides">
                        {move || t(locale.get(), "浏览攻略", "Browse guides")}
                    </a>
                    <button
                        type="button"
                        class="button button-ghost-on-map"
                        on:click=move |_| create_modal.open.set(true)
                    >
                        {move || t(locale.get(), "创建旅行空间", "Create travel Space")}
                    </button>
                </div>
            </div>

            <div class="home-hero-visual product-vision-visual" aria-label=move || t(locale.get(), "Space Guide QR 示例", "Space Guide QR example")>
                <div class="vision-map-card">
                    <span class="vision-map-grid" aria-hidden="true"></span>
                    <span class="vision-pin vision-pin-a" aria-hidden="true"></span>
                    <span class="vision-pin vision-pin-b" aria-hidden="true"></span>
                    <span class="vision-pin vision-pin-c" aria-hidden="true"></span>
                    <span class="vision-map-label">{move || t(locale.get(), "地图发现 Space", "Map discovers Spaces")}</span>
                </div>
                <article class="vision-space-card">
                    <small>"SPACE"</small>
                    <strong>{move || t(locale.get(), "外滩数字攻略空间", "Bund digital guide Space")}</strong>
                    <p>{move || t(locale.get(), "真实地点入口 · 攻略 · 社群 · 二维码", "Real-place entry · guides · community · QR")}</p>
                </article>
                <article class="vision-guide-card">
                    <small>"GUIDE"</small>
                    <strong>{move || t(locale.get(), "半日路线 / 拍照机位 / 夜景避坑", "Half-day route / photo spots / night-view tips")}</strong>
                </article>
                <div class="vision-qr-card" aria-hidden="true">
                    <span><i></i><i></i><i></i><i></i><i></i><i></i><i></i><i></i><i></i></span>
                    <b>{move || t(locale.get(), "扫码进入 Space", "Scan into Space")}</b>
                </div>
            </div>
        </section>
    }
}

#[component]
fn SpaceResults(
    items: Vec<SpaceMarker>,
    selected_space: RwSignal<Option<SpaceMarker>>,
) -> impl IntoView {
    let list_items = items.clone();
    let on_select = Callback::new(move |space: SpaceMarker| {
        instant_map_ui::focus_point("map", space.lng, space.lat);
        selected_space.set(Some(space));
    });

    view! {
        {if list_items.is_empty() {
            let locale = use_i18n().locale;
            view! {
                <div class="empty-state">
                    <strong>{move || t(locale.get(), "没有找到空间", "No spaces found")}</strong>
                    <span>{move || t(locale.get(), "请尝试其他关键词或清除筛选。", "Try another keyword or clear the filter.")}</span>
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
pub fn MapMarkerSync(
    spaces: Vec<SpaceMarker>,
    selected_space: RwSignal<Option<SpaceMarker>>,
) -> impl IntoView {
    let sync_spaces = spaces.clone();
    Effect::new(move |_| {
        if let Ok(points_json) = serde_json::to_string(&sync_spaces) {
            // maplibre_shim keeps pending points if the map is not ready yet,
            // and re-applies after mount / style load.
            instant_map_ui::sync_points("map", &points_json);
        }
    });

    // Hidden proxy buttons: map markers dispatch a click to `[data-space-select="{id}"]`.
    // These must exist regardless of whether the explorer panel is open, so a marker
    // tap opens the detail drawer even in the pure-map view.
    view! {
        <div class="space-open-proxies" aria-hidden="true" style="display:none">
            <For
                each=move || spaces.clone()
                key=|space| space.id.clone()
                children=move |space| {
                    let for_click = space.clone();
                    view! {
                        <button
                            type="button"
                            data-space-select=space.id.clone()
                            on:click=move |_| {
                                instant_map_ui::focus_point("map", for_click.lng, for_click.lat);
                                selected_space.set(Some(for_click.clone()));
                            }
                        />
                    }
                }
            />
        </div>
    }
}

#[component]
fn SpaceCard(space: SpaceMarker, on_select: Callback<SpaceMarker>) -> impl IntoView {
    let locale = use_i18n().locale;
    let is_public = space.is_public;
    let selectable_space = space.clone();
    let space_id = space.id.clone();
    let location = location_label(&space);
    let space_type = space.space_type.clone();
    let name_zh = space.name_zh.clone();
    let name_en = space.name_en.clone();
    let online_count = space.online_count;
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
                        {move || localize_optional(locale.get(), &name_zh, name_en.as_deref())}
                    </button>
                    <p>{location}</p>
                </div>
                <div class="space-card-meta">
                    <span class="space-badge">{move || space_type_label(locale.get(), &space_type)}</span>
                    <span class=status_class>{move || public_status_label(locale.get(), is_public)}</span>
                    <span class="space-count">{move || localized_online_count(locale.get(), online_count)}</span>
                </div>
            </article>
        </li>
    }
}

#[component]
pub fn SpaceDetailDrawer(space: SpaceMarker, on_close: Callback<()>) -> impl IntoView {
    let locale = use_i18n().locale;
    let is_public = space.is_public;
    let space_id = space.id.clone();
    let detail_name_zh = space.name_zh.clone();
    let private_space_name = detail_name_zh.clone();
    let community_space_name = detail_name_zh.clone();
    let share_name_zh = detail_name_zh.clone();
    let detail_name_en = space.name_en.clone();
    let location = location_label(&space);
    let space_type = space.space_type.clone();
    let online_count = space.online_count;
    let lat = space.lat;
    let lng = space.lng;
    let space_id_for_href = space.id.clone();
    let status_class = if is_public {
        "space-badge space-badge-public"
    } else {
        "space-badge space-badge-private"
    };

    view! {
        <aside class="space-detail-drawer" aria-label="Space detail">
            <div class="space-detail-head">
                <div>
                    <p class="eyebrow">{move || space_type_label(locale.get(), &space_type)}</p>
                    <h2>{move || localize_optional(locale.get(), &detail_name_zh, detail_name_en.as_deref())}</h2>
                </div>
                <button
                    type="button"
                    class="drawer-close"
                    aria-label="Close space detail"
                    on:click=move |_| on_close.run(())
                >
                    "×"
                </button>
            </div>

            <p class="space-detail-location">{location}</p>

            <div class="space-detail-stats">
                <span class=status_class>{move || public_space_status_label(locale.get(), is_public)}</span>
                <span class="space-count">{move || localized_online_count(locale.get(), online_count)}</span>
                <span class="space-coordinates">{format!("{:.5}, {:.5}", lat, lng)}</span>
            </div>

            <div class="space-detail-actions">
                <a class="button button-primary" href={format!("/inspace/spaces/{}", space_id_for_href)}>
                    {move || t(locale.get(), "打开空间", "Open space")}
                </a>
                <a class="button button-secondary" href={format!("/inspace/guides/new?space_id={}", space_id_for_href)}>
                    {move || t(locale.get(), "写攻略", "Write guide")}
                </a>
                <button
                    type="button"
                    class="button button-secondary"
                    on:click=move |_| instant_map_ui::focus_point("map", lng, lat)
                >
                    {move || t(locale.get(), "居中地图", "Center map")}
                </button>
            </div>

            <SpaceDetailGuides space_id=space_id_for_href.clone() />

            <SpaceSharePanel
                space_id=space_id_for_href.clone()
                space_name=share_name_zh
                compact=true
            />

            <aside class="community-links community-links-compact" aria-label=move || t(locale.get(), "空间社群链接", "Space community links")>
                <div class="community-link-actions">
                    <a class="button button-secondary" href="https://discord.gg/zsmYWvXyy" target="_blank" rel="noreferrer">"Discord"</a>
                    <a class="button button-secondary" href="https://pd.qq.com/s/8ru51ih0m?b=9" target="_blank" rel="noreferrer">"QQ 频道"</a>
                </div>
                <p class="community-links-note">
                    {move || format!(
                        "{}「{}」{}",
                        t(locale.get(), "在社群内搜索空间名", "Search the community for space"),
                        community_space_name,
                        t(locale.get(), "获取进入密码", "to get the access password")
                    )}
                </p>
            </aside>

            {(!is_public).then(move || view! {
                <PrivateVerify
                    space_id=space_id.clone()
                    space_name=private_space_name.clone()
                />
            })}
        </aside>
    }
}

#[component]
fn SpaceDetailGuides(space_id: String) -> impl IntoView {
    let locale = use_i18n().locale;
    let guides = Resource::new(
        move || space_id.clone(),
        |space_id| async move { list_space_guides(space_id).await.unwrap_or_default() },
    );

    view! {
        <section class="space-detail-guides" aria-label=move || t(locale.get(), "相关攻略", "Related guides")>
            <div class="space-detail-guides-head">
                <strong>{move || t(locale.get(), "相关攻略", "Related guides")}</strong>
            </div>
            <Suspense fallback=move || view! { <p class="muted">{move || t(locale.get(), "加载攻略…", "Loading guides…")}</p> }>
                {move || Suspend::new(async move {
                    let items = guides.await;
                    if items.is_empty() {
                        view! {
                            <p class="muted">{move || t(locale.get(), "暂无已发布攻略，可点击「写攻略」创建第一篇。", "No published guides yet. Use Write guide to create one.")}</p>
                        }.into_any()
                    } else {
                        view! {
                            <ul class="space-detail-guide-list">
                                <For
                                    each=move || items.clone()
                                    key=|guide| guide.id
                                    children=move |guide| {
                                        let title_zh = guide.title_zh.clone();
                                        let title_en = guide.title_en.clone();
                                        let href = format!("/inspace/guides/{}", guide.id);
                                        view! {
                                            <li>
                                                <a href=href>
                                                    {move || localize_optional(locale.get(), &title_zh, title_en.as_deref())}
                                                </a>
                                            </li>
                                        }
                                    }
                                />
                            </ul>
                        }.into_any()
                    }
                })}
            </Suspense>
        </section>
    }
}

#[component]
pub fn SpaceListSkeleton() -> impl IntoView {
    let locale = use_i18n().locale;
    view! {
        <div class="space-list-skeleton" aria-label=move || t(locale.get(), "正在加载空间", "Loading spaces")>
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

fn space_type_label(locale: Locale, space_type: &SpaceType) -> &'static str {
    match space_type {
        SpaceType::Scenic => t(locale, "景点", "Scenic"),
        SpaceType::Food => t(locale, "美食", "Food"),
        SpaceType::Park => t(locale, "公园", "Park"),
        SpaceType::Transit => t(locale, "交通", "Transit"),
        SpaceType::Event => t(locale, "活动", "Event"),
        SpaceType::Custom => t(locale, "自定义", "Custom"),
    }
}

fn public_status_label(locale: Locale, is_public: bool) -> &'static str {
    if is_public {
        t(locale, "公开", "Public")
    } else {
        t(locale, "私密", "Private")
    }
}

fn public_space_status_label(locale: Locale, is_public: bool) -> &'static str {
    if is_public {
        t(locale, "公开空间", "Public space")
    } else {
        t(locale, "私密空间", "Private space")
    }
}

fn location_label(space: &SpaceMarker) -> String {
    let parts = [
        space.country.as_deref(),
        space.province.as_deref(),
        space.city.as_deref(),
        space.district.as_deref(),
        space.spot_name.as_deref(),
        space.address_line.as_deref(),
    ]
    .into_iter()
    .flatten()
    .filter(|value| !value.trim().is_empty())
    .collect::<Vec<_>>();
    if parts.is_empty() {
        space
            .province
            .clone()
            .unwrap_or_else(|| "Location pending".to_string())
    } else {
        parts.join(" / ")
    }
}

fn query_param(search: &str, key: &str) -> Option<String> {
    let raw = search.trim_start_matches('?');
    for pair in raw.split('&') {
        if pair.is_empty() {
            continue;
        }
        let mut parts = pair.splitn(2, '=');
        let k = parts.next().unwrap_or("");
        let v = parts.next().unwrap_or("");
        if k == key {
            return Some(urlencoding_decode(v));
        }
    }
    None
}

fn urlencoding_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                let hex = |c: u8| -> Option<u8> {
                    match c {
                        b'0'..=b'9' => Some(c - b'0'),
                        b'a'..=b'f' => Some(c - b'a' + 10),
                        b'A'..=b'F' => Some(c - b'A' + 10),
                        _ => None,
                    }
                };
                if let (Some(a), Some(b)) = (hex(bytes[i + 1]), hex(bytes[i + 2])) {
                    out.push((a << 4) | b);
                    i += 3;
                } else {
                    out.push(bytes[i]);
                    i += 1;
                }
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn country_center_fallback(country: &str) -> Option<(f64, f64, f64)> {
    // (lng, lat, zoom) — national CAPITAL coords for instant fly (API refines via PPLC).
    let key = country.trim().to_ascii_lowercase();
    let hit = match key.as_str() {
        "" => None,
        // Beijing
        "china" | "中国" => Some((116.39723, 39.9075, 5.8)),
        // Tokyo
        "japan" | "日本" => Some((139.69171, 35.6895, 5.8)),
        // Seoul
        "south korea" | "korea" | "韩国" | "韓國" => Some((126.978, 37.5665, 6.0)),
        // Washington, D.C.
        "united states" | "usa" | "us" | "美国" => Some((-77.0369, 38.9072, 5.5)),
        // Bangkok
        "thailand" | "泰国" => Some((100.5018, 13.7563, 6.0)),
        // Singapore
        "singapore" | "新加坡" => Some((103.8198, 1.3521, 10.5)),
        // London
        "united kingdom" | "uk" | "英国" | "great britain" | "britain" => {
            Some((-0.12574, 51.50853, 6.0))
        }
        // Paris
        "france" | "法国" => Some((2.3522, 48.8566, 6.0)),
        // Berlin
        "germany" | "德国" => Some((13.405, 52.52, 6.0)),
        // Rome
        "italy" | "意大利" => Some((12.4964, 41.9028, 6.0)),
        // Madrid
        "spain" | "西班牙" => Some((-3.7038, 40.4168, 6.0)),
        // Canberra
        "australia" | "澳大利亚" => Some((149.12807, -35.28346, 5.5)),
        // Ottawa
        "canada" | "加拿大" => Some((-75.6972, 45.4215, 5.5)),
        // New Delhi
        "india" | "印度" => Some((77.209, 28.6139, 5.8)),
        // Jakarta
        "indonesia" | "印度尼西亚" => Some((106.8456, -6.2088, 5.8)),
        // Kuala Lumpur
        "malaysia" | "马来西亚" => Some((101.6869, 3.139, 6.2)),
        // Hanoi
        "vietnam" | "越南" => Some((105.8342, 21.0278, 6.0)),
        // Moscow
        "russia" | "俄罗斯" => Some((37.6173, 55.7558, 5.2)),
        // Brasília
        "brazil" | "巴西" => Some((-47.92972, -15.77972, 5.5)),
        // Mexico City
        "mexico" | "墨西哥" => Some((-99.1332, 19.4326, 5.8)),
        _ => None,
    };
    hit
}

fn fallback_countries() -> Vec<instant_domain::locations::GeoOption> {
    // Offline/SSR-safe starter list if geo API is slow/fails.
    [
        ("China", "China / 中国"),
        ("United States", "United States / 美国"),
        ("Japan", "Japan / 日本"),
        ("South Korea", "South Korea / 韩国"),
        ("Thailand", "Thailand / 泰国"),
        ("Singapore", "Singapore / 新加坡"),
        ("United Kingdom", "United Kingdom / 英国"),
        ("France", "France / 法国"),
        ("Germany", "Germany / 德国"),
        ("Italy", "Italy / 意大利"),
        ("Spain", "Spain / 西班牙"),
        ("Australia", "Australia / 澳大利亚"),
        ("Canada", "Canada / 加拿大"),
        ("India", "India / 印度"),
        ("Indonesia", "Indonesia / 印度尼西亚"),
        ("Malaysia", "Malaysia / 马来西亚"),
        ("Vietnam", "Vietnam / 越南"),
        ("Russia", "Russia / 俄罗斯"),
        ("Brazil", "Brazil / 巴西"),
        ("Mexico", "Mexico / 墨西哥"),
    ]
    .into_iter()
    .map(|(value, label)| instant_domain::locations::GeoOption {
        value: value.to_string(),
        label: label.to_string(),
    })
    .collect()
}

fn nonempty(value: String) -> Option<String> {
    let trimmed = value.trim().to_string();
    (!trimmed.is_empty()).then_some(trimmed)
}
