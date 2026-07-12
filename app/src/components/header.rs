use instant_domain::auth::CurrentUser;
use instant_map_ui::{MapProjection, MapStyle};
use leptos::prelude::*;
use leptos_router::hooks::use_location;

use crate::{
    app_state::{
        clear_destination, destination_label, open_explorer, open_hero, refresh_session,
        use_app_refresh_state,
    },
    components::space_form::OpenCreateSpaceButton,
    i18n::{t, use_i18n, Locale},
    server::auth::{current_session, logout_user},
};

#[component]
pub fn Header() -> impl IntoView {
    let locale = use_i18n().locale;
    let refresh = use_app_refresh_state();
    let location = use_location();
    let pathname = location.pathname;
    let map_style = RwSignal::new(MapStyle::Road);
    let map_projection = RwSignal::new(MapProjection::Flat2d);
    let session = Resource::new(
        move || refresh.session.get(),
        |_| async move { current_session().await.ok().flatten() },
    );

    view! {
        <header class="topbar" aria-label="Instant Space navigation">
            <a href="/inspace" class="brand" aria-label="Instant Space home">
                <span class="brand-mark" aria-hidden="true"></span>
                <span>"Instant Space"</span>
            </a>
            <nav class="primary-nav" aria-label="Primary">
                <a
                    href="/inspace?home=1"
                    class=move || nav_class(&pathname.get(), &location.search.get(), "home")
                    aria-label=move || t(locale.get(), "首页介绍", "Home intro")
                    on:click=move |_| {
                        open_hero();
                    }
                >
                    {move || t(locale.get(), "首页", "Home")}
                </a>
                <a
                    href="/inspace?explore=1"
                    class=move || nav_class(&pathname.get(), &location.search.get(), "explore")
                    aria-label=move || t(locale.get(), "探索实时空间", "Explore live spaces")
                    on:click=move |_| {
                        open_explorer();
                    }
                >
                    {move || t(locale.get(), "探索", "Explore")}
                </a>
                <details class="nav-menu">
                    <summary
                        class="nav-menu-trigger"
                        aria-label=move || t(locale.get(), "打开导航菜单", "Open navigation menu")
                    >
                        <span class="hamburger-lines" aria-hidden="true">
                            <span></span>
                            <span></span>
                            <span></span>
                        </span>
                    </summary>
                    <div class="nav-menu-panel">
                        <Suspense fallback=move || view! { <a href="/inspace/login" class="nav-menu-item nav-menu-muted">{move || t(locale.get(), "登录", "Sign in")}</a> }>
                            {move || Suspend::new(async move {
                                match session.await {
                                    Some(user) => view! { <UserMenu user=user /> }.into_any(),
                                    None => view! { <a href="/inspace/login" class="nav-menu-item nav-menu-muted">{move || t(locale.get(), "登录", "Sign in")}</a> }.into_any(),
                                }
                            })}
                        </Suspense>
                        <div class="nav-menu-divider" aria-hidden="true"></div>
                        <a
                            href="/inspace/my-spaces"
                            class=move || nav_menu_class(&pathname.get(), "my-spaces")
                        >
                            {move || t(locale.get(), "我的空间", "My Spaces")}
                        </a>
                        <a href="/inspace/guides" class=move || nav_menu_class(&pathname.get(), "guides")>{move || t(locale.get(), "攻略", "Guides")}</a>
                        <OpenCreateSpaceButton class="nav-menu-item nav-menu-primary" />
                        <div class="nav-menu-divider" aria-hidden="true"></div>
                        <div class="nav-menu-section dest-menu-section">
                            <span class="nav-menu-label">{move || t(locale.get(), "目的地", "Destination")}</span>
                            {move || {
                                let st = use_app_refresh_state();
                                let confirmed = st.dest_confirmed.get();
                                let label = destination_label(
                                    &st.dest_country.get(),
                                    &st.dest_province.get(),
                                    &st.dest_city.get(),
                                );
                                if confirmed && !label.is_empty() {
                                    view! {
                                        <div class="dest-menu-active">
                                            <span class="dest-menu-chip">{label}</span>
                                            <button
                                                type="button"
                                                class="nav-menu-item"
                                                on:click=move |_| {
                                                    open_hero();
                                                }
                                            >
                                                {move || t(locale.get(), "修改目的地", "Change destination")}
                                            </button>
                                            <button
                                                type="button"
                                                class="nav-menu-item"
                                                on:click=move |_| {
                                                    clear_destination();
                                                    open_explorer();
                                                }
                                            >
                                                {move || t(locale.get(), "清除并全局探索", "Clear & explore global")}
                                            </button>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <button
                                            type="button"
                                            class="nav-menu-item"
                                            on:click=move |_| {
                                                open_hero();
                                            }
                                        >
                                            {move || t(locale.get(), "选择目的地（你想去哪）", "Pick destination")}
                                        </button>
                                    }.into_any()
                                }
                            }}
                        </div>
                        <div class="nav-menu-divider" aria-hidden="true"></div>
                        <div class="nav-menu-section">
                            <span class="nav-menu-label">{move || t(locale.get(), "语言", "Language")}</span>
                            <div class="language-switcher" aria-label="Language">
                                <button
                                    type="button"
                                    class=move || language_button_class(locale.get() == Locale::Zh)
                                    aria-pressed=move || aria_pressed(locale.get() == Locale::Zh)
                                    on:click=move |_| locale.set(Locale::Zh)
                                >
                                    "中文"
                                </button>
                                <button
                                    type="button"
                                    class=move || language_button_class(locale.get() == Locale::En)
                                    aria-pressed=move || aria_pressed(locale.get() == Locale::En)
                                    on:click=move |_| locale.set(Locale::En)
                                >
                                    "EN"
                                </button>
                            </div>
                        </div>
                        <div class="nav-menu-divider" aria-hidden="true"></div>
                        <div class="nav-menu-section map-menu-section">
                            <span class="nav-menu-label">{move || t(locale.get(), "地图控制", "Map controls")}</span>
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
                                    {move || t(locale.get(), "道路", "Road")}
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
                                    {move || t(locale.get(), "深色", "Dark")}
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
                                    {move || t(locale.get(), "2D 地图", "2D Map")}
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
                                    {move || t(locale.get(), "3D 地球", "3D Globe")}
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
                    </div>
                </details>
            </nav>
        </header>
    }
}

#[component]
fn UserMenu(user: CurrentUser) -> impl IntoView {
    let locale = use_i18n().locale;
    let label = user.name.clone().unwrap_or_else(|| user.email.clone());
    let initial = avatar_initial(&label);
    let title = format!("{} ({})", label, user.email);
    let logout = Action::new(move |_: &()| async move { logout_user().await });

    Effect::new(move |_| {
        if let Some(Ok(())) = logout.value().get() {
            refresh_session();
        }
    });

    view! {
        <details class="user-menu">
            <summary class="user-avatar-link" aria-label="My account" title=title>
                <span class="user-avatar" aria-hidden="true">{initial}</span>
                <span class="user-avatar-name">{label}</span>
            </summary>
            <div class="user-menu-panel">
                <a href="/inspace/my-spaces">{move || t(locale.get(), "我的空间", "My Spaces")}</a>
                <button
                    type="button"
                    on:click=move |_| {
                        logout.dispatch(());
                    }
                >
                    {move || t(locale.get(), "退出登录", "Sign out")}
                </button>
            </div>
        </details>
    }
}

fn nav_class(pathname: &str, search: &str, section: &str) -> &'static str {
    let normalized = if pathname == "/" {
        "/inspace"
    } else {
        pathname
    };
    let on_home = normalized == "/inspace" || normalized == "/";
    let wants_explore = search.contains("explore=1") || search.contains("explore=true");
    let wants_home = search.contains("home=1") || search.contains("home=true");
    let active = match section {
        "home" => on_home && wants_home,
        "explore" => on_home && wants_explore,
        "guides" => normalized.starts_with("/inspace/guides") || normalized.starts_with("/guides"),
        "my-spaces" => {
            normalized.starts_with("/inspace/my-spaces") || normalized.starts_with("/my-spaces")
        }
        _ => false,
    };

    if active {
        "nav-link is-active"
    } else {
        "nav-link"
    }
}

fn nav_menu_class(pathname: &str, section: &str) -> &'static str {
    let normalized = if pathname == "/" {
        "/inspace"
    } else {
        pathname
    };
    let active = match section {
        "guides" => normalized.starts_with("/inspace/guides") || normalized.starts_with("/guides"),
        "my-spaces" => {
            normalized.starts_with("/inspace/my-spaces") || normalized.starts_with("/my-spaces")
        }
        _ => false,
    };

    if active {
        "nav-menu-item is-active"
    } else {
        "nav-menu-item"
    }
}

fn avatar_initial(label: &str) -> String {
    label
        .trim()
        .chars()
        .next()
        .map(|value| value.to_uppercase().collect())
        .unwrap_or_else(|| "U".to_string())
}

fn language_button_class(is_active: bool) -> &'static str {
    if is_active {
        "language-button is-active"
    } else {
        "language-button"
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
