use instant_domain::auth::CurrentUser;
use instant_map_ui::{MapProjection, MapStyle};
use leptos::prelude::*;
use leptos_router::hooks::use_location;

use crate::{
    app_state::{open_hero, refresh_session, use_app_refresh_state},
    components::space_form::{provide_create_space_modal, use_create_space_modal},
    i18n::{t, use_i18n, Locale},
    server::auth::{current_session, logout_user},
};

#[component]
pub fn Header() -> impl IntoView {
    let locale = use_i18n().locale;
    let refresh = use_app_refresh_state();
    let location = use_location();
    let pathname = location.pathname;
    let drawer_open = RwSignal::new(false);
    let sidebar_collapsed = RwSignal::new(false);
    let create_modal = use_create_space_modal().unwrap_or_else(provide_create_space_modal);
    let session = Resource::new(
        move || refresh.session.get(),
        |_| async move { current_session().await.ok().flatten() },
    );

    view! {
        <a class="shell-skip-link" href="#main-content">
            {move || t(locale.get(), "跳到主要内容", "Skip to content")}
        </a>

        <aside
            class=move || shell_sidebar_class(drawer_open.get(), sidebar_collapsed.get())
            aria-label=move || t(locale.get(), "inspace 主导航", "inspace primary navigation")
        >
            <div class="shell-sidebar-head">
                <a href="/inspace" class="shell-brand" aria-label="inspace home" on:click=move |_| drawer_open.set(false)>
                    <span class="shell-brand-mark" aria-hidden="true"><i></i></span>
                    <span class="shell-brand-copy"><b>"inspace"</b><small>"beyond the map"</small></span>
                </a>
                <button
                    type="button"
                    class="shell-collapse-button"
                    aria-label=move || if sidebar_collapsed.get() {
                        t(locale.get(), "展开侧栏", "Expand sidebar")
                    } else {
                        t(locale.get(), "收起侧栏", "Collapse sidebar")
                    }
                    aria-expanded=move || (!sidebar_collapsed.get()).to_string()
                    on:click=move |_| sidebar_collapsed.update(|value| *value = !*value)
                >
                    <ShellIcon name="panel" />
                </button>
            </div>

            <nav class="shell-primary-nav" aria-label=move || t(locale.get(), "主要功能", "Main features")>
                <ShellNavLink
                    href="/inspace"
                    label_zh="首页"
                    label_en="Home"
                    hint_zh="了解 inspace"
                    hint_en="About inspace"
                    icon="home"
                    active=move || shell_nav_active(&pathname.get(), "home")
                    on_navigate=Callback::new(move |_| { open_hero(); drawer_open.set(false); })
                />
                <ShellNavLink
                    href="/inspace/map"
                    label_zh="发现地图"
                    label_en="Discover map"
                    hint_zh="从地点进入空间"
                    hint_en="Enter from a place"
                    icon="map"
                    active=move || shell_nav_active(&pathname.get(), "map")
                    on_navigate=Callback::new(move |_| drawer_open.set(false))
                />
                <ShellNavLink
                    href="/inspace/explore"
                    label_zh="探索空间"
                    label_en="Explore spaces"
                    hint_zh="分类、搜索与筛选"
                    hint_en="Browse and filter"
                    icon="compass"
                    active=move || shell_nav_active(&pathname.get(), "explore")
                    on_navigate=Callback::new(move |_| drawer_open.set(false))
                />
                <ShellNavLink
                    href="/inspace/guides"
                    label_zh="空间攻略"
                    label_en="Guides"
                    hint_zh="路线、玩法与现场经验"
                    hint_en="Routes and local insight"
                    icon="book"
                    active=move || shell_nav_active(&pathname.get(), "guides")
                    on_navigate=Callback::new(move |_| drawer_open.set(false))
                />
                <ShellNavLink
                    href="/inspace/lives"
                    label_zh="数字生命"
                    label_en="Digital lives"
                    hint_zh="云上家 · 在侧 · 追远"
                    hint_en="Cloud home · companions · memory"
                    icon="paw"
                    active=move || shell_nav_active(&pathname.get(), "lives")
                    on_navigate=Callback::new(move |_| drawer_open.set(false))
                />
            </nav>

            <Show when=move || shell_nav_active(&pathname.get(), "map")>
                <MapSidebarTools refresh=refresh drawer_open=drawer_open />
            </Show>

            <div class="shell-create-zone">
                <button
                    type="button"
                    class="shell-create-button"
                    on:click=move |_| {
                        drawer_open.set(false);
                        create_modal.open.set(true);
                    }
                >
                    <ShellIcon name="plus" />
                    <span>{move || t(locale.get(), "创建空间", "Create space")}</span>
                </button>
            </div>

            <div class="shell-sidebar-spacer"></div>

            <div class="shell-sidebar-bottom">
                <a
                    href="/inspace/about"
                    class=move || if shell_nav_active(&pathname.get(), "about") { "shell-utility-link shell-about-link is-active" } else { "shell-utility-link shell-about-link" }
                    aria-current=move || shell_nav_active(&pathname.get(), "about").then_some("page")
                    on:click=move |_| drawer_open.set(false)
                >
                    <ShellIcon name="info" />
                    <span><b>{move || t(locale.get(), "关于 inspace", "About inspace")}</b><small>{move || t(locale.get(), "愿景与主理人招募", "Vision and host call")}</small></span>
                </a>
                <Suspense fallback=move || view! {
                    <div class="shell-account-skeleton" aria-hidden="true"><span></span><span></span></div>
                }>
                    {move || Suspend::new(async move {
                        match session.await {
                            Some(user) => view! {
                                <SidebarAccount user=user pathname=pathname.get() drawer_open=drawer_open />
                            }.into_any(),
                            None => view! {
                                <a href="/inspace/login" class="shell-login-link" on:click=move |_| drawer_open.set(false)>
                                    <ShellIcon name="user" />
                                    <span>{move || t(locale.get(), "登录 / 注册", "Sign in")}</span>
                                </a>
                            }.into_any(),
                        }
                    })}
                </Suspense>

                <div class="shell-language-row" aria-label="Language">
                    <button
                        type="button"
                        class=move || language_button_class(locale.get() == Locale::Zh)
                        aria-pressed=move || aria_pressed(locale.get() == Locale::Zh)
                        on:click=move |_| locale.set(Locale::Zh)
                    >"中"</button>
                    <button
                        type="button"
                        class=move || language_button_class(locale.get() == Locale::En)
                        aria-pressed=move || aria_pressed(locale.get() == Locale::En)
                        on:click=move |_| locale.set(Locale::En)
                    >"EN"</button>
                </div>
            </div>
        </aside>

        <div
            class=move || if drawer_open.get() { "shell-drawer-scrim is-open" } else { "shell-drawer-scrim" }
            aria-hidden="true"
            on:click=move |_| drawer_open.set(false)
        ></div>

        <header class="shell-topbar" aria-label=move || t(locale.get(), "页面工具栏", "Page toolbar")>
            <button
                type="button"
                class="shell-menu-button"
                aria-label=move || t(locale.get(), "打开导航", "Open navigation")
                aria-expanded=move || drawer_open.get().to_string()
                on:click=move |_| drawer_open.set(true)
            >
                <ShellIcon name="menu" />
            </button>
            <div class="shell-topbar-context">
                <span class="shell-topbar-title">{move || shell_page_title(&pathname.get(), locale.get())}</span>
                <span class="shell-topbar-path">{move || shell_page_hint(&pathname.get(), locale.get())}</span>
            </div>
            <form class="shell-global-search" role="search" action="/inspace/explore" method="get">
                <ShellIcon name="search" />
                <input
                    type="search"
                    name="q"
                    autocomplete="off"
                    placeholder=move || t(locale.get(), "搜索地点、空间或攻略", "Search places, spaces, or guides")
                    aria-label=move || t(locale.get(), "全站搜索", "Search")
                />
                <kbd aria-hidden="true">"/"</kbd>
            </form>
            <div class="shell-topbar-actions">
                <div class="shell-topbar-language" aria-label="Language">
                    <button
                        type="button"
                        class=move || language_button_class(locale.get() == Locale::Zh)
                        aria-pressed=move || aria_pressed(locale.get() == Locale::Zh)
                        on:click=move |_| locale.set(Locale::Zh)
                    >"中"</button>
                    <button
                        type="button"
                        class=move || language_button_class(locale.get() == Locale::En)
                        aria-pressed=move || aria_pressed(locale.get() == Locale::En)
                        on:click=move |_| locale.set(Locale::En)
                    >"EN"</button>
                </div>
                <Suspense fallback=move || view! { <a href="/inspace/login" class="shell-topbar-login">{move || t(locale.get(), "登录", "Sign in")}</a> }>
                    {move || Suspend::new(async move {
                        match session.await {
                            Some(user) => view! { <TopbarAccount user=user /> }.into_any(),
                            None => view! { <a href="/inspace/login" class="shell-topbar-login">{move || t(locale.get(), "登录", "Sign in")}</a> }.into_any(),
                        }
                    })}
                </Suspense>
            </div>
        </header>

        <nav
            class=move || mobile_bottom_class(&pathname.get())
            aria-label=move || t(locale.get(), "手机主导航", "Mobile primary navigation")
        >
            <MobileNavLink href="/inspace" label_zh="首页" label_en="Home" icon="home" active=move || shell_nav_active(&pathname.get(), "home") />
            <MobileNavLink href="/inspace/map" label_zh="地图" label_en="Map" icon="map" active=move || shell_nav_active(&pathname.get(), "map") />
            <MobileNavLink href="/inspace/explore" label_zh="探索" label_en="Explore" icon="compass" active=move || shell_nav_active(&pathname.get(), "explore") />
            <MobileNavLink href="/inspace/guides" label_zh="攻略" label_en="Guides" icon="book" active=move || shell_nav_active(&pathname.get(), "guides") />
            <MobileNavLink href="/inspace/my-spaces" label_zh="我的" label_en="Me" icon="user" active=move || shell_nav_active(&pathname.get(), "my-spaces") />
        </nav>
    }
}

#[component]
fn ShellNavLink(
    href: &'static str,
    label_zh: &'static str,
    label_en: &'static str,
    hint_zh: &'static str,
    hint_en: &'static str,
    icon: &'static str,
    active: impl Fn() -> bool + Copy + Send + 'static,
    on_navigate: Callback<()>,
) -> impl IntoView {
    let locale = use_i18n().locale;
    view! {
        <a
            href=href
            class=move || if active() { "shell-nav-link is-active" } else { "shell-nav-link" }
            aria-current=move || active().then_some("page")
            on:click=move |_| on_navigate.run(())
        >
            <span class="shell-nav-icon"><ShellIcon name=icon /></span>
            <span class="shell-nav-copy">
                <b>{move || t(locale.get(), label_zh, label_en)}</b>
                <small>{move || t(locale.get(), hint_zh, hint_en)}</small>
            </span>
        </a>
    }
}

#[component]
fn MapSidebarTools(
    refresh: crate::app_state::AppRefreshState,
    drawer_open: RwSignal<bool>,
) -> impl IntoView {
    let locale = use_i18n().locale;
    view! {
        <section class="shell-map-tools" aria-label=move || t(locale.get(), "地图模式", "Map mode")>
            <div class="shell-map-tools-head">
                <ShellIcon name="layers" />
                <span><b>{move || t(locale.get(), "地图模式", "Map mode")}</b><small>{move || t(locale.get(), "显示方式", "Presentation")}</small></span>
            </div>
            <div class="shell-map-choice" role="group" aria-label=move || t(locale.get(), "地图样式", "Map style")>
                <button
                    type="button"
                    class=move || map_choice_class(refresh.map_style.get() == MapStyle::Road)
                    aria-pressed=move || aria_pressed(refresh.map_style.get() == MapStyle::Road)
                    on:click=move |_| {
                        refresh.map_style.set(MapStyle::Road);
                        instant_map_ui::set_style("map", MapStyle::Road);
                        drawer_open.set(false);
                    }
                >{move || t(locale.get(), "道路", "Road")}</button>
                <button
                    type="button"
                    class=move || map_choice_class(refresh.map_style.get() == MapStyle::Dark)
                    aria-pressed=move || aria_pressed(refresh.map_style.get() == MapStyle::Dark)
                    on:click=move |_| {
                        refresh.map_style.set(MapStyle::Dark);
                        instant_map_ui::set_style("map", MapStyle::Dark);
                        drawer_open.set(false);
                    }
                >{move || t(locale.get(), "深色", "Dark")}</button>
            </div>
            <div class="shell-map-choice map-projection-switcher" role="group" aria-label=move || t(locale.get(), "地图维度", "Map projection")>
                <button
                    type="button"
                    class=move || map_choice_class(refresh.map_projection.get() == MapProjection::Flat2d)
                    aria-label="Switch to 2D map"
                    aria-pressed=move || aria_pressed(refresh.map_projection.get() == MapProjection::Flat2d)
                    on:click=move |_| {
                        refresh.map_projection.set(MapProjection::Flat2d);
                        instant_map_ui::set_projection("map", MapProjection::Flat2d);
                        drawer_open.set(false);
                    }
                >"2D"</button>
                <button
                    type="button"
                    class=move || map_choice_class(refresh.map_projection.get() == MapProjection::Globe3d)
                    aria-label="Switch to 3D globe"
                    aria-pressed=move || aria_pressed(refresh.map_projection.get() == MapProjection::Globe3d)
                    on:click=move |_| {
                        refresh.map_projection.set(MapProjection::Globe3d);
                        instant_map_ui::set_projection("map", MapProjection::Globe3d);
                        drawer_open.set(false);
                    }
                >"3D"</button>
            </div>
        </section>
    }
}

#[component]
fn SidebarAccount(
    user: CurrentUser,
    pathname: String,
    drawer_open: RwSignal<bool>,
) -> impl IntoView {
    let locale = use_i18n().locale;
    let label = user.name.clone().unwrap_or_else(|| user.email.clone());
    let initial = avatar_initial(&label);
    let is_admin = user.role.is_admin();
    let workspace_active =
        pathname.starts_with("/inspace/my-spaces") || pathname.starts_with("/my-spaces");
    let admin_active = pathname.starts_with("/inspace/admin") || pathname.starts_with("/admin");

    view! {
        <div class="shell-workspace-links">
            <a
                href="/inspace/my-spaces"
                class=if workspace_active { "shell-utility-link is-active" } else { "shell-utility-link" }
                aria-current=workspace_active.then_some("page")
                on:click=move |_| drawer_open.set(false)
            >
                <ShellIcon name="grid" />
                <span><b>{move || t(locale.get(), "用户工作台", "Workspace")}</b><small>{move || t(locale.get(), "空间与内容管理", "Manage your spaces")}</small></span>
            </a>
            {is_admin.then(|| view! {
                <a
                    href="/inspace/admin"
                    class=if admin_active { "shell-utility-link is-active" } else { "shell-utility-link" }
                    aria-current=admin_active.then_some("page")
                    on:click=move |_| drawer_open.set(false)
                >
                    <ShellIcon name="shield" />
                    <span><b>{move || t(locale.get(), "管理员后台", "Admin console")}</b><small>{move || t(locale.get(), "运营与系统管理", "Operations and system")}</small></span>
                </a>
            })}
        </div>
        <div class="shell-account-row">
            <span class="shell-account-avatar" aria-hidden="true">{initial}</span>
            <span class="shell-account-copy"><b>{label}</b><small>{user.email}</small></span>
        </div>
    }
}

#[component]
fn TopbarAccount(user: CurrentUser) -> impl IntoView {
    let locale = use_i18n().locale;
    let label = user.name.clone().unwrap_or_else(|| user.email.clone());
    let initial = avatar_initial(&label);
    let logout = Action::new(move |_: &()| async move { logout_user().await });

    Effect::new(move |_| {
        if let Some(Ok(())) = logout.value().get() {
            refresh_session();
        }
    });

    view! {
        <details class="shell-account-menu">
            <summary aria-label=move || t(locale.get(), "打开账号菜单", "Open account menu") title=label>
                <span>{initial}</span>
            </summary>
            <div class="shell-account-popover">
                <a href="/inspace/my-spaces"><ShellIcon name="grid" />{move || t(locale.get(), "用户工作台", "Workspace")}</a>
                {user.role.is_admin().then(|| view! { <a href="/inspace/admin"><ShellIcon name="shield" />{move || t(locale.get(), "管理员后台", "Admin console")}</a> })}
                <button type="button" on:click=move |_| { logout.dispatch(()); }>
                    <ShellIcon name="logout" />{move || t(locale.get(), "退出登录", "Sign out")}
                </button>
            </div>
        </details>
    }
}

#[component]
fn MobileNavLink(
    href: &'static str,
    label_zh: &'static str,
    label_en: &'static str,
    icon: &'static str,
    active: impl Fn() -> bool + Copy + Send + 'static,
) -> impl IntoView {
    let locale = use_i18n().locale;
    view! {
        <a href=href class=move || if active() { "is-active" } else { "" } aria-current=move || active().then_some("page")>
            <ShellIcon name=icon />
            <span>{move || t(locale.get(), label_zh, label_en)}</span>
        </a>
    }
}

#[component]
fn ShellIcon(name: &'static str) -> impl IntoView {
    match name {
        "home" => view! { <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M3.5 10.5 12 3l8.5 7.5V20a1 1 0 0 1-1 1h-5v-6h-5v6h-5a1 1 0 0 1-1-1z"/></svg> }.into_any(),
        "map" => view! { <svg viewBox="0 0 24 24" aria-hidden="true"><path d="m3.5 6.5 5-2.5 7 2.5 5-2.5v14l-5 2.5-7-2.5-5 2.5zM8.5 4v14M15.5 6.5v14"/></svg> }.into_any(),
        "compass" => view! { <svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="9"/><path d="m15.5 8.5-2.1 4.9-4.9 2.1 2.1-4.9z"/></svg> }.into_any(),
        "book" => view! { <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M5 4.5h10.5A2.5 2.5 0 0 1 18 7v13H7.5A2.5 2.5 0 0 1 5 17.5zM5 17.5A2.5 2.5 0 0 1 7.5 15H18M9 8h5M9 11h4"/></svg> }.into_any(),
        "plus" => view! { <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 5v14M5 12h14"/></svg> }.into_any(),
        "paw" => view! { <svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="7" cy="9" r="2.1"/><circle cx="12" cy="6.5" r="2.1"/><circle cx="17" cy="9" r="2.1"/><path d="M12 11.5c2.6 0 5.2 2.3 5.2 4.9 0 1.4-1 2.4-2.2 2.4-1.1 0-1.9-.6-3-.6s-1.9.6-3 .6c-1.2 0-2.2-1-2.2-2.4 0-2.6 2.6-4.9 5.2-4.9z"/></svg> }.into_any(),
        "user" => view! { <svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="8" r="3.5"/><path d="M5.5 20c.6-4 2.8-6 6.5-6s5.9 2 6.5 6"/></svg> }.into_any(),
        "grid" => view! { <svg viewBox="0 0 24 24" aria-hidden="true"><rect x="4" y="4" width="6" height="6" rx="1"/><rect x="14" y="4" width="6" height="6" rx="1"/><rect x="4" y="14" width="6" height="6" rx="1"/><rect x="14" y="14" width="6" height="6" rx="1"/></svg> }.into_any(),
        "shield" => view! { <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 3 20 6v5c0 5-3.2 8.3-8 10-4.8-1.7-8-5-8-10V6z"/><path d="m9 12 2 2 4-4"/></svg> }.into_any(),
        "search" => view! { <svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="11" cy="11" r="7"/><path d="m20 20-3.4-3.4"/></svg> }.into_any(),
        "menu" => view! { <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M4 7h16M4 12h16M4 17h16"/></svg> }.into_any(),
        "panel" => view! { <svg viewBox="0 0 24 24" aria-hidden="true"><rect x="3" y="4" width="18" height="16" rx="2"/><path d="M9 4v16M14 9l-3 3 3 3"/></svg> }.into_any(),
        "layers" => view! { <svg viewBox="0 0 24 24" aria-hidden="true"><path d="m12 3 8 4-8 4-8-4zM4 12l8 4 8-4M4 17l8 4 8-4"/></svg> }.into_any(),
        "info" => view! { <svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="9"/><path d="M12 10v7M12 7h.01"/></svg> }.into_any(),
        "logout" => view! { <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M10 5H5v14h5M14 8l4 4-4 4M18 12H9"/></svg> }.into_any(),
        _ => view! { <svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="8"/></svg> }.into_any(),
    }
}

fn shell_sidebar_class(drawer_open: bool, collapsed: bool) -> &'static str {
    match (drawer_open, collapsed) {
        (true, true) => "shell-sidebar is-open is-collapsed",
        (true, false) => "shell-sidebar is-open",
        (false, true) => "shell-sidebar is-collapsed",
        (false, false) => "shell-sidebar",
    }
}

fn shell_nav_active(pathname: &str, section: &str) -> bool {
    let normalized = if pathname == "/" {
        "/inspace"
    } else {
        pathname
    };
    match section {
        "home" => normalized == "/inspace",
        "map" => normalized.starts_with("/inspace/map") || normalized.starts_with("/map"),
        "explore" => {
            normalized.starts_with("/inspace/explore") || normalized.starts_with("/explore")
        }
        "guides" => normalized.starts_with("/inspace/guides") || normalized.starts_with("/guides"),
        "my-spaces" => {
            normalized.starts_with("/inspace/my-spaces") || normalized.starts_with("/my-spaces")
        }
        "lives" => {
            normalized.starts_with("/inspace/lives")
                || normalized.starts_with("/lives")
                || normalized.starts_with("/inspace/homes/")
                || normalized.starts_with("/homes/")
        }
        "about" => normalized.starts_with("/inspace/about") || normalized.starts_with("/about"),
        _ => false,
    }
}

fn mobile_bottom_class(pathname: &str) -> &'static str {
    if pathname.contains("/admin") || pathname.contains("/spaces/") {
        "shell-mobile-nav is-hidden"
    } else {
        "shell-mobile-nav"
    }
}

fn shell_page_title(pathname: &str, locale: Locale) -> &'static str {
    if pathname.contains("/admin") {
        t(locale, "管理员后台", "Admin console")
    } else if pathname.contains("/about") {
        t(locale, "关于 inspace", "About inspace")
    } else if pathname.contains("/my-spaces") {
        t(locale, "用户工作台", "Workspace")
    } else if pathname.contains("/guides") {
        t(locale, "空间攻略", "Guides")
    } else if pathname.contains("/explore") {
        t(locale, "探索空间", "Explore")
    } else if pathname.contains("/map") {
        t(locale, "发现地图", "Map")
    } else if pathname.contains("/lives") {
        t(locale, "数字生命", "Digital lives")
    } else if pathname.contains("/homes/") {
        t(locale, "云上家", "Cloud home")
    } else if pathname.contains("/spaces/") {
        t(locale, "空间详情", "Space")
    } else {
        "inspace"
    }
}

fn shell_page_hint(pathname: &str, locale: Locale) -> &'static str {
    if pathname.contains("/admin") {
        t(locale, "运营、内容与系统", "Operations and system")
    } else if pathname.contains("/about") {
        t(locale, "愿景与空间主理人", "Vision and local hosts")
    } else if pathname.contains("/my-spaces") {
        t(locale, "管理你的空间", "Manage your spaces")
    } else if pathname.contains("/guides") {
        t(locale, "路线、玩法与经验", "Routes and local insight")
    } else if pathname.contains("/explore") {
        t(locale, "按地点和分类发现", "Browse by place and category")
    } else if pathname.contains("/map") {
        t(locale, "打开地图才加载瓦片", "Tiles load on map open")
    } else if pathname.contains("/lives") {
        t(
            locale,
            "云上家 · 在侧 · 追远",
            "Cloud home · companions · memory",
        )
    } else if pathname.contains("/homes/") {
        t(locale, "叩门而入", "Knock to enter")
    } else if pathname.contains("/spaces/") {
        t(locale, "进入真实地点的数字空间", "Enter the place")
    } else {
        t(locale, "走到导航的尽头，体验才开始", "Beyond the map")
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

fn map_choice_class(is_active: bool) -> &'static str {
    if is_active {
        "is-active"
    } else {
        ""
    }
}

fn aria_pressed(is_active: bool) -> &'static str {
    if is_active {
        "true"
    } else {
        "false"
    }
}
