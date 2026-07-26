use leptos::prelude::*;
use leptos_router::hooks::use_location;

use crate::i18n::{t, use_i18n};

#[component]
pub fn AdminNav() -> impl IntoView {
    let locale = use_i18n().locale;
    let location = use_location();
    let pathname = location.pathname;

    view! {
        <section class="backoffice-nav-shell" aria-label=move || t(locale.get(), "管理工作区导航", "Admin workspace navigation")>
            <div class="backoffice-nav-context">
                <span class="backoffice-nav-mark" aria-hidden="true"><AdminIcon name="shield" /></span>
                <span><b>{move || t(locale.get(), "管理控制台", "Admin console")}</b><small>{move || t(locale.get(), "内容、用户与运营", "Content, users and operations")}</small></span>
            </div>
            <nav class="backoffice-nav" aria-label=move || t(locale.get(), "管理页面", "Admin pages")>
                <AdminNavLink href="/inspace/admin" section="dashboard" label_zh="概览" label_en="Overview" icon="dashboard" pathname=pathname />
                <AdminNavLink href="/inspace/admin/home" section="home" label_zh="首页" label_en="Homepage" icon="home" pathname=pathname />
                <AdminNavLink href="/inspace/admin/spaces" section="spaces" label_zh="空间" label_en="Spaces" icon="spaces" pathname=pathname />
                <AdminNavLink href="/inspace/admin/guides" section="guides" label_zh="攻略" label_en="Guides" icon="guides" pathname=pathname />
                <AdminNavLink href="/inspace/admin/resident-applications" section="residents" label_zh="常驻" label_en="Residents" icon="resident" pathname=pathname />
                <AdminNavLink href="/inspace/admin/users" section="users" label_zh="用户" label_en="Users" icon="users" pathname=pathname />
            </nav>
        </section>
    }
}

#[component]
fn AdminNavLink(
    href: &'static str,
    section: &'static str,
    label_zh: &'static str,
    label_en: &'static str,
    icon: &'static str,
    pathname: Memo<String>,
) -> impl IntoView {
    let locale = use_i18n().locale;
    view! {
        <a
            href=href
            class=move || if admin_nav_active(&pathname.get(), section) { "is-active" } else { "" }
            aria-current=move || admin_nav_active(&pathname.get(), section).then_some("page")
        >
            <AdminIcon name=icon />
            <span>{move || t(locale.get(), label_zh, label_en)}</span>
        </a>
    }
}

#[component]
fn AdminIcon(name: &'static str) -> impl IntoView {
    match name {
        "shield" => view! { <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 3 20 6v5c0 5-3.2 8.3-8 10-4.8-1.7-8-5-8-10V6z"/><path d="m9 12 2 2 4-4"/></svg> }.into_any(),
        "dashboard" => view! { <svg viewBox="0 0 24 24" aria-hidden="true"><rect x="4" y="4" width="6" height="6" rx="1"/><rect x="14" y="4" width="6" height="6" rx="1"/><rect x="4" y="14" width="6" height="6" rx="1"/><rect x="14" y="14" width="6" height="6" rx="1"/></svg> }.into_any(),
        "home" => view! { <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M3.5 10.5 12 3l8.5 7.5V20a1 1 0 0 1-1 1h-5v-6h-5v6h-5a1 1 0 0 1-1-1z"/></svg> }.into_any(),
        "spaces" => view! { <svg viewBox="0 0 24 24" aria-hidden="true"><path d="m3.5 6.5 5-2.5 7 2.5 5-2.5v14l-5 2.5-7-2.5-5 2.5zM8.5 4v14M15.5 6.5v14"/></svg> }.into_any(),
        "guides" => view! { <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M5 4.5h10.5A2.5 2.5 0 0 1 18 7v13H7.5A2.5 2.5 0 0 1 5 17.5zM5 17.5A2.5 2.5 0 0 1 7.5 15H18M9 8h5M9 11h4"/></svg> }.into_any(),
        "resident" => view! { <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M5 20V8l7-5 7 5v12M9 20v-6h6v6"/><path d="M8 10h.01M16 10h.01"/></svg> }.into_any(),
        "users" => view! { <svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="9" cy="8" r="3"/><path d="M3.5 20c.5-4 2.4-6 5.5-6s5 2 5.5 6M15.5 5.5a3 3 0 0 1 0 5.5M16 14c2.7.3 4.2 2.3 4.5 6"/></svg> }.into_any(),
        _ => view! { <svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="8"/></svg> }.into_any(),
    }
}

fn admin_nav_active(pathname: &str, section: &str) -> bool {
    let normalized = pathname.strip_prefix("/inspace").unwrap_or(pathname);
    match section {
        "dashboard" => normalized == "/admin" || normalized == "/admin/",
        "home" => normalized.starts_with("/admin/home"),
        "spaces" => normalized.starts_with("/admin/spaces"),
        "guides" => normalized.starts_with("/admin/guides"),
        "residents" => normalized.starts_with("/admin/resident-applications"),
        "users" => normalized.starts_with("/admin/users"),
        _ => false,
    }
}
