use instant_domain::spaces::SpaceType;
use instant_map_ui::{MapProjection, MapStyle};
use leptos::prelude::*;

use crate::app_state::refresh_spaces;
use crate::i18n::{t, use_i18n};
use crate::server::auth::current_session;
use crate::server::geo::{
    list_geo_cities, list_geo_countries, list_geo_districts, list_geo_regions,
};
use crate::server::guides::{bind_guide_to_space, list_bindable_guides};
use crate::server::spaces::create_space;

#[derive(Clone, Copy)]
pub struct CreateSpaceModalState {
    pub open: RwSignal<bool>,
}

pub fn provide_create_space_modal() -> CreateSpaceModalState {
    let state = CreateSpaceModalState {
        open: RwSignal::new(false),
    };
    provide_context(state);
    state
}

pub fn use_create_space_modal() -> Option<CreateSpaceModalState> {
    use_context::<CreateSpaceModalState>()
}

#[component]
pub fn CreateSpaceModalHost() -> impl IntoView {
    let locale = use_i18n().locale;
    let state = use_create_space_modal().unwrap_or_else(provide_create_space_modal);
    let session = Resource::new(
        move || state.open.get(),
        |is_open| async move {
            if is_open {
                current_session().await.ok().flatten()
            } else {
                None
            }
        },
    );

    view! {
        {move || state.open.get().then(|| view! {
            <div class="modal-backdrop" role="presentation">
                <section
                    class="modal-card create-space-modal"
                    role="dialog"
                    aria-modal="true"
                    aria-labelledby="create-space-title"
                >
                    <div class="modal-head">
                        <div>
                            <p class="eyebrow">"inspace"</p>
                            <h2 id="create-space-title">{move || t(locale.get(), "创建空间", "Create Space")}</h2>
                        </div>
                        <button
                            type="button"
                            class="modal-close"
                            aria-label=move || t(locale.get(), "关闭创建空间", "Close create space")
                            on:click=move |_| state.open.set(false)
                        >
                            "×"
                        </button>
                    </div>
                    <Suspense fallback=move || view! { <p>{move || t(locale.get(), "正在检查登录状态", "Checking session")}</p> }>
                        {move || Suspend::new(async move {
                            if session.await.is_some() {
                                view! { <SpaceForm /> }.into_any()
                            } else {
                                view! {
                                    <div class="empty-state">
                                        <strong>{move || t(locale.get(), "请先登录", "Sign in first")}</strong>
                                        <span>{move || t(locale.get(), "登录后可以创建空间，并在我的空间中管理。", "Sign in to create spaces and manage them in My Spaces.")}</span>
                                        <a
                                            class="button button-primary"
                                            href="/inspace/login"
                                            on:click=move |_| state.open.set(false)
                                        >
                                            {move || t(locale.get(), "去登录", "Go to sign in")}
                                        </a>
                                    </div>
                                }.into_any()
                            }
                        })}
                    </Suspense>
                </section>
            </div>
        })}
    }
}

#[component]
pub fn OpenCreateSpaceButton(
    #[prop(default = "button button-primary")] class: &'static str,
) -> impl IntoView {
    let locale = use_i18n().locale;
    let state = use_create_space_modal().unwrap_or_else(provide_create_space_modal);
    view! {
        <button
            type="button"
            class=class
            on:click=move |_| state.open.set(true)
        >
            {move || t(locale.get(), "创建空间", "Create Space")}
        </button>
    }
}

#[component]
pub fn SpaceForm() -> impl IntoView {
    let locale = use_i18n().locale;
    let name_zh = RwSignal::new(String::new());
    let name_en = RwSignal::new(String::new());
    let country = RwSignal::new(String::new());
    let province = RwSignal::new(String::new());
    let city = RwSignal::new(String::new());
    let district = RwSignal::new(String::new());
    let spot_name = RwSignal::new(String::new());
    let address_line = RwSignal::new(String::new());
    let space_type = RwSignal::new("custom".to_string());
    let custom_type = RwSignal::new(String::new());
    let description_zh = RwSignal::new(String::new());
    let description_en = RwSignal::new(String::new());
    let tag_zh = RwSignal::new(String::new());
    let tag_en = RwSignal::new(String::new());
    let lat = RwSignal::new("31.2304".to_string());
    let lng = RwSignal::new("121.4737".to_string());
    let duration_hours = RwSignal::new("24".to_string());
    let is_public = RwSignal::new(true);
    let created_name = RwSignal::new(None::<String>);
    let created_password = RwSignal::new(None::<String>);
    let created_hotspot = RwSignal::new(None::<String>);
    let created_space_id = RwSignal::new(None::<String>);
    let bound_guide = RwSignal::new(None::<String>);
    let bind_guide_id = RwSignal::new(String::new());
    let bindable_guides = Resource::new(
        || (),
        |_| async { list_bindable_guides().await.unwrap_or_default() },
    );
    let countries = Resource::new(
        || (),
        |_| async { list_geo_countries().await.unwrap_or_default() },
    );
    let regions = Resource::new(
        move || country.get(),
        |country| async move { list_geo_regions(country).await.unwrap_or_default() },
    );
    let cities = Resource::new(
        move || (country.get(), province.get()),
        |(country, province)| async move {
            list_geo_cities(country, (!province.trim().is_empty()).then_some(province))
                .await
                .unwrap_or_default()
        },
    );
    let districts = Resource::new(
        move || (country.get(), province.get(), city.get()),
        |(country, province, city)| async move {
            list_geo_districts(
                country,
                (!province.trim().is_empty()).then_some(province),
                (!city.trim().is_empty()).then_some(city),
            )
            .await
            .unwrap_or_default()
        },
    );
    let error = RwSignal::new(None::<String>);
    let create = Action::new(move |_: &()| {
        let name_zh = name_zh.get();
        let name_en = name_en.get();
        let country = country.get();
        let province = province.get();
        let city = city.get();
        let district = district.get();
        let spot_name = spot_name.get();
        let address_line = address_line.get();
        let selected_space_type = parse_space_type(&space_type.get());
        let custom_type = custom_type.get();
        let description_zh = description_zh.get();
        let description_en = description_en.get();
        let tag_zh = tag_zh.get();
        let tag_en = tag_en.get();
        let lat_value = lat.get().parse::<f64>().unwrap_or(f64::NAN);
        let lng_value = lng.get().parse::<f64>().unwrap_or(f64::NAN);
        let duration_hours = duration_hours.get().parse::<i32>().unwrap_or(24);
        let is_public = is_public.get();
        async move {
            create_space(
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
                selected_space_type,
                (!custom_type.trim().is_empty()).then_some(custom_type),
                (!description_zh.trim().is_empty()).then_some(description_zh),
                (!description_en.trim().is_empty()).then_some(description_en),
                (!tag_zh.trim().is_empty()).then_some(tag_zh),
                (!tag_en.trim().is_empty()).then_some(tag_en),
                is_public,
                duration_hours,
            )
            .await
        }
    });

    let bind = Action::new(move |_: &()| {
        let guide_id = bind_guide_id.get();
        let space_id = created_space_id.get().unwrap_or_default();
        async move {
            if guide_id.trim().is_empty() || space_id.trim().is_empty() {
                return Err(leptos::prelude::ServerFnError::new("请选择要关联的攻略"));
            }
            bind_guide_to_space(guide_id, space_id).await
        }
    });

    Effect::new(move |_| {
        if let Some(result) = bind.value().get() {
            match result {
                Ok(guide) => {
                    bound_guide.set(Some(guide.title_zh));
                    error.set(None);
                }
                Err(err) => error.set(Some(err.to_string())),
            }
        }
    });

    Effect::new(move |_| {
        instant_map_ui::mount("create-space-map", MapStyle::Road, MapProjection::Flat2d);
        let lat_value = lat.get().parse::<f64>().unwrap_or(31.2304);
        let lng_value = lng.get().parse::<f64>().unwrap_or(121.4737);
        instant_map_ui::enable_picker(
            "create-space-map",
            "space-lat",
            "space-lng",
            lng_value,
            lat_value,
        );
    });

    on_cleanup(move || {
        instant_map_ui::disable_picker("create-space-map");
        instant_map_ui::destroy("create-space-map");
    });

    Effect::new(move |_| {
        if let Some(result) = create.value().get() {
            match result {
                Ok(space) => {
                    created_space_id.set(Some(space.id.clone()));
                    created_name.set(Some(space.name_zh));
                    created_password.set(space.generated_password);
                    created_hotspot.set(space.hotspot_name);
                    error.set(None);
                    refresh_spaces();
                }
                Err(err) => {
                    created_name.set(None);
                    created_password.set(None);
                    created_hotspot.set(None);
                    error.set(Some(err.to_string()));
                }
            }
        }
    });

    view! {
        <form
            class="space-form"
            on:submit=move |ev| {
                ev.prevent_default();
                create.dispatch(());
            }
        >
            <div class="space-form-grid">
                <div class="space-form-fields">
                    <label class="field-label">
                        <span>{move || t(locale.get(), "中文名称", "Chinese name")}</span>
                        <input name="name_zh" aria-label="Chinese name" placeholder=move || t(locale.get(), "例如：上海外滩日落点", "e.g. Shanghai Bund sunset spot") required=true on:input=move |ev| name_zh.set(event_target_value(&ev)) />
                    </label>
                    <label class="field-label">
                        <span>{move || t(locale.get(), "英文名称", "English name")}</span>
                        <input name="name_en" aria-label="English name" placeholder=move || t(locale.get(), "例如：Bund Sunset Spot", "e.g. Bund Sunset Spot") on:input=move |ev| name_en.set(event_target_value(&ev)) />
                    </label>
                    <div class="form-grid">
                        <label class="field-label">
                            <span>{move || t(locale.get(), "空间类型", "Space type")}</span>
                            <select
                                name="space_type"
                                aria-label="Space type"
                                on:change=move |ev| space_type.set(event_target_value(&ev))
                            >
                                <option value="custom" selected=true>{move || t(locale.get(), "自定义", "Custom")}</option>
                                <option value="scenic">{move || t(locale.get(), "景点", "Scenic")}</option>
                                <option value="food">{move || t(locale.get(), "美食", "Food")}</option>
                                <option value="park">{move || t(locale.get(), "公园", "Park")}</option>
                                <option value="transit">{move || t(locale.get(), "交通", "Transit")}</option>
                                <option value="event">{move || t(locale.get(), "活动", "Event")}</option>
                            </select>
                        </label>
                        <label class="field-label">
                            <span>{move || t(locale.get(), "自定义类型", "Custom type")}</span>
                            <input name="custom_type" aria-label="Custom type" placeholder=move || t(locale.get(), "例如：徒步集合点 / 摄影机位", "e.g. meetup point / photo spot") on:input=move |ev| custom_type.set(event_target_value(&ev)) />
                        </label>
                        <label class="field-label">
                            <span>{move || t(locale.get(), "有效期（小时）", "Duration hours")}</span>
                            <input name="duration_hours" type="number" min="1" max="720" aria-label="Duration hours" placeholder="24" prop:value=move || duration_hours.get() required=true on:input=move |ev| duration_hours.set(event_target_value(&ev)) />
                        </label>
                    </div>
                    <div class="form-grid">
                        <label class="field-label">
                            <span>{move || t(locale.get(), "国家", "Country")}</span>
                            <input id="space-country" list="country-options" name="country" aria-label="Country" placeholder=move || t(locale.get(), "选择或输入国家，例如：中国", "Select or type a country, e.g. China") prop:value=move || country.get() on:input=move |ev| country.set(event_target_value(&ev)) />
                            <datalist id="country-options">
                                {move || countries.get().unwrap_or_default().into_iter().map(|item| view! { <option value=item.value>{item.label}</option> }).collect_view()}
                            </datalist>
                        </label>
                        <label class="field-label">
                            <span>{move || t(locale.get(), "省份 / 地区", "Province / Region")}</span>
                            <input id="space-province" list="province-options" name="province" aria-label="Province" placeholder=move || t(locale.get(), "省 / 州 / 大区，例如：上海市", "Province / state / region, e.g. Shanghai") prop:value=move || province.get() required=true on:input=move |ev| province.set(event_target_value(&ev)) />
                            <datalist id="province-options">
                                {move || regions.get().unwrap_or_default().into_iter().map(|item| view! { <option value=item.value>{item.label}</option> }).collect_view()}
                            </datalist>
                        </label>
                        <label class="field-label">
                            <span>{move || t(locale.get(), "城市", "City")}</span>
                            <input id="space-city" list="city-options" name="city" aria-label="City" placeholder=move || t(locale.get(), "城市 / 镇，例如：上海市", "City / town, e.g. Shanghai") prop:value=move || city.get() required=true on:input=move |ev| city.set(event_target_value(&ev)) />
                            <datalist id="city-options">
                                {move || cities.get().unwrap_or_default().into_iter().map(|item| view! { <option value=item.value>{item.label}</option> }).collect_view()}
                            </datalist>
                        </label>
                        <label class="field-label">
                            <span>{move || t(locale.get(), "区域", "District")}</span>
                            <input id="space-district" list="district-options" name="district" aria-label="District" placeholder=move || t(locale.get(), "区 / 县 / County，例如：黄浦区", "District / county, e.g. Huangpu") prop:value=move || district.get() on:input=move |ev| district.set(event_target_value(&ev)) />
                            <datalist id="district-options">
                                {move || districts.get().unwrap_or_default().into_iter().map(|item| view! { <option value=item.value>{item.label}</option> }).collect_view()}
                            </datalist>
                        </label>
                    </div>
                    <div class="form-grid">
                        <label class="field-label">
                            <span>{move || t(locale.get(), "具体地点 / 景点", "Specific place / spot")}</span>
                            <input id="space-spot" name="spot_name" aria-label="Specific place" placeholder=move || t(locale.get(), "例如：外滩观景平台 / 3 号口", "e.g. viewing platform / Exit 3") prop:value=move || spot_name.get() on:input=move |ev| spot_name.set(event_target_value(&ev)) />
                        </label>
                        <label class="field-label">
                            <span>{move || t(locale.get(), "详细地址 / 备注", "Detailed address / note")}</span>
                            <input id="space-address" name="address_line" aria-label="Detailed address" placeholder=move || t(locale.get(), "例如：中山东一路，靠近南京东路", "e.g. street address, entrance, floor, landmark") prop:value=move || address_line.get() on:input=move |ev| address_line.set(event_target_value(&ev)) />
                        </label>
                    </div>
                    <div class="form-grid coordinate-grid">
                        <label class="field-label">
                            <span>{move || t(locale.get(), "纬度", "Latitude")}</span>
                            <input id="space-lat" name="lat" aria-label="Latitude" placeholder="31.230400" prop:value=move || lat.get() required=true on:input=move |ev| lat.set(event_target_value(&ev)) />
                        </label>
                        <label class="field-label">
                            <span>{move || t(locale.get(), "经度", "Longitude")}</span>
                            <input id="space-lng" name="lng" aria-label="Longitude" placeholder="121.473700" prop:value=move || lng.get() required=true on:input=move |ev| lng.set(event_target_value(&ev)) />
                        </label>
                    </div>
                    <div class="form-grid">
                        <label class="field-label">
                            <span>{move || t(locale.get(), "中文标签", "Chinese tag")}</span>
                            <input name="tag_zh" aria-label="Chinese tag" placeholder=move || t(locale.get(), "例如：日落, 摄影, 城市漫步", "e.g. sunset, photo, city walk") on:input=move |ev| tag_zh.set(event_target_value(&ev)) />
                        </label>
                        <label class="field-label">
                            <span>{move || t(locale.get(), "英文标签", "English tag")}</span>
                            <input name="tag_en" aria-label="English tag" placeholder=move || t(locale.get(), "例如：sunset, photo, city walk", "e.g. sunset, photo, city walk") on:input=move |ev| tag_en.set(event_target_value(&ev)) />
                        </label>
                    </div>
                    <label class="field-label">
                        <span>{move || t(locale.get(), "中文描述", "Chinese description")}</span>
                        <textarea name="description_zh" aria-label="Chinese description" placeholder=move || t(locale.get(), "写给游客看的中文攻略提示，例如最佳时间、入口、注意事项。", "Chinese guide notes: best time, entrance, tips.") rows="3" on:input=move |ev| description_zh.set(event_target_value(&ev))></textarea>
                    </label>
                    <label class="field-label">
                        <span>{move || t(locale.get(), "英文描述", "English description")}</span>
                        <textarea name="description_en" aria-label="English description" placeholder=move || t(locale.get(), "English guide notes for global visitors.", "English guide notes for global visitors.") rows="3" on:input=move |ev| description_en.set(event_target_value(&ev))></textarea>
                    </label>
                    <p class="form-hint">
                        {move || t(locale.get(), "空间密码将由系统自动生成，创建成功后只展示一次。", "The space password is generated automatically and shown once after creation.")}
                    </p>
                    <label class="check-row">
                        <input type="checkbox" checked=true on:change=move |ev| is_public.set(event_target_checked(&ev)) />
                        <span>{move || t(locale.get(), "公开展示", "Publicly visible")}</span>
                    </label>
                </div>
                <div class="space-picker-panel">
                    <div class="space-picker-copy">
                        <strong>{move || t(locale.get(), "在地图上选点", "Pick a point on the map")}</strong>
                        <span>{move || t(locale.get(), "点击地图会自动填写经纬度和位置层级；拖动地图可移动视野。", "Click the map to fill coordinates and location fields; drag to move the map.")}</span>
                    </div>
                    <div id="create-space-map" class="create-space-map" aria-label="Pick space location map">
                        <div class="map-loading" aria-live="polite">
                            <span class="map-loading-dot" aria-hidden="true"></span>
                            <span>{move || t(locale.get(), "正在加载地图", "Loading map")}</span>
                        </div>
                    </div>
                </div>
            </div>
            <div class="form-actions">
                <button class="button button-primary" type="submit">
                    {move || t(locale.get(), "创建", "Create")}
                </button>
                {move || created_name.get().map(|name| view! {
                    <p class="form-success">{format!("{} {name}", t(locale.get(), "已创建：", "Created:"))}</p>
                })}
                {move || created_password.get().map(|password| {
                    let hotspot = created_hotspot.get().unwrap_or_default();
                    let space_name = created_name.get().unwrap_or_default();
                    view! {
                        <div class="password-result" role="status">
                            <strong>{move || t(locale.get(), "系统生成密码", "Generated password")}</strong>
                            <code>{password}</code>
                            <span>{hotspot}</span>
                            <div class="space-guide-bind-card">
                                <strong>{move || t(locale.get(), "给这个空间配一篇攻略", "Add a guide to this Space")}</strong>
                                <p>{move || t(locale.get(), "把你已经写过的攻略关联到刚创建的空间；没有攻略也可以先去写一篇。", "Attach one of your existing guides to the new Space, or write one now.")}</p>
                                <Suspense fallback=move || view! { <span>{move || t(locale.get(), "正在读取可关联攻略", "Loading guides")}</span> }>
                                    {move || Suspend::new(async move {
                                        let guides = bindable_guides.await;
                                        view! {
                                            <div class="space-guide-bind-actions">
                                                <select
                                                    aria-label=move || t(locale.get(), "选择要关联的攻略", "Choose a guide to attach")
                                                    prop:value=move || bind_guide_id.get()
                                                    on:change=move |ev| bind_guide_id.set(event_target_value(&ev))
                                                >
                                                    <option value="">{move || t(locale.get(), "选择已有攻略（可选）", "Choose an existing guide (optional)")}</option>
                                                    <For each=move || guides.clone() key=|guide| guide.id.to_string() children=move |guide| view! {
                                                        <option value=guide.id.to_string()>{guide.title_zh}</option>
                                                    } />
                                                </select>
                                                <button class="button button-secondary-light" type="button" on:click=move |_| { bind.dispatch(()); }>
                                                    {move || t(locale.get(), "关联攻略", "Attach guide")}
                                                </button>
                                                <a class="button button-secondary-light" href=move || format!("/inspace/admin/guides/new?space_id={}", created_space_id.get().unwrap_or_default())>
                                                    {move || t(locale.get(), "写一篇新攻略", "Write a new guide")}
                                                </a>
                                            </div>
                                        }
                                    })}
                                </Suspense>
                                {move || bound_guide.get().map(|title| view! { <p class="form-success">{format!("{}：{}", t(locale.get(), "已关联", "Attached"), title)}</p> })}
                            </div>
                            <div class="community-setup-card">
                                <strong>{move || t(locale.get(), "📡 设置手机热点名", "📡 Set phone hotspot name")}</strong>
                                <p>{move || t(locale.get(), "请将手机热点名设为上方 InstantSpace_六位数字，用户在 WiFi 列表里直接抄 6 位密码。", "Set the phone hotspot name to the InstantSpace_ six-digit value above; visitors copy the six-digit password from WiFi.")}</p>
                                <strong>{move || t(locale.get(), "💬 创建 Discord/QQ 讨论组", "💬 Create Discord/QQ group")}</strong>
                                <ol>
                                    <li>{move || t(locale.get(), "在 Discord/QQ 频道找到对应省份板块。", "Find the matching province section in Discord/QQ.")}</li>
                                    <li>{format!("{}「{}」", t(locale.get(), "创建讨论组", "Create discussion group"), space_name)}</li>
                                    <li>{move || t(locale.get(), "向频道管理员申请讨论组管理员身份。", "Ask channel admins to grant you group admin.")}</li>
                                    <li>{move || t(locale.get(), "在讨论组里发布密码给审核通过的成员。", "Post the password only for approved group members.")}</li>
                                </ol>
                            </div>
                        </div>
                    }
                })}
                {move || created_name.get().map(|_| {
                    let modal = use_create_space_modal();
                    view! {
                        <button
                            class="button button-secondary button-secondary-light"
                            type="button"
                            on:click=move |_| {
                                if let Some(modal) = modal {
                                    modal.open.set(false);
                                }
                            }
                        >
                            {move || t(locale.get(), "关闭", "Close")}
                        </button>
                    }
                })}
                {move || error.get().map(|message| view! { <p class="error">{message}</p> })}
            </div>
        </form>
    }
}

fn parse_space_type(value: &str) -> SpaceType {
    match value {
        "scenic" => SpaceType::Scenic,
        "food" => SpaceType::Food,
        "park" => SpaceType::Park,
        "transit" => SpaceType::Transit,
        "event" => SpaceType::Event,
        _ => SpaceType::Custom,
    }
}
