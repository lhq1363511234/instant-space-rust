use leptos::prelude::*;

use crate::i18n::{t, use_i18n};

#[component]
pub fn AdminNav() -> impl IntoView {
    let locale = use_i18n().locale;

    view! {
        <nav class="admin-nav">
            <a href="/inspace/admin">{move || t(locale.get(), "仪表盘", "Dashboard")}</a>
            <a href="/inspace/admin/spaces">{move || t(locale.get(), "空间", "Spaces")}</a>
            <a href="/inspace/admin/guides">{move || t(locale.get(), "攻略", "Guides")}</a>
            <a href="/inspace/admin/templates">{move || t(locale.get(), "模板", "Templates")}</a>
            <a href="/inspace/admin/resident-applications">{move || t(locale.get(), "常驻", "Resident")}</a>
        </nav>
    }
}
