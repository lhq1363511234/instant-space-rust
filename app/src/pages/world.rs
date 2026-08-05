use instant_domain::world::{EnterSpaceOutcome, SceneBundle, SceneObject, SceneObjectKind};
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::{use_params_map, use_query_map};
use serde::Serialize;
use uuid::Uuid;

use crate::components::space_experience_modal::OpenSpaceLink;
use crate::i18n::{localize_optional, t, use_i18n, Locale};
use crate::pages::space::SpacePanel;
use crate::server::world::{enter_world_space, get_space_scene};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct WorldLoad {
    bundle: SceneBundle,
    outcome: Option<EnterSpaceOutcome>,
    login_required: bool,
}

#[derive(Debug, Clone, Serialize)]
struct ClientSpawn {
    x: f64,
    y: f64,
    facing: String,
}

#[derive(Debug, Clone, Serialize)]
struct WorldClientPayload {
    bundle: SceneBundle,
    spawn: ClientSpawn,
    companions_moved: i64,
    locale: &'static str,
    login_required: bool,
}

#[component]
pub fn WorldScenePage() -> impl IntoView {
    let locale = use_i18n().locale;
    let params = use_params_map();
    let query = use_query_map();
    let space_id = Memo::new(move |_| params.with(|map| map.get("space_id").unwrap_or_default()));
    let scene_id = Memo::new(move |_| query.with(|map| map.get("scene").filter(|v| !v.is_empty())));
    let spawn_key =
        Memo::new(move |_| query.with(|map| map.get("spawn").filter(|v| !v.is_empty())));
    let entry_method = Memo::new(move |_| {
        query.with(|map| {
            map.get("via")
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| "link".to_string())
        })
    });
    let source_space_id =
        Memo::new(move |_| query.with(|map| map.get("from_space").filter(|v| !v.is_empty())));
    let source_object_id =
        Memo::new(move |_| query.with(|map| map.get("from_object").filter(|v| !v.is_empty())));

    let world = Resource::new(
        move || {
            (
                space_id.get(),
                scene_id.get(),
                spawn_key.get(),
                entry_method.get(),
                source_space_id.get(),
                source_object_id.get(),
            )
        },
        |(space_id, scene_id, spawn_key, method, source_space_id, source_object_id)| async move {
            if space_id.is_empty() {
                return Err("invalid space id".to_string());
            }
            match enter_world_space(
                space_id.clone(),
                scene_id.clone(),
                spawn_key,
                method,
                source_space_id,
                source_object_id,
            )
            .await
            {
                Ok(outcome) => Ok(WorldLoad {
                    bundle: outcome.bundle.clone(),
                    outcome: Some(outcome),
                    login_required: false,
                }),
                Err(entry_error) => get_space_scene(space_id, scene_id)
                    .await
                    .map(|bundle| WorldLoad {
                        bundle,
                        outcome: None,
                        login_required: entry_error.to_string().contains("login required"),
                    })
                    .map_err(|error| error.to_string()),
            }
        },
    );

    view! {
        <Title text=move || t(locale.get(), "进入空间｜inspace", "Enter Space | inspace") />
        <main id="main-content" class="page world-scene-page">
            <Suspense fallback=move || view! {
                <section class="world-loading" aria-live="polite">
                    <span class="world-loading-mark" aria-hidden="true"></span>
                    <p>{move || t(locale.get(), "正在推门入内……", "Opening the Space…")}</p>
                </section>
            }>
                {move || Suspend::new(async move {
                    match world.await {
                        Ok(load) => view! { <WorldRuntime load=load /> }.into_any(),
                        Err(message) => view! {
                            <section class="world-error">
                                <p class="world-kicker">{move || t(locale.get(), "门外", "Outside")}</p>
                                <h1>{move || t(locale.get(), "这处空间暂时不能进入", "This Space cannot be entered yet")}</h1>
                                <p>{message}</p>
                                <a class="world-link" href=format!("/inspace/spaces/{}", space_id.get())>
                                    {move || t(locale.get(), "返回空间详情", "Back to Space details")}
                                </a>
                            </section>
                        }.into_any(),
                    }
                })}
            </Suspense>
        </main>
    }
}

#[component]
fn WorldRuntime(load: WorldLoad) -> impl IntoView {
    let locale = use_i18n().locale;
    let bundle = load.bundle;
    let space_id = bundle.space_id;
    let scene_id = bundle.scene.id;
    let scene_kind = bundle.scene.kind.as_db().to_string();
    let is_home = scene_kind == "home";
    let back_href = if is_home {
        format!("/inspace/homes/{space_id}")
    } else {
        format!("/inspace/spaces/{space_id}")
    };
    let space_name = localize_optional(
        locale.get(),
        &bundle.space_name_zh,
        bundle.space_name_en.as_deref(),
    );
    let scene_name = localize_optional(
        locale.get(),
        &bundle.scene.name_zh,
        bundle.scene.name_en.as_deref(),
    );
    let default_spawn = bundle
        .spawn_points
        .iter()
        .find(|spawn| spawn.is_default)
        .or_else(|| bundle.spawn_points.first())
        .cloned();
    let spawn = load
        .outcome
        .as_ref()
        .map(|outcome| outcome.spawn.clone())
        .or(default_spawn);
    let spawn = ClientSpawn {
        x: spawn.as_ref().map(|value| value.x).unwrap_or(50.0),
        y: spawn.as_ref().map(|value| value.y).unwrap_or(84.0),
        facing: spawn
            .as_ref()
            .map(|value| value.facing.clone())
            .unwrap_or_else(|| "north".to_string()),
    };
    let companions_moved = load
        .outcome
        .as_ref()
        .map(|outcome| outcome.companions_moved)
        .unwrap_or(0)
        .clamp(0, 3);
    let payload = WorldClientPayload {
        bundle: bundle.clone(),
        spawn,
        companions_moved,
        locale: if matches!(locale.get(), Locale::Zh) {
            "zh"
        } else {
            "en"
        },
        login_required: load.login_required,
    };
    let payload_json = serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string());
    let world_aria_label = format!("{} · {}", space_name, scene_name);
    let objects_for_sheet = bundle.objects.clone();
    let objects_for_fallback = bundle.objects.clone();
    let object_count = bundle.objects.len();
    let login_required = load.login_required;

    view! {
        <article
            class="world-runtime"
            class:world-runtime-home=is_home
            data-world-runtime="true"
            data-world-kind=scene_kind
            data-world-state="Loading"
        >
            <header class="world-runtime-bar">
                <a class="world-back" href=back_href>
                    <span aria-hidden="true">"←"</span>
                    <span>{move || if is_home { t(locale.get(), "家门", "Home door") } else { t(locale.get(), "空间详情", "Space details") }}</span>
                </a>
                <div class="world-runtime-title">
                    <span>{space_name}</span>
                    <h1>{scene_name}</h1>
                </div>
                <span class="world-presence" data-world-status aria-live="polite">
                    {move || t(locale.get(), "正在推门入内", "Opening the Space")}
                </span>
            </header>

            <Show when=move || login_required>
                <p class="world-login-note">
                    {move || t(locale.get(), "当前是访客浏览；登录后才会记录到访与随行足迹。", "Guest view: sign in to record presence and companion trails.")}
                    <a href="/inspace/login">{move || t(locale.get(), "登录", "Sign in")}</a>
                </p>
            </Show>

            <section class="world-canvas-frame" aria-label=move || t(locale.get(), "可行走空间", "Walkable Space")>
                <div
                    class="world-canvas-host"
                    data-world-canvas="true"
                    data-world-payload=payload_json
                    role="application"
                    aria-label=world_aria_label
                    aria-describedby="world-controls-help"
                    tabindex="0"
                >
                    <div class="world-engine-boot" aria-hidden="true">
                        <span></span>
                        <p>{move || t(locale.get(), "正在铺开庭院……", "Preparing the scene…")}</p>
                    </div>
                </div>

                <p id="world-controls-help" class="sr-only">{move || t(locale.get(), "点按地面移动，也可使用方向键或 WASD。靠近物件后，使用出现的操作按钮或按回车互动。", "Tap the ground to move, or use arrow keys and WASD. When near an object, use the action button or press Enter.")}</p>
                <button type="button" class="world-context-action" data-world-action hidden></button>

                <div class="world-control-hint" aria-hidden="true">
                    <span>{move || t(locale.get(), "点地面行走", "Tap to walk")}</span>
                    <i></i>
                    <span>{move || t(locale.get(), "靠近后互动", "Approach to interact")}</span>
                </div>
            </section>

            <section class="world-text-fallback" aria-labelledby="world-fallback-title">
                <div>
                    <p class="world-kicker">{move || t(locale.get(), "文字入口", "Text access")}</p>
                    <h2 id="world-fallback-title">{move || t(locale.get(), "院中可去之处", "Places in this scene")}</h2>
                </div>
                <div class="world-fallback-list">
                    {objects_for_fallback.into_iter().map(|object| {
                        let id = object.id.to_string();
                        let name = localize_optional(locale.get(), &object.name_zh, object.name_en.as_deref());
                        view! {
                            <button type="button" data-world-fallback-object=id>
                                <span>{name}</span><span aria-hidden="true">"↗"</span>
                            </button>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </section>

            <div class="world-sheet" data-world-sheet hidden>
                <button class="world-sheet-curtain" type="button" data-world-sheet-curtain aria-label=move || t(locale.get(), "关闭内容", "Close content")></button>
                <section class="world-sheet-dialog" role="dialog" aria-modal="true" aria-label=move || t(locale.get(), "空间物件内容", "Space object content")>
                    <div class="world-sheet-handle" aria-hidden="true"></div>
                    <button class="world-sheet-close" type="button" data-world-sheet-close aria-label=move || t(locale.get(), "关闭", "Close")>"×"</button>
                    {objects_for_sheet.into_iter().map(|object| view! {
                        <WorldObjectCard object=object space_id=space_id scene_id=scene_id />
                    }).collect::<Vec<_>>()}
                </section>
            </div>

            <div class="world-space-modal-bridges" hidden aria-hidden="true">
                <OpenSpaceLink space_id=space_id.to_string() initial_panel=SpacePanel::Wall class="world-space-modal-bridge world-space-modal-bridge--wall">"Space"</OpenSpaceLink>
                <OpenSpaceLink space_id=space_id.to_string() initial_panel=SpacePanel::Intro class="world-space-modal-bridge world-space-modal-bridge--intro">"Introduction"</OpenSpaceLink>
                <OpenSpaceLink space_id=space_id.to_string() initial_panel=SpacePanel::Host class="world-space-modal-bridge world-space-modal-bridge--host">"Host"</OpenSpaceLink>
                <OpenSpaceLink space_id=space_id.to_string() initial_panel=SpacePanel::Story class="world-space-modal-bridge world-space-modal-bridge--story">"Stories"</OpenSpaceLink>
                <OpenSpaceLink space_id=space_id.to_string() initial_panel=SpacePanel::Capsules class="world-space-modal-bridge world-space-modal-bridge--capsules">"Capsules"</OpenSpaceLink>
                <OpenSpaceLink space_id=space_id.to_string() initial_panel=SpacePanel::Guides class="world-space-modal-bridge world-space-modal-bridge--guides">"Guides"</OpenSpaceLink>
                <OpenSpaceLink space_id=space_id.to_string() initial_panel=SpacePanel::Discussion class="world-space-modal-bridge world-space-modal-bridge--discussion">"Discussion"</OpenSpaceLink>
            </div>

            <footer class="world-runtime-footer">
                <span>{move || format!("{} {}", object_count, t(locale.get(), "处可互动之物", "interactive objects"))}</span>
                <span>{move || if companions_moved > 0 {
                    format!("{} {}", companions_moved, t(locale.get(), "位家人正在随行", "companions travelling with you"))
                } else {
                    t(locale.get(), "此刻独行", "Travelling alone").to_string()
                }}</span>
            </footer>
        </article>
    }
}

#[component]
fn WorldObjectCard(object: SceneObject, space_id: Uuid, scene_id: Uuid) -> impl IntoView {
    let locale = use_i18n().locale;
    let object_id = object.id.to_string();
    let name = localize_optional(locale.get(), &object.name_zh, object.name_en.as_deref());
    let copy_zh = object
        .config
        .get("copy_zh")
        .and_then(|value| value.as_str())
        .unwrap_or("此物尚待主理人补上一段来历。")
        .to_string();
    let copy_en = object
        .config
        .get("copy_en")
        .and_then(|value| value.as_str())
        .map(ToOwned::to_owned);
    let copy = localize_optional(locale.get(), &copy_zh, copy_en.as_deref());
    let action = object
        .config
        .get("action")
        .and_then(|value| value.as_str())
        .unwrap_or("detail");
    let detail_href = object_action_href(&object, action, space_id);
    let is_portal = object.kind == SceneObjectKind::Portal && object.target_space_id.is_some();
    let kind_label = object.kind.as_db().replace('_', " · ");
    let action_label = object_action_label(action, is_portal, locale.get());
    let _ = scene_id;
    let binding_note = if object.content_id.is_some() {
        t(
            locale.get(),
            "已与此空间的真实内容相连",
            "Connected to this Space's live content",
        )
    } else if matches!(object.kind, SceneObjectKind::AiGuide) {
        t(
            locale.get(),
            "先读取现有空间志；完整 AI 能力仍在接入",
            "Reads current journals first; full AI is still being connected",
        )
    } else if is_portal {
        t(
            locale.get(),
            "目标来自已建立的空间关系",
            "Target comes from an established Space relation",
        )
    } else {
        t(
            locale.get(),
            "等待主理人与来访者留下内容",
            "Waiting for hosts and visitors to leave content",
        )
    };

    view! {
        <article class="world-object-card" data-world-object-card=object_id hidden>
            <p class="world-kicker">{kind_label}</p>
            <h2>{name}</h2>
            <p>{copy}</p>
            <a class="world-primary-action" href=detail_href>{action_label}</a>
            <small>{binding_note}</small>
        </article>
    }
}

fn object_action_href(object: &SceneObject, action: &str, space_id: Uuid) -> String {
    if object.kind == SceneObjectKind::Portal {
        if let Some(target) = object.target_space_id {
            let mut href = format!(
                "/inspace/world/{target}?via=portal&from_space={space_id}&from_object={}",
                object.id
            );
            if let Some(scene) = object.target_scene_id {
                href.push_str(&format!("&scene={scene}"));
            }
            if let Some(spawn) = object.target_spawn_key.as_deref() {
                href.push_str(&format!("&spawn={spawn}"));
            }
            return href;
        }
    }

    match action {
        "guide" | "ai" => format!("/inspace/spaces/{space_id}#space-guides"),
        "host" => format!("/inspace/spaces/{space_id}#space-host"),
        "stories" => format!("/inspace/spaces/{space_id}#space-traces"),
        "capsule" => format!("/inspace/spaces/{space_id}#space-capsules"),
        "notice" => format!("/inspace/spaces/{space_id}/chat"),
        "trails" | "home" | "memorial" | "biography" => "/inspace/lives".to_string(),
        _ => format!("/inspace/spaces/{space_id}"),
    }
}

fn object_action_label(action: &str, is_portal: bool, locale: Locale) -> &'static str {
    if is_portal {
        return t(locale, "穿过传送门", "Pass through the portal");
    }
    match action {
        "stories" => t(locale, "读一读留下的故事", "Read the stories"),
        "notice" => t(locale, "看看今日现场", "See what is happening"),
        "trails" => t(locale, "翻开足迹册", "Open the trail album"),
        "home" => t(locale, "走进屋内", "Enter the home"),
        "memorial" | "biography" => t(locale, "翻开追远录", "Open the memorial record"),
        "guide" => t(locale, "走进游客中心", "Enter the visitor center"),
        "host" => t(locale, "听主理人说", "Hear from the host"),
        "capsule" => t(locale, "查看埋信处", "Open the capsule grove"),
        "ai" => t(
            locale,
            "请它从空间志中带路",
            "Ask it to guide from the journals",
        ),
        _ => t(locale, "打开相关内容", "Open related content"),
    }
}
