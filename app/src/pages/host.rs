use leptos::prelude::*;

use crate::app_state::{refresh_spaces, use_app_refresh_state};
use crate::components::space_experience_modal::OpenSpaceLink;
use crate::components::space_form::OpenCreateSpaceButton;
use crate::components::space_share::SpaceSharePanel;
use crate::feedback::use_feedback;
use crate::i18n::{localize_optional, t, use_i18n};
use crate::pages::space::SpacePanel;
use crate::server::auth::current_session;
use crate::server::guides::{delete_guide, list_manageable_space_guides};
use crate::server::spaces::{
    add_my_space_member, apply_my_space_resident, archive_my_space_template, close_my_space,
    delete_my_space, get_space_detail, list_my_space_members, list_my_spaces, reactivate_my_space,
    regenerate_space_password, remove_my_space_member, update_my_space, PasswordRotationResult,
    SpaceMarker,
};
use crate::server::world::{
    appoint_space_host, get_my_space_governance, leave_space_host_role, remove_space_host,
    set_space_system_care, transfer_space_host, update_space_recruitment_note,
};
use instant_domain::guides::GuideStatus;
use instant_domain::world::{
    HostGovernanceState, HostTenureRole, SpaceGovernanceEvent, SpaceHostIdentity,
};

#[component]
pub fn HostRoutes() -> impl IntoView {
    let locale = use_i18n().locale;
    let refresh = use_app_refresh_state();
    let session = Resource::new(
        move || refresh.session.get(),
        |_| async move { current_session().await.ok().flatten() },
    );
    let my_spaces = Resource::new(
        move || refresh.spaces.get(),
        |_| async move { list_my_spaces().await.unwrap_or_default() },
    );

    view! {
        <main id="main-content" class="page my-spaces-page">
            <div class="page-head">
                <div>
                    <p class="eyebrow">"inspace"</p>
                    <h1>{move || {
                        let is_admin = session
                            .get()
                            .and_then(|value| value)
                            .is_some_and(|user| user.role.is_admin());
                        if is_admin {
                            t(locale.get(), "全部空间管理", "All Spaces")
                        } else {
                            t(locale.get(), "我的旅行空间", "My travel spaces")
                        }
                    }}</h1>
                    <p>{move || {
                        let is_admin = session
                            .get()
                            .and_then(|value| value)
                            .is_some_and(|user| user.role.is_admin());
                        if is_admin {
                            t(locale.get(), "管理员可管理全站空间；普通用户只管理自己创建的空间。", "Admins can manage all spaces; regular users manage only their own spaces.")
                        } else {
                            t(locale.get(), "管理你创建的真实地点入口：基础信息、密码、二维码分享、驻留申请，以及这个空间下的相关攻略。", "Manage your real-place entries: basic info, passwords, QR sharing, residency requests, and related guides under each Space.")
                        }
                    }}</p>
                </div>
                <OpenCreateSpaceButton />
            </div>
            <section class="workspace-guidance" aria-label=move || t(locale.get(), "攻略入口说明", "Where guides come from")>
                <p class="survey-kicker">{move || t(locale.get(), "怎么写攻略", "How guides work")}</p>
                <ol class="workspace-guidance-steps">
                    <li>
                        <b>"1"</b>
                        <span>
                            <strong>{move || t(locale.get(), "先建一个空间", "Create a Space first")}</strong>
                            <small>{move || t(locale.get(), "一个真实地点 = 一个空间，攻略挂在它下面。", "One real place = one Space. Guides live under it.")}</small>
                        </span>
                    </li>
                    <li>
                        <b>"2"</b>
                        <span>
                            <strong>{move || t(locale.get(), "在空间里点「写攻略」", "Open the Space and click Write guide")}</strong>
                            <small>{move || t(locale.get(), "攻略的入口在空间内部，不在导航栏——这样地点信息会自动带入。", "The entry point is inside the Space, not in the nav — the place details are filled in for you.")}</small>
                        </span>
                    </li>
                    <li>
                        <b>"3"</b>
                        <span>
                            <strong>{move || t(locale.get(), "发布后分享二维码", "Publish, then share the QR")}</strong>
                            <small>{move || t(locale.get(), "扫码的人进入的是空间，能看到这个地点的全部攻略。", "Scanning opens the Space, where every guide for this place is listed.")}</small>
                        </span>
                    </li>
                </ol>
            </section>
            <Suspense fallback=move || view! { <p>{move || t(locale.get(), "正在检查登录状态", "Checking session")}</p> }>
                {move || Suspend::new(async move {
                    if session.await.is_some() {
                        view! {
                            <Suspense fallback=move || view! { <div class="space-list-skeleton"><span></span><span></span></div> }>
                                {move || Suspend::new(async move {
                                    let items = my_spaces.await;
                                    view! { <MySpaceList items=items /> }
                                })}
                            </Suspense>
                        }.into_any()
                    } else {
                        view! {
                            <section class="form">
                                <h2>{move || t(locale.get(), "请先登录", "Sign in first")}</h2>
                                <p>{move || t(locale.get(), "登录后可以创建和管理你的空间。", "Sign in to create and manage your spaces.")}</p>
                                <a class="button button-primary" href="/inspace/login">
                                    {move || t(locale.get(), "去登录", "Go to sign in")}
                                </a>
                            </section>
                        }.into_any()
                    }
                })}
            </Suspense>
        </main>
    }
}

#[component]
fn MySpaceList(items: Vec<SpaceMarker>) -> impl IntoView {
    let locale = use_i18n().locale;
    if items.is_empty() {
        return view! {
            <section class="empty-state">
                <strong>{move || t(locale.get(), "还没有空间", "No spaces yet")}</strong>
                <span>{move || t(locale.get(), "点击创建空间，把一个景点、餐馆、街区或活动地点变成可扫码分享的数字入口。", "Create a Space to turn a scenic spot, restaurant, district, or event place into a QR-shareable digital entry.")}</span>
            </section>
        }
        .into_any();
    }

    const PAGE_SIZE: usize = 24;
    let items = StoredValue::new(items);
    let query = RwSignal::new(String::new());
    let page = RwSignal::new(1usize);

    let matching_count = move || {
        let needle = query.get().trim().to_lowercase();
        items.with_value(|all| {
            all.iter()
                .filter(|space| {
                    needle.is_empty()
                        || space.name_zh.to_lowercase().contains(&needle)
                        || space
                            .name_en
                            .as_deref()
                            .is_some_and(|name| name.to_lowercase().contains(&needle))
                        || location_label(space).to_lowercase().contains(&needle)
                })
                .count()
        })
    };

    let page_items = move || {
        let needle = query.get().trim().to_lowercase();
        let count = matching_count();
        let total_pages = count.max(1).div_ceil(PAGE_SIZE);
        let current = page.get().clamp(1, total_pages);
        if current != page.get_untracked() {
            page.set(current);
        }
        let start = (current - 1) * PAGE_SIZE;
        items.with_value(|all| {
            all.iter()
                .filter(|space| {
                    needle.is_empty()
                        || space.name_zh.to_lowercase().contains(&needle)
                        || space
                            .name_en
                            .as_deref()
                            .is_some_and(|name| name.to_lowercase().contains(&needle))
                        || location_label(space).to_lowercase().contains(&needle)
                })
                .skip(start)
                .take(PAGE_SIZE)
                .cloned()
                .collect::<Vec<_>>()
        })
    };

    view! {
        <section class="my-space-directory" aria-label="My spaces">
            <div class="my-space-toolbar">
                <label class="my-space-search">
                    <span>{move || t(locale.get(), "查找空间", "Find a Space")}</span>
                    <input
                        type="search"
                        placeholder=move || t(locale.get(), "输入空间名、城市或地点", "Space name, city, or place")
                        prop:value=move || query.get()
                        on:input=move |ev| {
                            query.set(event_target_value(&ev));
                            page.set(1);
                        }
                    />
                </label>
                <p class="my-space-summary" aria-live="polite">
                    {move || {
                        let count = matching_count();
                        let total_pages = count.max(1).div_ceil(PAGE_SIZE);
                        let current = page.get().clamp(1, total_pages);
                        if locale.get() == crate::i18n::Locale::Zh {
                            format!("{count} 个空间 · 第 {current}/{total_pages} 页")
                        } else {
                            format!("{count} Spaces · page {current}/{total_pages}")
                        }
                    }}
                </p>
            </div>

            <Show
                when=move || { matching_count() > 0 }
                fallback=move || view! {
                    <div class="directory-empty">
                        <strong>{move || t(locale.get(), "没有找到匹配的空间", "No matching Spaces")}</strong>
                        <span>{move || t(locale.get(), "换一个名称或地点再试。", "Try another name or place.")}</span>
                    </div>
                }
            >
                <section class="my-space-grid" aria-label="My spaces page">
                    <For
                        each=page_items
                        key=|space| space.id.clone()
                        children=move |space| view! { <MySpaceCard space=space /> }
                    />
                </section>
                <nav class="my-space-pagination" aria-label=move || t(locale.get(), "我的空间分页", "My Spaces pagination")>
                    <button
                        type="button"
                        class="button button-secondary-light"
                        disabled=move || page.get() <= 1
                        on:click=move |_| page.update(|value| *value = value.saturating_sub(1).max(1))
                    >
                        {move || t(locale.get(), "上一页", "Previous")}
                    </button>
                    <span>
                        {move || {
                            let total_pages = matching_count().max(1).div_ceil(PAGE_SIZE);
                            format!("{} / {}", page.get().clamp(1, total_pages), total_pages)
                        }}
                    </span>
                    <button
                        type="button"
                        class="button button-secondary-light"
                        disabled=move || {
                            let total_pages = matching_count().max(1).div_ceil(PAGE_SIZE);
                            page.get() >= total_pages
                        }
                        on:click=move |_| {
                            let total_pages = matching_count().max(1).div_ceil(PAGE_SIZE);
                            page.update(|value| *value = (*value + 1).min(total_pages));
                        }
                    >
                        {move || t(locale.get(), "下一页", "Next")}
                    </button>
                </nav>
            </Show>
        </section>
    }
    .into_any()
}

#[component]
fn MySpaceCard(space: SpaceMarker) -> impl IntoView {
    let locale = use_i18n().locale;
    let manage_open = RwSignal::new(false);
    let name_zh = space.name_zh.clone();
    let name_en = space.name_en.clone();
    let location = location_label(&space);
    let visibility = if space.is_public { "Public" } else { "Private" };
    let coordinates = format!("{:.5}, {:.5}", space.lat, space.lng);
    let status = space.status.clone();
    let expires_at = space
        .expires_at
        .clone()
        .unwrap_or_else(|| "No expiry".to_string());
    let modal_space = space.clone();
    let space_id = space.id.clone();
    let write_guide_href = format!("/inspace/guides/new?space_id={}", space.id);
    let world_href = format!("/inspace/world/{}?via=link", space.id);

    view! {
        <article class="my-space-card">
            <div>
                <h2>{move || localize_optional(locale.get(), &name_zh, name_en.as_deref())}</h2>
                <p>{location}</p>
            </div>
            <div class="space-card-meta">
                <span class="space-badge">{status}</span>
                <span class="space-badge">{visibility}</span>
                <span class="space-coordinates">{coordinates}</span>
            </div>
            <p class="my-space-expiry">
                {move || t(locale.get(), "到期时间：", "Expires: ")}
                {expires_at}
            </p>
            <div class="my-space-card-actions">
                <OpenSpaceLink space_id=space_id.clone() initial_panel=SpacePanel::Wall class="button button-secondary-light">{move || t(locale.get(), "打开空间", "Open Space")}</OpenSpaceLink>
                <a class="button button-secondary-light" href=write_guide_href>{move || t(locale.get(), "写攻略", "Write guide")}</a>
                <OpenSpaceLink space_id=space_id.clone() initial_panel=SpacePanel::Discussion class="button button-secondary-light">{move || t(locale.get(), "讨论区", "Discussion")}</OpenSpaceLink>
                <a class="button button-secondary-light" href=world_href>{move || t(locale.get(), "进入场景", "Enter scene")}</a>
                <button
                    type="button"
                    class="button button-primary manage-space-open-btn"
                    on:click=move |ev| {
                        ev.stop_propagation();
                        manage_open.set(true);
                    }
                >
                    {move || t(locale.get(), "管理空间", "Manage")}
                </button>
            </div>
        </article>
        {move || if manage_open.get() {
            view! {
                <ManageSpaceModal space=modal_space.clone() open=manage_open />
            }.into_any()
        } else {
            view! { <span class="manage-space-closed" aria-hidden="true"></span> }.into_any()
        }}
    }
}

#[component]
pub(crate) fn ManageSpaceModal(space: SpaceMarker, open: RwSignal<bool>) -> impl IntoView {
    let locale = use_i18n().locale;
    let title = space
        .name_en
        .clone()
        .unwrap_or_else(|| space.name_zh.clone());
    view! {
        <div
            class="modal-backdrop manage-space-backdrop"
            role="presentation"
            on:click=move |_| {
                // click outside closes
                refresh_spaces();
                open.set(false);
            }
        >
            <section
                class="modal-card manage-space-modal"
                role="dialog"
                aria-modal="true"
                aria-labelledby="manage-space-title"
                on:click=move |ev| ev.stop_propagation()
            >
                <div class="modal-head">
                    <div>
                        <p class="eyebrow">"inspace"</p>
                        <h2 id="manage-space-title">{move || format!("{} · {title}", t(locale.get(), "管理空间", "Manage space"))}</h2>
                    </div>
                    <button
                        type="button"
                        class="modal-close"
                        aria-label=move || t(locale.get(), "关闭管理空间", "Close manage space")
                        on:click=move |_| {
                            refresh_spaces();
                            open.set(false);
                        }
                    >
                        "×"
                    </button>
                </div>
                <ManageSpacePanel space=space />
            </section>
        </div>
    }
}

#[component]
fn ManageSpacePanel(space: SpaceMarker) -> impl IntoView {
    let locale = use_i18n().locale;
    let space_id = space.id.clone();
    let related_space_id = space.id.clone();
    let share_space_name = space.name_zh.clone();
    let name_zh = RwSignal::new(space.name_zh.clone());
    let name_en = RwSignal::new(space.name_en.clone().unwrap_or_default());
    let country = RwSignal::new(space.country.clone().unwrap_or_default());
    let province = RwSignal::new(space.province.clone().unwrap_or_default());
    let city = RwSignal::new(space.city.clone().unwrap_or_default());
    let district = RwSignal::new(space.district.clone().unwrap_or_default());
    let spot_name = RwSignal::new(space.spot_name.clone().unwrap_or_default());
    let address_line = RwSignal::new(space.address_line.clone().unwrap_or_default());
    let lat = RwSignal::new(format!("{:.6}", space.lat));
    let lng = RwSignal::new(format!("{:.6}", space.lng));
    let is_public = RwSignal::new(space.is_public);
    // These four are host-editable content that renders on the Space detail
    // cards (简介 / 主理人). We prefill them from the full detail once it loads.
    let custom_type = RwSignal::new(String::new());
    let description_zh = RwSignal::new(String::new());
    let description_en = RwSignal::new(String::new());
    let tag_zh = RwSignal::new(String::new());
    let tag_en = RwSignal::new(String::new());
    let host_bio_zh = RwSignal::new(String::new());
    let host_bio_en = RwSignal::new(String::new());
    let detail_space_id = space.id.clone();
    let detail = Resource::new(
        move || detail_space_id.clone(),
        |space_id| async move { get_space_detail(space_id).await.ok().flatten() },
    );
    Effect::new(move |_| {
        if let Some(Some(d)) = detail.get() {
            custom_type.set(d.custom_type.unwrap_or_default());
            description_zh.set(d.description_zh.unwrap_or_default());
            description_en.set(d.description_en.unwrap_or_default());
            tag_zh.set(d.tag_zh.unwrap_or_default());
            tag_en.set(d.tag_en.unwrap_or_default());
            host_bio_zh.set(d.host_bio_zh.unwrap_or_default());
            host_bio_en.set(d.host_bio_en.unwrap_or_default());
        }
    });
    let message = RwSignal::new(None::<String>);
    let error = RwSignal::new(None::<String>);
    let rotated_password = RwSignal::new(None::<PasswordRotationResult>);
    let confirm_delete = RwSignal::new(false);
    let status = RwSignal::new(space.status.clone());

    Effect::new(move |_| {
        // Mount after the modal is in the DOM; maplibre_shim also retries if missing.
        let lat_value = lat.get().parse::<f64>().unwrap_or(space.lat);
        let lng_value = lng.get().parse::<f64>().unwrap_or(space.lng);
        instant_map_ui::mount(
            "manage-space-map",
            instant_map_ui::MapStyle::Road,
            instant_map_ui::MapProjection::Flat2d,
        );
        instant_map_ui::enable_picker(
            "manage-space-map",
            "space-lat",
            "space-lng",
            lng_value,
            lat_value,
        );
    });

    on_cleanup(move || {
        instant_map_ui::disable_picker("manage-space-map");
        instant_map_ui::destroy("manage-space-map");
    });

    let update = Action::new(move |_: &()| {
        let space_id = space_id.clone();
        let name_zh = name_zh.get();
        let name_en = name_en.get();
        let country = country.get();
        let province = province.get();
        let city = city.get();
        let district = district.get();
        let spot_name = spot_name.get();
        let address_line = address_line.get();
        let lat_value = lat.get().parse::<f64>().unwrap_or(f64::NAN);
        let lng_value = lng.get().parse::<f64>().unwrap_or(f64::NAN);
        let is_public = is_public.get();
        let custom_type = custom_type.get();
        let description_zh = description_zh.get();
        let description_en = description_en.get();
        let tag_zh = tag_zh.get();
        let tag_en = tag_en.get();
        let host_bio_zh = host_bio_zh.get();
        let host_bio_en = host_bio_en.get();
        async move {
            update_my_space(
                space_id,
                name_zh,
                (!name_en.trim().is_empty()).then_some(name_en),
                (!country.trim().is_empty()).then_some(country),
                province,
                city,
                (!district.trim().is_empty()).then_some(district),
                (!spot_name.trim().is_empty()).then_some(spot_name),
                (!address_line.trim().is_empty()).then_some(address_line),
                lat_value,
                lng_value,
                is_public,
                (!custom_type.trim().is_empty()).then_some(custom_type),
                (!description_zh.trim().is_empty()).then_some(description_zh),
                (!description_en.trim().is_empty()).then_some(description_en),
                (!tag_zh.trim().is_empty()).then_some(tag_zh),
                (!tag_en.trim().is_empty()).then_some(tag_en),
                (!host_bio_zh.trim().is_empty()).then_some(host_bio_zh),
                (!host_bio_en.trim().is_empty()).then_some(host_bio_en),
            )
            .await
        }
    });

    let close = Action::new({
        let space_id = space.id.clone();
        move |_: &()| {
            let space_id = space_id.clone();
            async move { close_my_space(space_id).await }
        }
    });
    let reactivate = Action::new({
        let space_id = space.id.clone();
        move |_: &()| {
            let space_id = space_id.clone();
            async move { reactivate_my_space(space_id).await }
        }
    });
    let archive = Action::new({
        let space_id = space.id.clone();
        move |_: &()| {
            let space_id = space_id.clone();
            async move { archive_my_space_template(space_id).await }
        }
    });
    let delete = Action::new({
        let space_id = space.id.clone();
        move |_: &()| {
            let space_id = space_id.clone();
            async move { delete_my_space(space_id).await }
        }
    });
    let apply_resident = Action::new({
        let space_id = space.id.clone();
        move |_: &()| {
            let space_id = space_id.clone();
            async move { apply_my_space_resident(space_id).await }
        }
    });
    let rotate_password = Action::new({
        let space_id = space.id.clone();
        move |_: &()| {
            let space_id = space_id.clone();
            async move { regenerate_space_password(space_id).await }
        }
    });

    Effect::new(move |_| {
        if let Some(result) = update.value().get() {
            handle_space_result(
                result,
                t(locale.get(), "空间信息已保存", "Space updated"),
                message,
                error,
                status,
            );
        }
    });
    Effect::new(move |_| {
        if let Some(result) = close.value().get() {
            handle_space_result(
                result,
                t(
                    locale.get(),
                    "空间已关闭，访客暂时无法进入",
                    "Space closed — visitors can no longer enter",
                ),
                message,
                error,
                status,
            );
        }
    });
    Effect::new(move |_| {
        if let Some(result) = reactivate.value().get() {
            handle_space_result(
                result,
                t(locale.get(), "空间已重新开放", "Space reactivated"),
                message,
                error,
                status,
            );
        }
    });
    Effect::new(move |_| {
        if let Some(result) = archive.value().get() {
            handle_space_result(
                result,
                t(
                    locale.get(),
                    "已存为模板，可用于快速创建同类空间",
                    "Archived as a template for creating similar Spaces",
                ),
                message,
                error,
                status,
            );
        }
    });
    Effect::new(move |_| {
        if let Some(result) = delete.value().get() {
            match result {
                Ok(_) => {
                    error.set(None);
                    message.set(Some(
                        t(locale.get(), "空间已删除", "Space deleted").to_string(),
                    ));
                    confirm_delete.set(false);
                    refresh_spaces();
                }
                Err(err) => {
                    message.set(None);
                    error.set(Some(err.to_string()));
                }
            }
        }
    });
    Effect::new(move |_| {
        if let Some(result) = apply_resident.value().get() {
            match result {
                Ok(()) => {
                    rotated_password.set(None);
                    error.set(None);
                    message.set(Some(
                        t(
                            locale.get(),
                            "驻留申请已提交，等待管理员审核",
                            "Residency request submitted for admin review",
                        )
                        .to_string(),
                    ));
                }
                Err(err) => {
                    message.set(None);
                    error.set(Some(err.to_string()));
                }
            }
        }
    });
    Effect::new(move |_| {
        if let Some(result) = rotate_password.value().get() {
            match result {
                Ok(result) => {
                    message.set(Some(
                        t(
                            locale.get(),
                            "新密码已生成，请立即抄录",
                            "New password generated — copy it now",
                        )
                        .to_string(),
                    ));
                    error.set(None);
                    rotated_password.set(Some(result));
                }
                Err(err) => {
                    message.set(None);
                    rotated_password.set(None);
                    error.set(Some(err.to_string()));
                }
            }
        }
    });

    view! {
        <div class="my-space-panel">
            <section class="password-management-card">
                <div>
                    <strong>{move || t(locale.get(), "空间密码", "Space password")}</strong>
                    <p>
                        {move || t(
                            locale.get(),
                            "旧密码已加密保存，不能反查显示。你可以重置生成一个新密码，新密码会在这里展示一次。",
                            "The old password is encrypted and cannot be shown. Reset it to generate a new password, which will be shown here once."
                        )}
                    </p>
                </div>
                <button class="button button-primary" type="button" on:click=move |_| { rotate_password.dispatch(()); }>
                    {move || t(locale.get(), "重置并显示新密码", "Reset and show new password")}
                </button>
                {move || rotated_password.get().map(|result| view! {
                    <div class="password-result" role="status">
                        <strong>{move || t(locale.get(), "新密码", "New password")}</strong>
                        <code>{result.password}</code>
                        <span>{result.hotspot_name}</span>
                        <small>{format!("version {}", result.password_version)}</small>
                        <div class="community-setup-card">
                            <strong>{move || t(locale.get(), "同步更新热点和社群", "Update hotspot and community")}</strong>
                            <p>{move || t(locale.get(), "请把手机热点名改为上方 InstantSpace_六位数字，并在对应 Discord/QQ 空间讨论组里只发给审核通过的成员。", "Change your phone hotspot name to the InstantSpace_ six-digit value above, and post it only to approved members in the matching Discord/QQ space group.")}</p>
                        </div>
                    </div>
                })}
            </section>

            <form
                class="my-space-edit-form"
                on:submit=move |ev| {
                    ev.prevent_default();
                    update.dispatch(());
                }
            >
                <div class="form-grid">
                    <label class="field-label">
                        <span>{move || t(locale.get(), "中文名称", "Chinese name")}</span>
                        <input aria-label="Manage Chinese name" prop:value=move || name_zh.get() on:input=move |ev| name_zh.set(event_target_value(&ev)) required=true />
                    </label>
                    <label class="field-label">
                        <span>{move || t(locale.get(), "英文名称", "English name")}</span>
                        <input aria-label="Manage English name" prop:value=move || name_en.get() on:input=move |ev| name_en.set(event_target_value(&ev)) />
                    </label>
                    <label class="field-label">
                        <span>{move || t(locale.get(), "公开展示", "Visibility")}</span>
                        <select aria-label="Manage visibility" on:change=move |ev| is_public.set(event_target_value(&ev) == "public")>
                            <option value="public" selected=space.is_public>{move || t(locale.get(), "公开", "Public")}</option>
                            <option value="private" selected=!space.is_public>{move || t(locale.get(), "私密", "Private")}</option>
                        </select>
                    </label>
                </div>
                <div class="form-grid">
                    <label class="field-label">
                        <span>{move || t(locale.get(), "国家", "Country")}</span>
                        <input id="space-country" aria-label="Manage country" prop:value=move || country.get() on:input=move |ev| country.set(event_target_value(&ev)) />
                    </label>
                    <label class="field-label">
                        <span>{move || t(locale.get(), "省份 / 地区", "Province / Region")}</span>
                        <input id="space-province" aria-label="Manage province" prop:value=move || province.get() on:input=move |ev| province.set(event_target_value(&ev)) required=true />
                    </label>
                    <label class="field-label">
                        <span>{move || t(locale.get(), "城市", "City")}</span>
                        <input id="space-city" aria-label="Manage city" prop:value=move || city.get() on:input=move |ev| city.set(event_target_value(&ev)) required=true />
                    </label>
                    <label class="field-label">
                        <span>{move || t(locale.get(), "区域", "District")}</span>
                        <input id="space-district" aria-label="Manage district" prop:value=move || district.get() on:input=move |ev| district.set(event_target_value(&ev)) />
                    </label>
                </div>
                <div class="form-grid">
                    <label class="field-label">
                        <span>{move || t(locale.get(), "具体地点 / 景点", "Specific place / spot")}</span>
                        <input id="space-spot" aria-label="Manage specific place" placeholder=move || t(locale.get(), "例如：观景台 / 入口 / 店名", "e.g. viewpoint / entrance / shop") prop:value=move || spot_name.get() on:input=move |ev| spot_name.set(event_target_value(&ev)) />
                    </label>
                    <label class="field-label">
                        <span>{move || t(locale.get(), "详细地址 / 备注", "Detailed address / note")}</span>
                        <input aria-label="Manage detailed address" placeholder=move || t(locale.get(), "街道、楼层、入口或地标", "Street, floor, entrance, or landmark") prop:value=move || address_line.get() on:input=move |ev| address_line.set(event_target_value(&ev)) />
                    </label>
                </div>
                <div class="form-grid">
                    <label class="field-label">
                        <span>{move || t(locale.get(), "纬度", "Latitude")}</span>
                        <input id="space-lat" aria-label="Manage latitude" prop:value=move || lat.get() on:input=move |ev| lat.set(event_target_value(&ev)) required=true />
                    </label>
                    <label class="field-label">
                        <span>{move || t(locale.get(), "经度", "Longitude")}</span>
                        <input id="space-lng" aria-label="Manage longitude" prop:value=move || lng.get() on:input=move |ev| lng.set(event_target_value(&ev)) required=true />
                    </label>
                </div>
                <div class="my-space-content-fields">
                    <p class="survey-kicker">{move || t(locale.get(), "空间详情内容", "Space detail content")}</p>
                    <p class="my-space-content-hint">{move || t(locale.get(), "这些内容会显示在空间详情页的「简介」和「主理人」卡片上。", "This content appears on the About and Host cards of the Space detail page.")}</p>
                    <div class="form-grid">
                        <label class="field-label">
                            <span>{move || t(locale.get(), "自定义类型（可选）", "Custom type (optional)")}</span>
                            <input aria-label="Manage custom type" placeholder=move || t(locale.get(), "例如：书店 / 露营地 / 展览", "e.g. bookshop / campsite / exhibition") prop:value=move || custom_type.get() on:input=move |ev| custom_type.set(event_target_value(&ev)) />
                        </label>
                        <label class="field-label">
                            <span>{move || t(locale.get(), "标签（中文）", "Tag (Chinese)")}</span>
                            <input aria-label="Manage Chinese tag" placeholder=move || t(locale.get(), "一个短标签，例如：宋代园林", "A short tag, e.g. Song-era garden") prop:value=move || tag_zh.get() on:input=move |ev| tag_zh.set(event_target_value(&ev)) />
                        </label>
                        <label class="field-label">
                            <span>{move || t(locale.get(), "标签（英文）", "Tag (English)")}</span>
                            <input aria-label="Manage English tag" prop:value=move || tag_en.get() on:input=move |ev| tag_en.set(event_target_value(&ev)) />
                        </label>
                    </div>
                    <label class="field-label field-label-wide">
                        <span>{move || t(locale.get(), "简介（中文）", "About (Chinese)")}</span>
                        <textarea aria-label="Manage Chinese description" rows="4" placeholder=move || t(locale.get(), "这是什么地方、为什么建这个空间。", "What this place is, and why this Space exists.") prop:value=move || description_zh.get() on:input=move |ev| description_zh.set(event_target_value(&ev))></textarea>
                    </label>
                    <label class="field-label field-label-wide">
                        <span>{move || t(locale.get(), "简介（英文，可选）", "About (English, optional)")}</span>
                        <textarea aria-label="Manage English description" rows="4" prop:value=move || description_en.get() on:input=move |ev| description_en.set(event_target_value(&ev))></textarea>
                    </label>
                    <label class="field-label field-label-wide">
                        <span>{move || t(locale.get(), "主理人寄语（中文）", "Host note (Chinese)")}</span>
                        <textarea aria-label="Manage Chinese host bio" rows="3" placeholder=move || t(locale.get(), "你和这个地点的关系，想对到访者说的话。", "Your relationship to this place, and a word to visitors.") prop:value=move || host_bio_zh.get() on:input=move |ev| host_bio_zh.set(event_target_value(&ev))></textarea>
                    </label>
                    <label class="field-label field-label-wide">
                        <span>{move || t(locale.get(), "主理人寄语（英文，可选）", "Host note (English, optional)")}</span>
                        <textarea aria-label="Manage English host bio" rows="3" prop:value=move || host_bio_en.get() on:input=move |ev| host_bio_en.set(event_target_value(&ev))></textarea>
                    </label>
                </div>
                <div class="space-picker-panel manage-picker-panel">
                    <div class="space-picker-copy">
                        <strong>{move || t(locale.get(), "重新在地图上选点", "Pick a new point on the map")}</strong>
                        <span>{move || t(locale.get(), "点击或拖动地图选择新位置，会自动更新经纬度和位置层级。保存修改后生效。", "Click or drag the map to choose a new location. Coordinates and location fields update automatically; save to apply.")}</span>
                    </div>
                    <div id="manage-space-map" class="create-space-map manage-space-map" aria-label="Manage space location map">
                        <div class="map-loading" aria-live="polite">
                            <span class="map-loading-dot" aria-hidden="true"></span>
                            <span>{move || t(locale.get(), "正在加载地图", "Loading map")}</span>
                        </div>
                    </div>
                </div>
                <button class="button button-primary" type="submit">
                    {move || t(locale.get(), "保存修改", "Save changes")}
                </button>
            </form>

            <SpaceSharePanel
                space_id=related_space_id.clone()
                space_name=share_space_name
                compact=false
            />

            <SpaceGovernancePanel space_id=related_space_id.clone() />
            <SpaceMembersPanel space_id=related_space_id.clone() />

            <RelatedSpaceGuides space_id=related_space_id />

            <section class="my-space-actions" aria-label=move || t(locale.get(), "空间状态操作", "Space status actions")>
                <div class="my-space-actions-head">
                    <p class="survey-kicker">{move || t(locale.get(), "状态操作", "Status")}</p>
                    <p>{move || t(locale.get(), "以下操作会改变访客能否进入这个空间，攻略内容不会被删除。", "These change whether visitors can enter. Guide content is never deleted here.")}</p>
                </div>
                <div class="my-space-actions-row">
                    <button class="button button-secondary-light" type="button" disabled=move || status.get() == "closed" on:click=move |_| { close.dispatch(()); }>
                        {move || t(locale.get(), "暂停开放", "Pause access")}
                    </button>
                    <button class="button button-secondary-light" type="button" disabled=move || status.get() == "active" on:click=move |_| { reactivate.dispatch(()); }>
                        {move || t(locale.get(), "重新开放", "Reopen")}
                    </button>
                    <button class="button button-secondary-light" type="button" on:click=move |_| { apply_resident.dispatch(()); }>
                        {move || t(locale.get(), "申请长期驻留", "Request residency")}
                    </button>
                    <button class="button button-secondary-light" type="button" on:click=move |_| { archive.dispatch(()); }>
                        {move || t(locale.get(), "存为模板", "Save as template")}
                    </button>
                </div>
                <div class="my-space-danger-row">
                    {move || if confirm_delete.get() {
                        view! {
                            <>
                                <span class="my-space-danger-warning">{move || t(locale.get(), "删除后这个地点的入口和二维码立即失效。", "Deleting retires this place’s entry and QR immediately.")}</span>
                                <button class="button button-danger" type="button" on:click=move |_| { delete.dispatch(()); }>
                                    {move || t(locale.get(), "确认删除", "Confirm delete")}
                                </button>
                                <button class="button button-secondary-light" type="button" on:click=move |_| confirm_delete.set(false)>
                                    {move || t(locale.get(), "取消", "Cancel")}
                                </button>
                            </>
                        }.into_any()
                    } else {
                        view! {
                            <button class="button button-danger-light" type="button" on:click=move |_| confirm_delete.set(true)>
                                {move || t(locale.get(), "删除空间", "Delete Space")}
                            </button>
                        }.into_any()
                    }}
                </div>
            </section>
            {move || message.get().map(|value| view! { <p class="form-success">{value}</p> })}
            {move || error.get().map(|value| view! { <p class="error">{value}</p> })}
        </div>
    }
}

fn governance_role_label(locale: crate::i18n::Locale, role: HostTenureRole) -> &'static str {
    match role {
        HostTenureRole::Primary => t(locale, "主理人", "Primary host"),
        HostTenureRole::CoHost => t(locale, "共同主理人", "Co-host"),
        HostTenureRole::Steward => t(locale, "系统看护", "Steward"),
    }
}

fn governance_state_label(locale: crate::i18n::Locale, state: HostGovernanceState) -> &'static str {
    match state {
        HostGovernanceState::Hosted => t(locale, "有人长期主理", "Hosted"),
        HostGovernanceState::Recruiting => t(locale, "正在招募主理人", "Recruiting"),
        HostGovernanceState::SystemCare => t(locale, "由系统临时看护", "System care"),
    }
}

fn governance_date(value: impl ToString) -> String {
    value.to_string().chars().take(10).collect()
}

fn governance_event_copy(locale: crate::i18n::Locale, event: &SpaceGovernanceEvent) -> String {
    let from = event.from_name.as_deref().unwrap_or("—");
    let to = event.to_name.as_deref().unwrap_or("—");
    match event.action.as_str() {
        "appoint_co_host" => format!(
            "{} {to}",
            t(locale, "共同主理人加入：", "Co-host appointed: ")
        ),
        "appoint_steward" => format!(
            "{} {to}",
            t(locale, "系统看护加入：", "Steward appointed: ")
        ),
        "remove_host" => format!(
            "{} {from}",
            t(locale, "结束协作任期：", "Supporting tenure ended: ")
        ),
        "leave_host" => format!("{} {from}", t(locale, "主动退出任期：", "Host left: ")),
        "transfer_primary" => format!(
            "{from} → {to} · {}",
            t(locale, "完成主理权交接", "Primary stewardship transferred")
        ),
        "release_to_recruiting" => t(
            locale,
            "原主理人退出，空间重新招募",
            "Primary host left; recruitment reopened",
        )
        .to_string(),
        "place_in_system_care" => t(
            locale,
            "空间转为系统临时看护",
            "Space moved into system care",
        )
        .to_string(),
        "update_recruitment_note" => {
            t(locale, "更新了主理人招募说明", "Recruitment note updated").to_string()
        }
        _ => event.action.clone(),
    }
}

#[component]
fn SpaceGovernancePanel(space_id: String) -> impl IntoView {
    let locale = use_i18n().locale;
    let feedback = use_feedback();
    let refresh = RwSignal::new(0u32);
    let resource_id = space_id.clone();
    let governance = Resource::new(
        move || (resource_id.clone(), refresh.get()),
        |(space_id, _)| async move { get_my_space_governance(space_id).await },
    );
    let session = Resource::new(
        || (),
        |_| async move { current_session().await.ok().flatten() },
    );

    let invite_email = RwSignal::new(String::new());
    let invite_note = RwSignal::new(String::new());
    let invite_role = RwSignal::new("co_host".to_string());
    let successor_email = RwSignal::new(String::new());
    let handover_note = RwSignal::new(String::new());
    let recruitment_note = RwSignal::new(String::new());
    let leave_armed = RwSignal::new(false);
    let system_care_armed = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);

    let appoint_id = space_id.clone();
    let appoint = Action::new(move |_: &()| {
        let space_id = appoint_id.clone();
        let email = invite_email.get();
        let role = invite_role.get();
        let note = invite_note.get();
        async move {
            appoint_space_host(
                space_id,
                email,
                role,
                (!note.trim().is_empty()).then_some(note),
            )
            .await
        }
    });

    let remove_id = space_id.clone();
    let remove = Action::new(move |user_id: &String| {
        let space_id = remove_id.clone();
        let user_id = user_id.clone();
        async move { remove_space_host(space_id, user_id, None).await }
    });

    let transfer_id = space_id.clone();
    let transfer = Action::new(move |_: &()| {
        let space_id = transfer_id.clone();
        let email = successor_email.get();
        let note = handover_note.get();
        async move {
            transfer_space_host(space_id, email, (!note.trim().is_empty()).then_some(note)).await
        }
    });

    let leave_id = space_id.clone();
    let leave = Action::new(move |_: &()| {
        let space_id = leave_id.clone();
        async move { leave_space_host_role(space_id, None).await }
    });

    let care_id = space_id.clone();
    let system_care = Action::new(move |_: &()| {
        let space_id = care_id.clone();
        async move { set_space_system_care(space_id, None).await }
    });

    let note_id = space_id.clone();
    let save_note = Action::new(move |_: &()| {
        let space_id = note_id.clone();
        let note = recruitment_note.get();
        async move {
            update_space_recruitment_note(space_id, (!note.trim().is_empty()).then_some(note)).await
        }
    });

    Effect::new(move |_| {
        if let Some(result) = appoint.value().get() {
            match result {
                Ok(()) => {
                    invite_email.set(String::new());
                    invite_note.set(String::new());
                    error.set(None);
                    feedback.success(t(locale.get(), "协作任期已建立", "Host role appointed"));
                    refresh.update(|value| *value += 1);
                }
                Err(err) => error.set(Some(err.to_string())),
            }
        }
    });
    Effect::new(move |_| {
        if let Some(result) = remove.value().get() {
            match result {
                Ok(true) => {
                    error.set(None);
                    feedback.success(t(locale.get(), "协作任期已结束", "Host role ended"));
                    refresh.update(|value| *value += 1);
                }
                Ok(false) => error.set(Some(
                    t(
                        locale.get(),
                        "该任期已结束或不存在",
                        "That tenure is no longer active",
                    )
                    .to_string(),
                )),
                Err(err) => error.set(Some(err.to_string())),
            }
        }
    });
    Effect::new(move |_| {
        if let Some(result) = transfer.value().get() {
            match result {
                Ok(()) => {
                    error.set(None);
                    feedback.success(t(
                        locale.get(),
                        "主理权已完成交接",
                        "Primary stewardship transferred",
                    ));
                    let _ = window().location().set_href("/inspace/my-spaces");
                }
                Err(err) => error.set(Some(err.to_string())),
            }
        }
    });
    Effect::new(move |_| {
        if let Some(result) = leave.value().get() {
            match result {
                Ok(()) => {
                    leave_armed.set(false);
                    error.set(None);
                    feedback.success(t(
                        locale.get(),
                        "你已退出这个空间的主理任期",
                        "You left this host tenure",
                    ));
                    let _ = window().location().set_href("/inspace/my-spaces");
                }
                Err(err) => error.set(Some(err.to_string())),
            }
        }
    });
    Effect::new(move |_| {
        if let Some(result) = system_care.value().get() {
            match result {
                Ok(()) => {
                    system_care_armed.set(false);
                    error.set(None);
                    feedback.success(t(
                        locale.get(),
                        "空间已转为系统临时看护",
                        "Space moved to system care",
                    ));
                    refresh.update(|value| *value += 1);
                }
                Err(err) => error.set(Some(err.to_string())),
            }
        }
    });
    Effect::new(move |_| {
        if let Some(result) = save_note.value().get() {
            match result {
                Ok(()) => {
                    error.set(None);
                    feedback.success(t(locale.get(), "招募说明已保存", "Recruitment note saved"));
                    refresh.update(|value| *value += 1);
                }
                Err(err) => error.set(Some(err.to_string())),
            }
        }
    });

    view! {
        <section class="space-governance-panel" aria-label=move || t(locale.get(), "主理人治理", "Host governance")>
            <div class="card-head-inline space-governance-head">
                <div>
                    <p class="survey-kicker">{move || t(locale.get(), "空间不会随主理人离开而消失", "A Space outlives every host")}</p>
                    <h3>{move || t(locale.get(), "主理人治理", "Host governance")}</h3>
                    <p>{move || t(locale.get(), "管理长期主理、共同主理、交接与任期记录。场景、物件、传送门和出生点只向有效任期开放。", "Manage primary and supporting hosts, handovers and tenure history. Scene objects, portals and spawn points require an active tenure.")}</p>
                </div>
            </div>
            {move || error.get().map(|value| view! { <p class="error" role="alert">{value}</p> })}
            <Suspense fallback=move || view! { <p class="directory-loading">{move || t(locale.get(), "正在读取任期…", "Loading governance…")}</p> }>
                {move || Suspend::new(async move {
                    match governance.await {
                        Ok(snapshot) => {
                            recruitment_note.set(snapshot.recruitment_note.clone().unwrap_or_default());
                            let can_govern = snapshot.can_manage_governance;
                            let current_role = snapshot.current_user_role;
                            let state = snapshot.state;
                            let active = snapshot.active_hosts.clone();
                            let history = snapshot.past_hosts.clone();
                            let events = snapshot.events.clone();
                            let is_admin = session.await.is_some_and(|user| user.role.is_admin());
                            view! {
                                <div class="space-governance-summary">
                                    <span class="space-governance-state">{governance_state_label(locale.get_untracked(), state)}</span>
                                    <span>{format!("{} {}", t(locale.get_untracked(), "当前有效任期", "Active tenures"), active.len())}</span>
                                    <span>{format!("{} {}", t(locale.get_untracked(), "历史任期", "Past tenures"), history.len())}</span>
                                </div>
                                <ul class="space-governance-hosts">
                                    <For
                                        each=move || active.clone()
                                        key=|host| host.tenure_id
                                        children=move |host: SpaceHostIdentity| {
                                            let remove_target = StoredValue::new(host.user_id.to_string());
                                            let removable = can_govern && host.role != HostTenureRole::Primary;
                                            view! {
                                                <li>
                                                    <div>
                                                        <strong>{host.display_name}</strong>
                                                        <span>{governance_role_label(locale.get(), host.role)}</span>
                                                        {host.email.map(|email| view! { <small>{email}</small> })}
                                                    </div>
                                                    <time>{format!("{} {}", t(locale.get(), "始于", "Since"), governance_date(host.started_at))}</time>
                                                    <Show when=move || removable>
                                                        <button class="button button-secondary-light" type="button" on:click=move |_| { remove.dispatch(remove_target.get_value()); }>
                                                            {move || t(locale.get(), "结束任期", "End tenure")}
                                                        </button>
                                                    </Show>
                                                </li>
                                            }
                                        }
                                    />
                                </ul>
                                <Show when=move || can_govern>
                                    <div class="space-governance-actions">
                                        <form on:submit=move |ev| { ev.prevent_default(); appoint.dispatch(()); }>
                                            <div>
                                                <h4>{move || t(locale.get(), "邀请共同主理人", "Invite a co-host")}</h4>
                                                <p>{move || t(locale.get(), "共同主理人可维护简介、内容与场景；只有主理人能转交主理权。", "Co-hosts can maintain content and scenes; only the primary host can transfer stewardship.")}</p>
                                            </div>
                                            <input type="email" required=true placeholder=move || t(locale.get(), "对方的注册邮箱", "Their account email") prop:value=move || invite_email.get() on:input=move |ev| invite_email.set(event_target_value(&ev)) />
                                            <Show when=move || is_admin>
                                                <select prop:value=move || invite_role.get() on:change=move |ev| invite_role.set(event_target_value(&ev))>
                                                    <option value="co_host">{move || t(locale.get(), "共同主理人", "Co-host")}</option>
                                                    <option value="steward">{move || t(locale.get(), "系统看护", "Steward")}</option>
                                                </select>
                                            </Show>
                                            <input placeholder=move || t(locale.get(), "任期说明（可选）", "Tenure note (optional)") prop:value=move || invite_note.get() on:input=move |ev| invite_note.set(event_target_value(&ev)) />
                                            <button class="button button-primary" type="submit" disabled=move || appoint.pending().get()>{move || t(locale.get(), "建立任期", "Appoint")}</button>
                                        </form>
                                        <form on:submit=move |ev| { ev.prevent_default(); transfer.dispatch(()); }>
                                            <div>
                                                <h4>{move || t(locale.get(), "转交主理权", "Transfer stewardship")}</h4>
                                                <p>{move || t(locale.get(), "交接完成后，你的主理任期结束，但空间、故事与历史都会保留。", "Your primary tenure ends after handover, while the Space and all its history remain.")}</p>
                                            </div>
                                            <input type="email" required=true placeholder=move || t(locale.get(), "继任者注册邮箱", "Successor account email") prop:value=move || successor_email.get() on:input=move |ev| successor_email.set(event_target_value(&ev)) />
                                            <input placeholder=move || t(locale.get(), "交接寄语（可选）", "Handover note (optional)") prop:value=move || handover_note.get() on:input=move |ev| handover_note.set(event_target_value(&ev)) />
                                            <button class="button button-secondary" type="submit" disabled=move || transfer.pending().get()>{move || t(locale.get(), "确认交接", "Transfer")}</button>
                                        </form>
                                        <form on:submit=move |ev| { ev.prevent_default(); save_note.dispatch(()); }>
                                            <div>
                                                <h4>{move || t(locale.get(), "主理人招募说明", "Host recruitment note")}</h4>
                                                <p>{move || t(locale.get(), "无主时会公开显示，告诉熟悉这里的人需要承担什么。", "Shown while vacant so applicants understand what care this Space needs.")}</p>
                                            </div>
                                            <textarea rows="3" prop:value=move || recruitment_note.get() on:input=move |ev| recruitment_note.set(event_target_value(&ev))></textarea>
                                            <button class="button button-secondary-light" type="submit" disabled=move || save_note.pending().get()>{move || t(locale.get(), "保存说明", "Save note")}</button>
                                        </form>
                                    </div>
                                </Show>
                                <div class="space-governance-danger">
                                    <Show when=move || current_role.is_some()>
                                        <button class="button button-secondary-light" type="button" on:click=move |_| {
                                            if leave_armed.get() { leave.dispatch(()); } else { leave_armed.set(true); }
                                        }>
                                            {move || if leave_armed.get() { t(locale.get(), "再次点击确认退出", "Click again to leave") } else { t(locale.get(), "退出我的主理任期", "Leave my host role") }}
                                        </button>
                                    </Show>
                                    <Show when=move || is_admin && state != HostGovernanceState::SystemCare>
                                        <button class="button button-danger" type="button" on:click=move |_| {
                                            if system_care_armed.get() { system_care.dispatch(()); } else { system_care_armed.set(true); }
                                        }>
                                            {move || if system_care_armed.get() { t(locale.get(), "再次点击转为系统看护", "Click again for system care") } else { t(locale.get(), "转为系统临时看护", "Move to system care") }}
                                        </button>
                                    </Show>
                                </div>
                                <details class="space-governance-history">
                                    <summary>{move || t(locale.get(), "查看历史主理人与治理记录", "View past hosts and governance log")}</summary>
                                    <div class="space-governance-history-grid">
                                        <section>
                                            <h4>{move || t(locale.get(), "历史任期", "Past tenures")}</h4>
                                            <ul>
                                                <For
                                                    each=move || history.clone()
                                                    key=|host| host.tenure_id
                                                    children=move |host: SpaceHostIdentity| view! {
                                                        <li>
                                                            <strong>{host.display_name}</strong>
                                                            <span>{governance_role_label(locale.get(), host.role)}</span>
                                                            <small>{format!("{} — {}", governance_date(host.started_at), host.ended_at.map(governance_date).unwrap_or_else(|| "—".to_string()))}</small>
                                                        </li>
                                                    }
                                                />
                                            </ul>
                                        </section>
                                        <section>
                                            <h4>{move || t(locale.get(), "治理记录", "Governance log")}</h4>
                                            <ul>
                                                <For
                                                    each=move || events.clone()
                                                    key=|event| event.id
                                                    children=move |event: SpaceGovernanceEvent| {
                                                        let copy = governance_event_copy(locale.get(), &event);
                                                        view! {
                                                            <li>
                                                                <strong>{copy}</strong>
                                                                <span>{event.actor_name.unwrap_or_else(|| t(locale.get(), "系统", "System").to_string())}</span>
                                                                <small>{governance_date(event.created_at)}</small>
                                                            </li>
                                                        }
                                                    }
                                                />
                                            </ul>
                                        </section>
                                    </div>
                                </details>
                            }.into_any()
                        }
                        Err(err) => view! { <p class="error">{err.to_string()}</p> }.into_any(),
                    }
                })}
            </Suspense>
        </section>
    }
}

#[component]
fn SpaceMembersPanel(space_id: String) -> impl IntoView {
    let locale = use_i18n().locale;
    let feedback = use_feedback();
    let refresh = RwSignal::new(0u32);
    let members_id = space_id.clone();
    let add_id = space_id.clone();
    let remove_id = space_id.clone();
    let member_email = RwSignal::new(String::new());
    let member_role = RwSignal::new("member".to_string());
    let add_error = RwSignal::new(None::<String>);

    let members = Resource::new(
        move || (members_id.clone(), refresh.get()),
        |(space_id, _)| async move { list_my_space_members(space_id).await.unwrap_or_default() },
    );

    let add_member = Action::new(move |_: &()| {
        let space_id = add_id.clone();
        let email = member_email.get().trim().to_string();
        let role = member_role.get();
        async move { add_my_space_member(space_id, email, role).await }
    });

    let remove_member = Action::new(move |email: &String| {
        let space_id = remove_id.clone();
        let email = email.clone();
        async move { remove_my_space_member(space_id, email).await }
    });

    Effect::new(move |_| {
        if let Some(result) = add_member.value().get() {
            match result {
                Ok(member) => {
                    member_email.set(String::new());
                    add_error.set(None);
                    feedback.success(format!(
                        "已加入成员 {}（{}）",
                        member.display_name.as_deref().unwrap_or(""),
                        member.email
                    ));
                    refresh.update(|value| *value += 1);
                }
                Err(err) => add_error.set(Some(err.to_string())),
            }
        }
    });

    Effect::new(move |_| {
        if let Some(result) = remove_member.value().get() {
            match result {
                Ok(true) => {
                    feedback.success("成员已移除");
                    refresh.update(|value| *value += 1);
                }
                Ok(false) => feedback.info("该邮箱不是此空间成员"),
                Err(err) => add_error.set(Some(err.to_string())),
            }
        }
    });

    view! {
        <section class="space-members-panel" aria-label=move || t(locale.get(), "空间成员", "Space members")>
            <div class="card-head-inline">
                <div>
                    <h3>{move || t(locale.get(), "空间成员", "Space members")}</h3>
                    <p>
                        {move || t(
                            locale.get(),
                            "按邮箱邀请普通成员。主理人与共同主理人请在上方「主理人治理」中建立正式任期。",
                            "Invite regular members here. Create primary and co-host tenures in Host governance above."
                        )}
                    </p>
                </div>
            </div>

            {move || add_error.get().map(|value| view! { <p class="error">{value}</p> })}

            <div class="space-members-add">
                <input
                    type="email"
                    placeholder=move || t(locale.get(), "成员邮箱", "Member email")
                    prop:value=move || member_email.get()
                    on:input=move |ev| member_email.set(event_target_value(&ev))
                />
                <input type="hidden" value="member" />
                <button class="button button-primary" type="button" on:click=move |_| { add_member.dispatch(()); }>
                    {move || t(locale.get(), "添加成员", "Add member")}
                </button>
            </div>

            <ul class="space-members-list">
                <Suspense fallback=move || view! { <li class="directory-loading">{move || t(locale.get(), "加载成员…", "Loading members…")}</li> }>
                    {move || Suspend::new(async move {
                        let items = members.await;
                        view! {
                            <For
                                each=move || items.clone()
                                key=|member| member.user_id
                                children=move |member: instant_domain::spaces::SpaceMember| {
                                    let email = member.email.clone();
                                    let name = member.display_name.clone().unwrap_or_default();
                                    let role_zh = if member.role == "host" {
                                        t(locale.get(), "主持人", "Host").to_string()
                                    } else {
                                        t(locale.get(), "成员", "Member").to_string()
                                    };
                                    view! {
                                        <li class="space-member-row">
                                            <span class="space-member-name">{name}</span>
                                            <span class="space-member-email">{email.clone()}</span>
                                            <span class="space-member-role">{role_zh}</span>
                                            <button
                                                class="button button-secondary-light"
                                                type="button"
                                                on:click=move |_| { remove_member.dispatch(email.clone()); }
                                            >
                                                {move || t(locale.get(), "移除", "Remove")}
                                            </button>
                                        </li>
                                    }
                                }
                            />
                        }
                    })}
                </Suspense>
            </ul>
        </section>
    }
}

#[component]
fn RelatedSpaceGuides(space_id: String) -> impl IntoView {
    let locale = use_i18n().locale;
    let refresh_guides = RwSignal::new(0u32);
    let guide_message = RwSignal::new(None::<String>);
    let guide_error = RwSignal::new(None::<String>);
    let write_href = format!("/inspace/guides/new?space_id={space_id}");
    let space_id_for_resource = space_id.clone();

    let guides = Resource::new(
        move || (space_id_for_resource.clone(), refresh_guides.get()),
        |(space_id, _)| async move {
            list_manageable_space_guides(space_id)
                .await
                .unwrap_or_default()
        },
    );

    // Deleting is destructive, so the first click only arms the row.
    let pending_delete = RwSignal::new(None::<String>);
    let delete_action = Action::new(move |guide_id: &String| {
        let guide_id = guide_id.clone();
        async move { delete_guide(guide_id).await }
    });

    Effect::new(move |_| {
        if let Some(result) = delete_action.value().get() {
            match result {
                Ok(_) => {
                    guide_error.set(None);
                    pending_delete.set(None);
                    guide_message.set(Some(
                        t(locale.get(), "攻略已删除", "Guide deleted").to_string(),
                    ));
                    refresh_guides.update(|value| *value += 1);
                }
                Err(err) => {
                    guide_message.set(None);
                    guide_error.set(Some(err.to_string()));
                }
            }
        }
    });

    view! {
        <section class="related-space-guides" aria-label=move || t(locale.get(), "相关攻略", "Related guides")>
            <div class="card-head-inline">
                <div>
                    <h3>{move || t(locale.get(), "相关攻略", "Related guides")}</h3>
                    <p>
                        {move || t(
                            locale.get(),
                            "一个空间可挂多篇攻略。写新攻略会新增；编辑已有攻略会更新同一篇，不会重复创建。",
                            "One space can have multiple guides. Writing creates a new guide; editing updates the same guide."
                        )}
                    </p>
                </div>
                <a class="button button-primary" href=write_href>
                    {move || t(locale.get(), "写新攻略", "Write guide")}
                </a>
            </div>

            {move || guide_message.get().map(|value| view! { <p class="form-success">{value}</p> })}
            {move || guide_error.get().map(|value| view! { <p class="error">{value}</p> })}

            <Suspense fallback=move || view! { <p class="muted">{move || t(locale.get(), "正在加载相关攻略…", "Loading related guides…")}</p> }>
                {move || Suspend::new(async move {
                    let items = guides.await;
                    if items.is_empty() {
                        view! {
                            <div class="empty-state compact-empty">
                                <span>{move || t(locale.get(), "这个空间还没有攻略。点击右上角「写新攻略」开始。", "No guides for this space yet. Use Write guide to start.")}</span>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <ul class="guide-list space-guide-list related-guide-list">
                                <For
                                    each=move || items.clone()
                                    key=|guide| guide.id
                                    children=move |guide| {
                                        let title_zh = guide.title_zh.clone();
                                        let title_en = guide.title_en.clone();
                                        let status_class = guide_status_class(guide.status);
                                        let status_label_zh = guide_status_label_zh(guide.status);
                                        let status_label_en = guide_status_label_en(guide.status);
                                        let can_edit = guide.can_edit;
                                        let view_href = format!("/inspace/guides/{}", guide.id);
                                        let edit_href = format!("/inspace/guides/{}/edit", guide.id);
                                        let guide_id = guide.id.to_string();
                                        let delete = delete_action;
                                        view! {
                                            <li class="related-guide-item">
                                                <div class="related-guide-main">
                                                    <a class="guide-list-link" href=view_href>
                                                        <strong>{move || localize_optional(locale.get(), &title_zh, title_en.as_deref())}</strong>
                                                    </a>
                                                    <span class=status_class>
                                                        {move || t(locale.get(), status_label_zh, status_label_en)}
                                                    </span>
                                                </div>
                                                {
                                                    if can_edit {
                                                        let guide_id = guide_id.clone();
                                                        view! {
                                                            <div class="guide-list-actions">
                                                                <a class="button button-secondary-light" href=edit_href>
                                                                    {move || t(locale.get(), "编辑", "Edit")}
                                                                </a>
                                                                {
                                                                    let arm_id = guide_id.clone();
                                                                    let confirm_id = guide_id.clone();
                                                                    let is_armed = {
                                                                        let armed_id = guide_id.clone();
                                                                        move || pending_delete.get().as_deref() == Some(armed_id.as_str())
                                                                    };
                                                                    let is_armed_label = is_armed.clone();
                                                                    view! {
                                                                        <button
                                                                            type="button"
                                                                            class="button button-danger-light"
                                                                            on:click=move |_| {
                                                                                if pending_delete.get().as_deref() == Some(confirm_id.as_str()) {
                                                                                    delete.dispatch(confirm_id.clone());
                                                                                } else {
                                                                                    pending_delete.set(Some(arm_id.clone()));
                                                                                }
                                                                            }
                                                                        >
                                                                            {move || if is_armed_label() {
                                                                                t(locale.get(), "确认删除", "Confirm delete")
                                                                            } else {
                                                                                t(locale.get(), "删除", "Delete")
                                                                            }}
                                                                        </button>
                                                                    }
                                                                }
                                                            </div>
                                                        }.into_any()
                                                    } else {
                                                        view! { <span class="related-guide-actions-empty"></span> }.into_any()
                                                    }
                                                }
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

fn guide_status_label_zh(status: GuideStatus) -> &'static str {
    match status {
        GuideStatus::Draft => "草稿",
        GuideStatus::Published => "已发布",
        GuideStatus::Archived => "已归档",
    }
}

fn guide_status_label_en(status: GuideStatus) -> &'static str {
    match status {
        GuideStatus::Draft => "Draft",
        GuideStatus::Published => "Published",
        GuideStatus::Archived => "Archived",
    }
}

fn guide_status_class(status: GuideStatus) -> &'static str {
    match status {
        GuideStatus::Draft => "space-badge space-badge-private",
        GuideStatus::Published => "space-badge space-badge-public",
        GuideStatus::Archived => "space-badge",
    }
}

/// Space mutations always refresh the workspace list so the card badges
/// (status, visibility) match what the modal just changed.
fn handle_space_result(
    result: Result<SpaceMarker, ServerFnError>,
    success: &str,
    message: RwSignal<Option<String>>,
    error: RwSignal<Option<String>>,
    status: RwSignal<String>,
) {
    match result {
        Ok(updated) => {
            // Keep the modal mounted so the user can see the result and continue
            // working. The outer workspace list refreshes when the modal closes.
            status.set(updated.status.clone());
            error.set(None);
            message.set(Some(success.to_string()));
        }
        Err(err) => {
            message.set(None);
            error.set(Some(err.to_string()));
        }
    }
}

fn location_label(space: &SpaceMarker) -> String {
    [
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
    .collect::<Vec<_>>()
    .join(" / ")
}
