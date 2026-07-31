use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use crate::components::private_verify::PrivateVerify;
use crate::components::space_share::SpaceSharePanel;
use crate::components::space_traces::{CarveButton, SpaceTraces};
use crate::i18n::{localize_optional, t, use_i18n};
use crate::server::chat::{
    check_space_access, list_chat_messages, list_space_helps, raise_space_help,
    resolve_space_help, send_chat_message,
};
use crate::server::guides::list_space_guides;
use crate::server::spaces::{
    apply_host_claim, get_space_detail, my_host_claim, HostClaimState, SpaceDetailView, SpaceMarker,
};

/// `2026-07-08 18:36:17.568978000 +00:00:00` -> `("2026-07-08", "18:36")`.
fn split_chat_stamp(created_at: impl ToString) -> (String, String) {
    let raw = created_at.to_string();
    let mut parts = raw.split_whitespace();
    let date = parts.next().unwrap_or("").to_string();
    let time = parts.next().unwrap_or("");
    let mut hm = time.split(':');
    let hour = hm.next().unwrap_or("0");
    let minute = hm.next().unwrap_or("0");
    let clock = match (hour.parse::<u8>(), minute.parse::<u8>()) {
        (Ok(h), Ok(m)) => format!("{h:02}:{m:02}"),
        _ => format!("{hour}:{minute}"),
    };
    if date.is_empty() {
        return (raw, clock);
    }
    (date, clock)
}

/// First character of a sender name, used as the transcript monogram.
fn sender_monogram(sender: &str) -> String {
    sender
        .chars()
        .find(|c| !c.is_whitespace())
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_else(|| "?".to_string())
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SpacePanel {
    Wall,
    Intro,
    Host,
    Story,
    Guides,
}

#[component]
pub fn SpacePage() -> impl IntoView {
    let locale = use_i18n().locale;
    let params = use_params_map();
    let space_id =
        Memo::new(move |_| params.with(|params| params.get("space_id").unwrap_or_default()));
    let refresh = RwSignal::new(0u32);
    // The detail page opens as a restrained card wall; opening a card swaps the
    // main region to that one section instead of stacking everything vertically.
    let panel = RwSignal::new(SpacePanel::Wall);

    let access = Resource::new(
        move || (space_id.get(), refresh.get()),
        |(space_id, _)| async move {
            if space_id.is_empty() {
                None
            } else {
                check_space_access(space_id).await.ok()
            }
        },
    );
    let guides = Resource::new(
        move || space_id.get(),
        |space_id| async move {
            if space_id.is_empty() {
                Vec::new()
            } else {
                list_space_guides(space_id).await.unwrap_or_default()
            }
        },
    );
    let detail = Resource::new(
        move || space_id.get(),
        |space_id| async move {
            if space_id.is_empty() {
                None
            } else {
                get_space_detail(space_id).await.ok().flatten()
            }
        },
    );

    view! {
        <main id="main-content" class="page space-detail-page">
            <Suspense fallback=move || view! { <p class="space-section-loading">{move || t(locale.get(), "正在检查空间访问权限", "Checking space access")}</p> }>
                {move || Suspend::new(async move {
                    match access.await {
                        Some(state) if state.allowed => {
                            let name_en_opt = state.space_name_en.clone();
                            let space_name = crate::i18n::localize_optional(locale.get_untracked(), &state.space_name, name_en_opt.as_deref());
                            let id = state.space_id.to_string();
                            let is_public = state.is_public;
                            let guide_href = format!("/inspace/guides/new?space_id={id}");
                            let chat_href = format!("/inspace/spaces/{id}/chat");
                            view! {
                                <article class="space-detail-shell" aria-label="Space detail">
                                    <header class="space-detail-header">
                                        <div class="space-detail-breadcrumb">
                                            <a href="/inspace/explore">{move || t(locale.get(), "空间探索", "Explore")}</a>
                                            <span aria-hidden="true">"/"</span>
                                            <span>{move || t(locale.get(), "空间详情", "Space detail")}</span>
                                        </div>
                                        <div class="space-detail-heading-row">
                                            <div class="space-app-header-main">
                                                <div class="space-detail-badges">
                                                    <span class="space-detail-visibility">{move || if is_public { t(locale.get(), "公开空间", "Public Space") } else { t(locale.get(), "私密空间", "Private Space") }}</span>
                                                </div>
                                                <h1>{space_name.clone()}</h1>
                                                <Suspense fallback=move || view! { <span class="space-meta-line">{move || t(locale.get(), "正在加载地点信息…", "Loading place details…")}</span> }>
                                                    {move || Suspend::new(async move {
                                                        view! { <SpaceMetaLine space=detail.await.map(|d| d.summary) /> }
                                                    })}
                                                </Suspense>
                                            </div>
                                        </div>
                                    </header>

                                    {
                                        let id = id.clone();
                                        let space_name = space_name.clone();
                                        let guide_href = guide_href.clone();
                                        let chat_href = chat_href.clone();
                                        move || {
                                            let id = id.clone();
                                            let space_name = space_name.clone();
                                            let guide_href = guide_href.clone();
                                            let chat_href = chat_href.clone();
                                            match panel.get() {
                                                SpacePanel::Wall => view! {
                                                    <SpaceCardWall
                                                        space_id=id
                                                        space_name=space_name
                                                        chat_href=chat_href
                                                        panel=panel
                                                    />
                                                }.into_any(),
                                                SpacePanel::Intro => view! {
                                                    <SpacePanelFrame panel=panel title=t(locale.get(), "简介", "About")>
                                                        <Suspense fallback=move || view! { <div class="space-section-loading">{move || t(locale.get(), "正在加载简介…", "Loading…")}</div> }>
                                                            {move || Suspend::new(async move {
                                                                view! { <SpaceIntroPanel detail=detail.await /> }
                                                            })}
                                                        </Suspense>
                                                    </SpacePanelFrame>
                                                }.into_any(),
                                                SpacePanel::Host => view! {
                                                    <SpacePanelFrame panel=panel title=t(locale.get(), "主理人", "Host")>
                                                        <Suspense fallback=move || view! { <div class="space-section-loading">{move || t(locale.get(), "正在加载…", "Loading…")}</div> }>
                                                            {
                                                                let host_space_id = id.clone();
                                                                move || {
                                                                    let host_space_id = host_space_id.clone();
                                                                    Suspend::new(async move {
                                                                        view! { <SpaceHostPanel space_id=host_space_id detail=detail.await /> }
                                                                    })
                                                                }
                                                            }
                                                        </Suspense>
                                                    </SpacePanelFrame>
                                                }.into_any(),
                                                SpacePanel::Story => view! {
                                                    <SpacePanelFrame panel=panel title=t(locale.get(), "故事", "Stories")>
                                                        <SpaceTraces space_id=id.clone() space_name=space_name.clone() />
                                                    </SpacePanelFrame>
                                                }.into_any(),
                                                SpacePanel::Guides => view! {
                                                    <SpacePanelFrame panel=panel title=t(locale.get(), "空间志", "Guides")>
                                                        <Suspense fallback=move || view! { <div class="space-section-loading">{move || t(locale.get(), "正在加载空间志…", "Loading guides…")}</div> }>
                                                            {
                                                                let write_href = guide_href.clone();
                                                                move || {
                                                                    let write_href = write_href.clone();
                                                                    Suspend::new(async move {
                                                                        let items = guides.await;
                                                                        view! { <SpaceGuideList guides=items write_href=write_href /> }
                                                                    })
                                                                }
                                                            }
                                                        </Suspense>
                                                    </SpacePanelFrame>
                                                }.into_any(),
                                            }
                                        }
                                    }
                                </article>
                            }.into_any()
                        }
                        Some(state) => {
                            let name_en_opt = state.space_name_en.clone();
                            let space_name = crate::i18n::localize_optional(locale.get_untracked(), &state.space_name, name_en_opt.as_deref());
                            let community_space_name = space_name.clone();
                            let id = state.space_id.to_string();
                            view! {
                                <section class="form">
                                    <h1>{move || t(locale.get(), "私密空间需要访问码", "Private Space requires an access code")}</h1>
                                    <p>{move || t(locale.get(), "验证后进入这个地点的空间详情：简介、主理人、空间志、故事、分享和讨论都在这里。", "After verification, enter this place’s Space detail: about, host, guides, stories, sharing and discussion are all here.")}</p>
                                    <PrivateVerify
                                        space_id=id
                                        space_name=space_name
                                        on_verified=Callback::new(move |_| refresh.update(|value| *value += 1))
                                    />
                                    <CommunityLinks space_name=community_space_name />
                                </section>
                            }.into_any()
                        }
                        None => view! {
                            <section class="empty-state">
                                <strong>{move || t(locale.get(), "空间不存在或暂不可用", "Space not found or unavailable")}</strong>
                                <a class="button button-primary" href="/inspace">{move || t(locale.get(), "返回地图", "Back to map")}</a>
                            </section>
                        }.into_any(),
                    }
                })}
            </Suspense>
        </main>
    }
}

/// The default detail view: a quiet, single-column index. Long explanations
/// and sharing tools stay behind deliberate actions instead of competing in
/// the first viewport.
#[component]
fn SpaceCardWall(
    space_id: String,
    space_name: String,
    chat_href: String,
    panel: RwSignal<SpacePanel>,
) -> impl IntoView {
    let locale = use_i18n().locale;
    let share_space_name = space_name.clone();
    let community_space_name = space_name.clone();
    view! {
        <div class="space-card-wall">
            <section class="space-place-index" aria-labelledby="space-place-index-title">
                <header class="space-place-index-head">
                    <p class="survey-kicker">{move || t(locale.get(), "空间目录", "Space index")}</p>
                    <h2 id="space-place-index-title">{move || t(locale.get(), "从这里，读懂这个地点。", "Start here to understand this place.")}</h2>
                </header>

                <nav class="space-entry-index" aria-label=move || t(locale.get(), "空间内容入口", "Space contents")>
                    <button type="button" class="space-entry-row space-entry-row-lead" on:click=move |_| panel.set(SpacePanel::Intro)>
                        <span class="space-entry-row-main">
                            <span class="space-entry-card-label">{move || t(locale.get(), "简介", "About")}</span>
                            <span class="space-entry-card-desc">{move || t(locale.get(), "地点与来历", "Place and background")}</span>
                        </span>
                        <span class="space-entry-card-go">{move || t(locale.get(), "打开", "Open")}</span>
                    </button>
                    <button type="button" class="space-entry-row" on:click=move |_| panel.set(SpacePanel::Host)>
                        <span class="space-entry-row-main">
                            <span class="space-entry-card-label">{move || t(locale.get(), "主理人", "Host")}</span>
                            <span class="space-entry-card-desc">{move || t(locale.get(), "谁在维护", "Who keeps it")}</span>
                        </span>
                        <span class="space-entry-card-go">{move || t(locale.get(), "打开", "Open")}</span>
                    </button>
                    <button type="button" class="space-entry-row" on:click=move |_| panel.set(SpacePanel::Story)>
                        <span class="space-entry-row-main">
                            <span class="space-entry-card-label">{move || t(locale.get(), "故事", "Stories")}</span>
                            <span class="space-entry-card-desc">{move || t(locale.get(), "来过的人留下什么", "What visitors left")}</span>
                        </span>
                        <span class="space-entry-card-go">{move || t(locale.get(), "打开", "Open")}</span>
                    </button>
                    <button type="button" class="space-entry-row" on:click=move |_| panel.set(SpacePanel::Guides)>
                        <span class="space-entry-row-main">
                            <span class="space-entry-card-label">{move || t(locale.get(), "空间志", "Guides")}</span>
                            <span class="space-entry-card-desc">{move || t(locale.get(), "实地记录", "Field notes")}</span>
                        </span>
                        <span class="space-entry-card-go">{move || t(locale.get(), "打开", "Open")}</span>
                    </button>
                    <a class="space-entry-row" href=chat_href>
                        <span class="space-entry-row-main">
                            <span class="space-entry-card-label">{move || t(locale.get(), "讨论", "Discussion")}</span>
                            <span class="space-entry-card-desc">{move || t(locale.get(), "今天现场", "What is happening now")}</span>
                        </span>
                        <span class="space-entry-card-go">{move || t(locale.get(), "进入", "Enter")}</span>
                    </a>
                </nav>
            </section>

            <details id="space-share" class="space-tools-disclosure">
                <summary>
                    <span>{move || t(locale.get(), "分享与进入", "Share and enter")}</span>
                    <small>{move || t(locale.get(), "链接、二维码、社群", "Link, QR code, community")}</small>
                </summary>
                <div class="space-card-tools" aria-label=move || t(locale.get(), "空间工具", "Space tools")>
                    <SpaceSharePanel space_id=space_id space_name=share_space_name compact=true />
                    <CommunityLinks space_name=community_space_name />
                </div>
            </details>
        </div>
    }
}

/// Shell around an opened panel: a back control returns to the card wall.
#[component]
fn SpacePanelFrame(
    panel: RwSignal<SpacePanel>,
    title: &'static str,
    children: Children,
) -> impl IntoView {
    let locale = use_i18n().locale;
    view! {
        <section class="space-panel-open" aria-label=title>
            <div class="space-panel-open-bar">
                <button type="button" class="space-panel-back" on:click=move |_| panel.set(SpacePanel::Wall)>
                    <span aria-hidden="true">"←"</span>
                    <span>{move || t(locale.get(), "返回", "Back")}</span>
                </button>
                <span class="space-panel-open-title">{title}</span>
            </div>
            <div class="space-panel-open-body">
                {children()}
            </div>
        </section>
    }
}

/// The intro panel: what this place is and why the Space exists. Every line is
/// drawn from stored fields — description, tag, type — so we never invent copy.
#[component]
fn SpaceIntroPanel(detail: Option<SpaceDetailView>) -> impl IntoView {
    let locale = use_i18n().locale;
    let Some(detail) = detail else {
        return view! {
            <div class="empty-state compact-empty">
                <strong>{move || t(locale.get(), "简介还没有补充", "No description yet")}</strong>
                <span>{move || t(locale.get(), "主理人认领后，会在这里写清楚这是什么地方、为什么值得进来。", "Once a host claims it, they will write here what this place is and why it is worth entering.")}</span>
            </div>
        }.into_any();
    };

    let (kind_zh, kind_en) = space_type_words(&detail.summary.space_type);
    let custom_type = detail.custom_type.clone();
    let type_label = move || {
        if let Some(custom) = custom_type.clone().filter(|value| !value.trim().is_empty()) {
            custom
        } else {
            t(locale.get(), kind_zh, kind_en).to_string()
        }
    };

    let description = localize_optional(
        locale.get_untracked(),
        detail.description_zh.as_deref().unwrap_or(""),
        detail.description_en.as_deref(),
    );
    let has_description = !description.trim().is_empty();
    let description_zh = detail.description_zh.clone().unwrap_or_default();
    let description_en = detail.description_en.clone();

    let tag = localize_optional(
        locale.get_untracked(),
        detail.tag_zh.as_deref().unwrap_or(""),
        detail.tag_en.as_deref(),
    );
    let has_tag = !tag.trim().is_empty();
    let tag_zh = detail.tag_zh.clone().unwrap_or_default();
    let tag_en = detail.tag_en.clone();

    let location = join_location(&detail.summary);

    view! {
        <div class="space-fact-wall">
            <div class="space-fact-card">
                <span class="space-fact-key">{move || t(locale.get(), "类型", "Type")}</span>
                <span class="space-fact-value">{type_label}</span>
            </div>
            {(!location.is_empty()).then(|| {
                let location = location.clone();
                view! {
                    <div class="space-fact-card">
                        <span class="space-fact-key">{move || t(locale.get(), "位置", "Location")}</span>
                        <span class="space-fact-value">{location.clone()}</span>
                    </div>
                }
            })}
            {has_tag.then(|| view! {
                <div class="space-fact-card">
                    <span class="space-fact-key">{move || t(locale.get(), "标签", "Tag")}</span>
                    <span class="space-fact-value">{move || localize_optional(locale.get(), &tag_zh, tag_en.as_deref())}</span>
                </div>
            })}
            {if has_description {
                view! {
                    <div class="space-fact-card space-fact-card-wide">
                        <span class="space-fact-key">{move || t(locale.get(), "这是什么地方", "What this place is")}</span>
                        <p class="space-fact-prose">{move || localize_optional(locale.get(), &description_zh, description_en.as_deref())}</p>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="space-fact-card space-fact-card-wide">
                        <span class="space-fact-key">{move || t(locale.get(), "这是什么地方", "What this place is")}</span>
                        <p class="space-fact-prose space-fact-prose-muted">{move || t(locale.get(), "还没有人写下这个地点的简介。认领主理人后可以补上。", "Nobody has written this place's description yet. A host can add it after claiming.")}</p>
                    </div>
                }.into_any()
            }}
        </div>
    }.into_any()
}

/// The host panel: who maintains the Space and since when. We only state what
/// the data supports — the host's display name and the creation date — and are
/// explicit when a Space is still waiting to be claimed.
#[component]
fn SpaceHostPanel(space_id: String, detail: Option<SpaceDetailView>) -> impl IntoView {
    let locale = use_i18n().locale;
    let host_name = detail
        .as_ref()
        .and_then(|d| d.host_name.clone())
        .filter(|value| !value.trim().is_empty());
    let bio_zh = detail
        .as_ref()
        .and_then(|d| d.host_bio_zh.clone())
        .unwrap_or_default();
    let bio_en = detail.as_ref().and_then(|d| d.host_bio_en.clone());
    let has_bio =
        !bio_zh.trim().is_empty() || bio_en.as_deref().is_some_and(|v| !v.trim().is_empty());
    let since = detail
        .as_ref()
        .and_then(|d| d.created_at.clone())
        .map(|value| value.chars().take(10).collect::<String>())
        .filter(|value| !value.trim().is_empty());

    view! {
        <div class="space-host-panel">
            {match host_name {
                Some(name) => view! {
                    <div class="space-host-identity">
                        <span class="space-host-avatar" aria-hidden="true">{name.chars().next().map(|c| c.to_uppercase().to_string()).unwrap_or_default()}</span>
                        <div>
                            <p class="space-host-name">{name}</p>
                            <p class="space-host-role">{move || t(locale.get(), "空间主理人", "Space host")}</p>
                        </div>
                    </div>
                    {has_bio.then(|| {
                        let bio_zh = bio_zh.clone();
                        let bio_en = bio_en.clone();
                        view! {
                            <p class="space-host-bio">{move || localize_optional(locale.get(), &bio_zh, bio_en.as_deref())}</p>
                        }
                    })}
                }.into_any(),
                None => view! {
                    <SpaceHostClaim space_id=space_id.clone() />
                }.into_any(),
            }}
            {since.map(|since| view! {
                <div class="space-host-since">
                    <span class="space-fact-key">{move || t(locale.get(), "空间建立于", "Space created")}</span>
                    <span class="space-fact-value">{since}</span>
                </div>
            })}
        </div>
    }.into_any()
}

/// The claim flow shown when a Space has no host yet. A signed-in visitor can
/// apply to become the host; an admin approves the application later. We read
/// the visitor's existing claim state so a repeat visit shows "under review"
/// rather than offering to apply again.
#[component]
fn SpaceHostClaim(space_id: String) -> impl IntoView {
    let locale = use_i18n().locale;
    let message = RwSignal::new(String::new());
    let error = RwSignal::new(None::<String>);
    let refresh = RwSignal::new(0u32);

    let claim_space_id = space_id.clone();
    let state = Resource::new(
        move || (claim_space_id.clone(), refresh.get()),
        |(space_id, _)| async move {
            my_host_claim(space_id)
                .await
                .unwrap_or(HostClaimState::None)
        },
    );

    let apply_space_id = space_id.clone();
    let apply = Action::new(move |_: &()| {
        let space_id = apply_space_id.clone();
        let note = message.get();
        async move { apply_host_claim(space_id, (!note.trim().is_empty()).then_some(note)).await }
    });

    Effect::new(move |_| {
        if let Some(result) = apply.value().get() {
            match result {
                Ok(_) => {
                    error.set(None);
                    refresh.update(|v| *v += 1);
                }
                Err(err) => error.set(Some(err.to_string())),
            }
        }
    });

    view! {
        <div class="space-host-vacant">
            <p class="space-host-vacant-title">{move || t(locale.get(), "这个空间正在招募主理人", "This Space is looking for a host")}</p>
            <p>{move || t(locale.get(), "目前由 inspace 编辑部临时看护。熟悉这里、愿意长期维护的人，可以申请成为主理人：整理简介、维护空间志、回答现场问题。", "For now it is tended by the inspace editorial team. Someone who knows the place and will maintain it can apply to become the host: write the intro, keep the guides, answer on-site questions.")}</p>
            <Suspense fallback=move || view! { <div class="space-section-loading">{move || t(locale.get(), "正在加载…", "Loading…")}</div> }>
                {move || Suspend::new(async move {
                    match state.await {
                        HostClaimState::Anonymous => view! {
                            <a class="button button-primary" href="/inspace/login">{move || t(locale.get(), "登录后申请认领", "Sign in to apply")}</a>
                        }.into_any(),
                        HostClaimState::Pending => view! {
                            <p class="space-host-claim-status">{move || t(locale.get(), "你的认领申请已提交，等待管理员审核。", "Your application has been submitted and is awaiting admin review.")}</p>
                        }.into_any(),
                        HostClaimState::Approved => view! {
                            <p class="space-host-claim-status">{move || t(locale.get(), "你的认领已通过，刷新后即可管理这个空间。", "Your claim was approved. Refresh to manage this Space.")}</p>
                        }.into_any(),
                        HostClaimState::Rejected => view! {
                            <p class="space-host-claim-status">{move || t(locale.get(), "上一次申请未通过，你可以补充说明后再次申请。", "Your last application was not approved. Add a note and apply again.")}</p>
                            <ClaimForm message=message error=error apply=apply />
                        }.into_any(),
                        _ => view! {
                            <ClaimForm message=message error=error apply=apply />
                        }.into_any(),
                    }
                })}
            </Suspense>
        </div>
    }.into_any()
}

#[component]
fn ClaimForm(
    message: RwSignal<String>,
    error: RwSignal<Option<String>>,
    apply: Action<(), Result<HostClaimState, ServerFnError>>,
) -> impl IntoView {
    let locale = use_i18n().locale;
    view! {
        <form class="space-host-claim-form" on:submit=move |ev| { ev.prevent_default(); apply.dispatch(()); }>
            <label class="field-label">
                <span>{move || t(locale.get(), "为什么由你来维护（可选）", "Why you (optional)")}</span>
                <textarea
                    rows="3"
                    placeholder=move || t(locale.get(), "例如：我在这附近生活/工作，熟悉这里。", "e.g. I live or work nearby and know this place well.")
                    prop:value=move || message.get()
                    on:input=move |ev| message.set(event_target_value(&ev))
                ></textarea>
            </label>
            {move || error.get().map(|err| view! { <p class="form-error">{err}</p> })}
            <button class="button button-primary" type="submit" prop:disabled=move || apply.pending().get()>
                {move || t(locale.get(), "申请成为主理人", "Apply to be the host")}
            </button>
        </form>
    }
}

/// Discussion lives on its own route so the space page stays readable and the
/// chat surface can own the full viewport height on mobile.
#[component]
pub fn SpaceChatPage() -> impl IntoView {
    let locale = use_i18n().locale;
    let params = use_params_map();
    let space_id =
        Memo::new(move |_| params.with(|params| params.get("space_id").unwrap_or_default()));
    let refresh = RwSignal::new(0u32);
    let message_body = RwSignal::new(String::new());
    let send_error = RwSignal::new(None::<String>);
    let help_body = RwSignal::new(String::new());
    let help_error = RwSignal::new(None::<String>);
    let show_help_form = RwSignal::new(false);

    let access = Resource::new(
        move || (space_id.get(), refresh.get()),
        |(space_id, _)| async move {
            if space_id.is_empty() {
                None
            } else {
                check_space_access(space_id).await.ok()
            }
        },
    );
    let messages = Resource::new(
        move || (space_id.get(), refresh.get()),
        |(space_id, _)| async move {
            if space_id.is_empty() {
                Vec::new()
            } else {
                list_chat_messages(space_id).await.unwrap_or_default()
            }
        },
    );

    let helps = Resource::new(
        move || (space_id.get(), refresh.get()),
        |(space_id, _)| async move {
            if space_id.is_empty() {
                Vec::new()
            } else {
                list_space_helps(space_id).await.unwrap_or_default()
            }
        },
    );

    let raise_help = Action::new(move |body: &String| {
        let space_id = space_id.get();
        let body = body.clone();
        async move { raise_space_help(space_id, body).await }
    });
    let resolve_help = Action::new(move |help_id: &String| {
        let help_id = help_id.clone();
        async move { resolve_space_help(help_id).await }
    });

    let send = Action::new(move |body: &String| {
        let space_id = space_id.get();
        let body = body.clone();
        async move { send_chat_message(space_id, body).await }
    });

    Effect::new(move |_| {
        if let Some(result) = raise_help.value().get() {
            match result {
                Ok(_) => {
                    help_body.set(String::new());
                    show_help_form.set(false);
                    help_error.set(None);
                    refresh.update(|value| *value += 1);
                }
                Err(err) => help_error.set(Some(err.to_string())),
            }
        }
    });

    Effect::new(move |_| {
        if let Some(result) = resolve_help.value().get() {
            match result {
                Ok(_) => {
                    help_error.set(None);
                    refresh.update(|value| *value += 1);
                }
                Err(err) => help_error.set(Some(err.to_string())),
            }
        }
    });

    Effect::new(move |_| {
        if let Some(result) = send.value().get() {
            match result {
                Ok(_) => {
                    message_body.set(String::new());
                    send_error.set(None);
                    refresh.update(|value| *value += 1);
                }
                Err(err) => send_error.set(Some(err.to_string())),
            }
        }
    });

    view! {
        <main id="main-content" class="page space-chat-page">
            <Suspense fallback=move || view! { <p class="space-section-loading">{move || t(locale.get(), "正在检查空间访问权限", "Checking space access")}</p> }>
                {move || Suspend::new(async move {
                    match access.await {
                        Some(state) if state.allowed => {
                            let name_en_opt = state.space_name_en.clone();
                            let space_name = crate::i18n::localize_optional(locale.get_untracked(), &state.space_name, name_en_opt.as_deref());
                            let verify_space_name = space_name.clone();
                            let id = state.space_id.to_string();
                            let messages_space_id = id.clone();
                            let carve_space_id = id.clone();
                            let is_public = state.is_public;
                            let back_href = format!("/inspace/spaces/{id}");
                            let keep_href = format!("/inspace/spaces/{id}#space-traces");
                            view! {
                                <section class="chat-shell chat-room" aria-label="Space discussion">
                                    <header class="chat-room-header">
                                        <a class="chat-room-back" href=back_href.clone() aria-label=move || t(locale.get(), "返回空间", "Back to Space")>
                                            <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M15 5l-7 7 7 7" /></svg>
                                        </a>
                                        <div class="chat-room-title">
                                            <h1>{space_name}</h1>
                                            <p class="chat-room-sub">
                                                <span class="chat-realtime-status" data-realtime-status="connecting" aria-live="polite">
                                                    {move || t(locale.get(), "连接中", "Connecting")}
                                                </span>
                                                <span class="chat-online-count" data-realtime-online="0">
                                                    {move || t(locale.get(), "0 人在场", "0 here")}
                                                </span>
                                            </p>
                                        </div>
                                        <a class="chat-room-keep" href=keep_href>
                                            {move || t(locale.get(), "这里留下的", "What stays")}
                                        </a>
                                    </header>
                                    <p class="chat-room-nature">
                                        {move || t(
                                            locale.get(),
                                            "讨论会滚走。想让一句话留在这个地点，把它刻进留痕。",
                                            "Discussion scrolls away. To make a line stay with this place, carve it into the record.",
                                        )}
                                    </p>

                                    <Suspense fallback=move || view! { <p class="space-section-loading">{move || t(locale.get(), "正在加载讨论…", "Loading discussion…")}</p> }>
                                        {move || {
                                            let realtime_space_id = messages_space_id.clone();
                                            let carve_space_id = carve_space_id.clone();
                                            Suspend::new(async move {
                                                let items = messages.await;
                                                view! {
                                                    <div
                                                        class="chat-message-list"
                                                        aria-label="Chat messages"
                                                        data-realtime-messages="true"
                                                        data-space-id=realtime_space_id
                                                    >
                                                        {if items.is_empty() {
                                                            view! {
                                                                <div class="chat-empty">
                                                                    <strong>{move || t(locale.get(), "这里还没有人说话", "Nobody has spoken here yet")}</strong>
                                                                    <span>{move || t(locale.get(), "问一个只有现场的人答得上来的问题：入口、排队、光线、末班车。", "Ask something only people on site can answer: entrances, queues, light, last train.")}</span>
                                                                </div>
                                                            }.into_any()
                                                        } else {
                                                            view! {
                                                                <For
                                                                    each=move || items.clone()
                                                                    key=|message| message.id
                                                                    children=move |message| {
                                                                        let (day, clock) = split_chat_stamp(message.created_at);
                                                                        let monogram = sender_monogram(&message.sender);
                                                                        let kind_class = match message.kind {
                                                                            instant_domain::chat::ChatMessageKind::System => "chat-message--system",
                                                                            instant_domain::chat::ChatMessageKind::Help => "chat-message--help",
                                                                            instant_domain::chat::ChatMessageKind::HelpResolved => "chat-message--help-resolved",
                                                                            instant_domain::chat::ChatMessageKind::Text => "",
                                                                        };
                                                                        view! {
                                                                            <article class=move || format!("chat-message {kind_class}") data-message-id=message.id.to_string()>
                                                                                <span class="chat-avatar" aria-hidden="true">{monogram}</span>
                                                                                <div class="chat-message-body">
                                                                                    <p class="chat-message-meta">
                                                                                        <strong>{message.sender}</strong>
                                                                                        <time datetime=day.clone()>{clock}</time>
                                                                                    </p>
                                                                                    <p class="chat-message-text">{message.body.clone()}</p>
                                                                                </div>
                                                                                <CarveButton
                                                                                    space_id=carve_space_id.clone()
                                                                                    message_id=message.id.to_string()
                                                                                    body=message.body
                                                                                />
                                                                            </article>
                                                                        }
                                                                    }
                                                                />
                                                            }.into_any()
                                                        }}
                                                    </div>
                                                }
                                            })
                                        }}
                                    </Suspense>

                                    <div class="chat-help-panel" aria-label=move || t(locale.get(), "求助", "Help")>
                                        <div class="chat-help-head">
                                            <button
                                                class="chat-help-toggle"
                                                type="button"
                                                on:click=move |_| show_help_form.update(|value| *value = !*value)
                                            >
                                                {move || if show_help_form.get() {
                                                    t(locale.get(), "收起求助", "Close help form").to_string()
                                                } else {
                                                    t(locale.get(), "现场求助", "Ask for help").to_string()
                                                }}
                                            </button>
                                            {move || help_error.get().map(|message| view! { <p class="error">{message}</p> })}
                                        </div>
                                        {move || show_help_form.get().then(|| view! {
                                            <form
                                                class="chat-help-form"
                                                on:submit=move |ev| {
                                                    ev.prevent_default();
                                                    raise_help.dispatch(help_body.get());
                                                }
                                            >
                                                <input
                                                    aria-label=move || t(locale.get(), "描述你需要的帮助", "Describe the help you need")
                                                    placeholder=move || t(locale.get(), "例：卫生间在哪？轮椅能进吗？", "e.g. Where is the toilet? Wheelchair accessible?")
                                                    prop:value=move || help_body.get()
                                                    on:input=move |ev| help_body.set(event_target_value(&ev))
                                                />
                                                <button
                                                    class="button button-primary"
                                                    type="submit"
                                                    disabled=move || help_body.get().trim().is_empty()
                                                >
                                                    {move || t(locale.get(), "发布求助", "Post help")}
                                                </button>
                                            </form>
                                        })}
                                        <Suspense fallback=move || view! { <span></span> }>
                                            {move || Suspend::new(async move {
                                                let active = helps.await;
                                                if active.is_empty() {
                                                    view! {}.into_any()
                                                } else {
                                                    view! {
                                                        <ul class="chat-help-list">
                                                            <For
                                                                each=move || active.clone()
                                                                key=|help| help.id
                                                                children=move |help| {
                                                                    let requester = help.requester_name.clone().unwrap_or_else(|| t(locale.get_untracked(), "现场访客", "On-site visitor").to_string());
                                                                    let help_id = help.id.to_string();
                                                                    view! {
                                                                        <li class="chat-help-item">
                                                                            <span class="chat-help-body">{help.body.clone()}</span>
                                                                            <small class="chat-help-requester">{requester}</small>
                                                                            <button
                                                                                class="button button-secondary-light chat-help-resolve"
                                                                                type="button"
                                                                                on:click=move |_| { resolve_help.dispatch(help_id.clone()); }
                                                                            >
                                                                                {move || t(locale.get(), "已解决", "Resolved")}
                                                                            </button>
                                                                        </li>
                                                                    }
                                                                }
                                                            />
                                                        </ul>
                                                    }.into_any()
                                                }
                                            })}
                                        </Suspense>
                                    </div>

                                    <form
                                        class="chat-compose"
                                        data-chat-form="true"
                                        data-space-id=id.clone()
                                        on:submit=move |ev| {
                                            ev.prevent_default();
                                            send.dispatch(message_body.get());
                                        }
                                    >
                                        <label class="field-label">
                                            <span class="visually-hidden">{move || t(locale.get(), "提出问题或补充现场信息", "Ask or add an on-site update")}</span>
                                            <textarea
                                                rows="1"
                                                aria-label=move || t(locale.get(), "聊天消息", "Chat message")
                                                data-chat-input="true"
                                                data-chat-autogrow="true"
                                                placeholder=move || t(locale.get(), "问点现场的事…", "Ask about right now…")
                                                prop:value=move || message_body.get()
                                                on:input=move |ev| message_body.set(event_target_value(&ev))
                                            ></textarea>
                                        </label>
                                        <button
                                            class="button button-primary chat-send"
                                            type="submit"
                                            aria-label=move || t(locale.get(), "发送", "Send")
                                            disabled=move || message_body.get().trim().is_empty()
                                        >
                                            <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M3.6 20.4L21 12 3.6 3.6l3 8.4-3 8.4z" /></svg>
                                            <span class="chat-send-label">{move || t(locale.get(), "发送", "Send")}</span>
                                        </button>
                                        {move || send_error.get().map(|message| view! { <p class="error">{message}</p> })}
                                    </form>

                                    {(!is_public).then(|| view! {
                                        <div class="realtime-reverify" data-private-reverify="true" style="display:none">
                                            <PrivateVerify space_id=id.clone() space_name=verify_space_name.clone() />
                                        </div>
                                    })}
                                </section>
                            }.into_any()
                        }
                        Some(state) => {
                            let name_en_opt = state.space_name_en.clone();
                            let space_name = crate::i18n::localize_optional(locale.get_untracked(), &state.space_name, name_en_opt.as_deref());
                            let id = state.space_id.to_string();
                            view! {
                                <section class="form">
                                    <h1>{move || t(locale.get(), "私密空间需要访问码", "Private Space requires an access code")}</h1>
                                    <PrivateVerify
                                        space_id=id
                                        space_name=space_name
                                        on_verified=Callback::new(move |_| refresh.update(|value| *value += 1))
                                    />
                                </section>
                            }.into_any()
                        }
                        None => view! {
                            <section class="empty-state">
                                <strong>{move || t(locale.get(), "空间不存在或暂不可用", "Space not found or unavailable")}</strong>
                                <a class="button button-primary" href="/inspace/explore">{move || t(locale.get(), "返回空间列表", "Back to Spaces")}</a>
                            </section>
                        }.into_any(),
                    }
                })}
            </Suspense>
        </main>
    }
}

fn space_type_words(
    space_type: &instant_domain::spaces::SpaceType,
) -> (&'static str, &'static str) {
    use instant_domain::spaces::SpaceType;
    match space_type {
        SpaceType::Scenic => ("景点", "Scenic"),
        SpaceType::Food => ("美食", "Food"),
        SpaceType::Park => ("公园", "Park"),
        SpaceType::Transit => ("交通", "Transit"),
        SpaceType::Event => ("活动", "Event"),
        SpaceType::Custom => ("其他", "Other"),
    }
}

/// Join the most specific 3 location parts of a Space, de-duplicated, coarse→fine.
fn join_location(space: &SpaceMarker) -> String {
    [
        space.country.clone(),
        space.province.clone(),
        space.city.clone(),
        space.district.clone(),
        space.spot_name.clone(),
    ]
    .into_iter()
    .flatten()
    .filter(|value| !value.trim().is_empty())
    .fold(Vec::<String>::new(), |mut values, value| {
        if !values.contains(&value) {
            values.push(value);
        }
        values
    })
    .into_iter()
    .rev()
    .take(3)
    .collect::<Vec<_>>()
    .into_iter()
    .rev()
    .collect::<Vec<_>>()
    .join(" · ")
}

#[component]
fn SpaceMetaLine(space: Option<SpaceMarker>) -> impl IntoView {
    let locale = use_i18n().locale;
    let Some(space) = space else {
        return view! { <span class="space-meta-line">{move || t(locale.get(), "真实地点空间", "Real-place Space")}</span> }.into_any();
    };
    let (kind_zh, kind_en) = space_type_words(&space.space_type);
    let location = join_location(&space);
    view! {
        <div class="space-meta-line">
            <span>{move || t(locale.get(), kind_zh, kind_en)}</span>
            {(!location.is_empty()).then(|| view! { <span>{location}</span> })}
        </div>
    }
    .into_any()
}

#[component]
fn SpaceGuideList(
    guides: Vec<instant_domain::guides::GuideSummary>,
    write_href: String,
) -> impl IntoView {
    let locale = use_i18n().locale;
    let empty_write_href = write_href.clone();
    view! {
        <section id="space-guides" class="space-guides-card" aria-label="Space guides">
            <div class="card-head-inline">
                <div>
                    <p class="survey-kicker">{move || t(locale.get(), "这个地点的记录", "Records for this place")}</p>
                    <h2>{move || t(locale.get(), "空间志", "Guides")}</h2>
                    <p>{move || t(locale.get(), "路线、时间、花费和避坑，写下来就留在这个地点上，下一个人不用重新踩一遍。", "Routes, timing, cost, and pitfalls stay attached to this place so the next visitor does not start over.")}</p>
                </div>
                <a class="button button-secondary-light" href=write_href>{move || t(locale.get(), "写一篇", "Write one")}</a>
            </div>
            {if guides.is_empty() {
                view! {
                    <div class="empty-state compact-empty">
                        <strong>{move || t(locale.get(), "这个地点还没有人写过", "Nobody has written about this place yet")}</strong>
                        <span>{move || t(locale.get(), "志只能在空间里创建：从这里开始写，它会自动挂在这个地点下。", "Records are created inside a Space. Start here and it is attached to this place automatically.")}</span>
                        <a class="button button-primary" href=empty_write_href>{move || t(locale.get(), "在这个空间里写第一篇", "Write the first guide here")}</a>
                    </div>
                }.into_any()
            } else {
                view! {
                    <ul class="guide-list space-guide-list">
                        <For
                            each=move || guides.clone()
                            key=|guide| guide.id
                            children=move |guide| {
                                let title_zh = guide.title_zh.clone();
                                let title_en = guide.title_en.clone();
                                let href = format!("/inspace/guides/{}", guide.id);
                                let edit_href = format!("/inspace/guides/{}/edit", guide.id);
                                let can_edit = guide.can_edit;
                                view! {
                                    <li>
                                        <a class="guide-list-link" href=href>
                                            <strong>{move || localize_optional(locale.get(), &title_zh, title_en.as_deref())}</strong>
                                        </a>
                                        {can_edit.then(|| view! {
                                            <div class="guide-list-actions">
                                                <a class="button button-secondary-light" href=edit_href>{move || t(locale.get(), "编辑", "Edit")}</a>
                                            </div>
                                        })}
                                    </li>
                                }
                            }
                        />
                    </ul>
                }.into_any()
            }}
        </section>
    }
}

#[component]
fn CommunityLinks(space_name: String) -> impl IntoView {
    let locale = use_i18n().locale;
    view! {
        <aside class="community-links" aria-label=move || t(locale.get(), "空间社群链接", "Space community links")>
            <div>
                <strong>{move || t(locale.get(), "社群获取实时密码", "Get live password in community")}</strong>
                <p>{move || t(locale.get(), "远程用户可看公告；具体空间讨论组需主理人审核后才能看到密码和更新。", "Remote users can read announcements; each space group requires host approval before passwords and updates are visible.")}</p>
            </div>
            <div class="community-link-actions">
                <a class="button button-secondary-light" href="https://discord.gg/zsmYWvXyy" target="_blank" rel="noreferrer">"Discord 社群"</a>
                <a class="button button-secondary-light" href="https://pd.qq.com/s/8ru51ih0m?b=9" target="_blank" rel="noreferrer">"QQ 频道【即时空间】"</a>
            </div>
            <p class="community-links-note">
                {move || format!(
                    "{}「{}」{}",
                    t(locale.get(), "在社群内搜索空间名", "Search the community for space"),
                    space_name,
                    t(locale.get(), "获取进入密码", "to get the access password")
                )}
            </p>
        </aside>
    }
}
