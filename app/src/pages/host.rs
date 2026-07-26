use leptos::prelude::*;

use crate::app_state::{refresh_spaces, use_app_refresh_state};
use crate::components::space_form::OpenCreateSpaceButton;
use crate::components::space_share::SpaceSharePanel;
use crate::i18n::{localize_optional, t, use_i18n};
use crate::server::auth::current_session;
use crate::server::guides::{delete_guide, list_manageable_space_guides};
use crate::server::spaces::{
    apply_my_space_resident, archive_my_space_template, close_my_space, delete_my_space,
    list_my_spaces, reactivate_my_space, regenerate_space_password, update_my_space,
    PasswordRotationResult, SpaceMarker,
};
use instant_domain::guides::GuideStatus;

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
        view! {
            <section class="empty-state">
                <strong>{move || t(locale.get(), "还没有空间", "No spaces yet")}</strong>
                <span>{move || t(locale.get(), "点击创建空间，把一个景点、餐馆、街区或活动地点变成可扫码分享的数字入口。", "Create a Space to turn a scenic spot, restaurant, district, or event place into a QR-shareable digital entry.")}</span>
            </section>
        }
        .into_any()
    } else {
        view! {
            <section class="my-space-grid" aria-label="My spaces">
                <For
                    each=move || items.clone()
                    key=|space| space.id.clone()
                    children=move |space| view! { <MySpaceCard space=space /> }
                />
            </section>
        }
        .into_any()
    }
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
    let space_href = format!("/inspace/spaces/{}", space.id);
    let chat_href = format!("/inspace/spaces/{}/chat", space.id);
    let write_guide_href = format!("/inspace/guides/new?space_id={}", space.id);

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
                <a class="button button-secondary-light" href=space_href>{move || t(locale.get(), "打开空间", "Open Space")}</a>
                <a class="button button-secondary-light" href=write_guide_href>{move || t(locale.get(), "写攻略", "Write guide")}</a>
                <a class="button button-secondary-light" href=chat_href>{move || t(locale.get(), "讨论区", "Discussion")}</a>
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
            handle_space_result(result, t(locale.get(), "空间信息已保存", "Space updated"), message, error, status);
        }
    });
    Effect::new(move |_| {
        if let Some(result) = close.value().get() {
            handle_space_result(result, t(locale.get(), "空间已关闭，访客暂时无法进入", "Space closed — visitors can no longer enter"), message, error, status);
        }
    });
    Effect::new(move |_| {
        if let Some(result) = reactivate.value().get() {
            handle_space_result(result, t(locale.get(), "空间已重新开放", "Space reactivated"), message, error, status);
        }
    });
    Effect::new(move |_| {
        if let Some(result) = archive.value().get() {
            handle_space_result(result, t(locale.get(), "已存为模板，可用于快速创建同类空间", "Archived as a template for creating similar Spaces"), message, error, status);
        }
    });
    Effect::new(move |_| {
        if let Some(result) = delete.value().get() {
            match result {
                Ok(_) => {
                    error.set(None);
                    message.set(Some(t(locale.get(), "空间已删除", "Space deleted").to_string()));
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
                    message.set(Some(t(locale.get(), "驻留申请已提交，等待管理员审核", "Residency request submitted for admin review").to_string()));
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
                    message.set(Some(t(locale.get(), "新密码已生成，请立即抄录", "New password generated — copy it now").to_string()));
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
