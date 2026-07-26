use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use crate::components::private_verify::PrivateVerify;
use crate::components::space_share::SpaceSharePanel;
use crate::i18n::{localize_optional, t, use_i18n};
use crate::server::chat::{check_space_access, list_chat_messages, send_chat_message};
use crate::server::guides::list_space_guides;
use crate::server::spaces::{get_space_for_guide, SpaceMarker};

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

#[component]
pub fn SpacePage() -> impl IntoView {
    let locale = use_i18n().locale;
    let params = use_params_map();
    let space_id =
        Memo::new(move |_| params.with(|params| params.get("space_id").unwrap_or_default()));
    let refresh = RwSignal::new(0u32);

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
    let space_meta = Resource::new(
        move || space_id.get(),
        |space_id| async move {
            if space_id.is_empty() {
                None
            } else {
                get_space_for_guide(space_id).await.ok().flatten()
            }
        },
    );

    view! {
        <main id="main-content" class="page space-detail-page">
            <Suspense fallback=move || view! { <p class="space-section-loading">{move || t(locale.get(), "正在检查空间访问权限", "Checking space access")}</p> }>
                {move || Suspend::new(async move {
                    match access.await {
                        Some(state) if state.allowed => {
                            let space_name = state.space_name.clone();
                            let community_space_name = space_name.clone();
                            let share_space_name = space_name.clone();
                            let title_space_name = space_name.clone();
                            let is_public = state.is_public;
                            let id = state.space_id.to_string();
                            let guide_href = format!("/inspace/guides/new?space_id={id}");
                            let chat_href = format!("/inspace/spaces/{id}/chat");
                            let nav_chat_href = chat_href.clone();
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
                                                <h1>{title_space_name}</h1>
                                                <Suspense fallback=move || view! { <span class="space-meta-line">{move || t(locale.get(), "正在加载地点信息…", "Loading place details…")}</span> }>
                                                    {move || Suspend::new(async move {
                                                        view! { <SpaceMetaLine space=space_meta.await /> }
                                                    })}
                                                </Suspense>
                                                <p class="space-app-header-desc">{move || t(locale.get(), "这里是这个地点的全部记录。先读攻略，再去讨论区问现场。", "Everything recorded about this place. Read the guides first, then ask the discussion room about right now.")}</p>
                                            </div>
                                            <div class="space-app-action-row">
                                                <a class="button button-primary" href=guide_href.clone()>{move || t(locale.get(), "写一篇攻略", "Write a guide")}</a>
                                                <a class="button button-secondary-light" href=chat_href.clone()>{move || t(locale.get(), "进入讨论区", "Open discussion")}</a>
                                            </div>
                                        </div>
                                    </header>

                                    <div class="space-detail-layout">
                                        <div class="space-detail-main">
                                            <Suspense fallback=move || view! { <div class="space-section-loading">{move || t(locale.get(), "正在加载空间攻略…", "Loading Space guides…")}</div> }>
                                                {move || {
                                                    let write_href = guide_href.clone();
                                                    Suspend::new(async move {
                                                        let items = guides.await;
                                                        view! { <SpaceGuideList guides=items write_href=write_href /> }
                                                    })
                                                }}
                                            </Suspense>

                                            <section class="space-discussion-entry">
                                                <p class="survey-kicker">{move || t(locale.get(), "实时补充", "Live context")}</p>
                                                <h2>{move || t(locale.get(), "现场讨论", "On-site discussion")}</h2>
                                                <p>{move || t(locale.get(), "讨论区是独立页面，只谈当天的变化和临时问题：今天哪个入口开、排队多久、天气如何。有长期价值的回答，请整理回攻略。", "The discussion room is its own page for today’s changes: which entrance is open, how long the queue is, what the weather is doing. Durable answers belong in a guide.")}</p>
                                                <a class="button button-primary" href=nav_chat_href>{move || t(locale.get(), "进入讨论区", "Open discussion")}</a>
                                            </section>
                                        </div>

                                        <aside id="space-share" class="space-detail-side" aria-label=move || t(locale.get(), "空间工具", "Space tools")>
                                            <SpaceSharePanel
                                                space_id=id.clone()
                                                space_name=share_space_name
                                                compact=true
                                            />
                                            <CommunityLinks space_name=community_space_name />
                                        </aside>
                                    </div>
                                </article>
                            }.into_any()
                        }
                        Some(state) => {
                            let space_name = state.space_name.clone();
                            let community_space_name = space_name.clone();
                            let id = state.space_id.to_string();
                            view! {
                                <section class="form">
                                    <h1>{move || t(locale.get(), "私密空间需要访问码", "Private Space requires an access code")}</h1>
                                    <p>{move || t(locale.get(), "验证后进入这个地点的空间详情：攻略、分享入口、社群和讨论都会在这里。", "After verification, enter this place’s Space detail: guides, share entry, community, and discussion are all here.")}</p>
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

    let send = Action::new(move |body: &String| {
        let space_id = space_id.get();
        let body = body.clone();
        async move { send_chat_message(space_id, body).await }
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
                            let space_name = state.space_name.clone();
                            let verify_space_name = space_name.clone();
                            let id = state.space_id.to_string();
                            let messages_space_id = id.clone();
                            let is_public = state.is_public;
                            let back_href = format!("/inspace/spaces/{id}");
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
                                    </header>

                                    <Suspense fallback=move || view! { <p class="space-section-loading">{move || t(locale.get(), "正在加载讨论…", "Loading discussion…")}</p> }>
                                        {move || {
                                            let realtime_space_id = messages_space_id.clone();
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
                                                                        view! {
                                                                            <article class="chat-message" data-message-id=message.id.to_string()>
                                                                                <span class="chat-avatar" aria-hidden="true">{monogram}</span>
                                                                                <div class="chat-message-body">
                                                                                    <p class="chat-message-meta">
                                                                                        <strong>{message.sender}</strong>
                                                                                        <time datetime=day.clone()>{clock}</time>
                                                                                    </p>
                                                                                    <p class="chat-message-text">{message.body}</p>
                                                                                </div>
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
                            let space_name = state.space_name.clone();
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

#[component]
fn SpaceMetaLine(space: Option<SpaceMarker>) -> impl IntoView {
    let locale = use_i18n().locale;
    let Some(space) = space else {
        return view! { <span class="space-meta-line">{move || t(locale.get(), "真实地点空间", "Real-place Space")}</span> }.into_any();
    };
    let (kind_zh, kind_en) = match space.space_type {
        instant_domain::spaces::SpaceType::Scenic => ("景点", "Scenic"),
        instant_domain::spaces::SpaceType::Food => ("美食", "Food"),
        instant_domain::spaces::SpaceType::Park => ("公园", "Park"),
        instant_domain::spaces::SpaceType::Transit => ("交通", "Transit"),
        instant_domain::spaces::SpaceType::Event => ("活动", "Event"),
        instant_domain::spaces::SpaceType::Custom => ("其他", "Other"),
    };
    let location = [
        space.country,
        space.province,
        space.city,
        space.district,
        space.spot_name,
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
    .join(" · ");
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
                    <h2>{move || t(locale.get(), "空间攻略", "Space guides")}</h2>
                    <p>{move || t(locale.get(), "路线、时间、花费、避坑——写下来就留在这个地点上，下一个人不用重新踩一遍。", "Routes, timing, cost, pitfalls — written once, they stay attached to this place so the next visitor doesn’t start over.")}</p>
                </div>
                <a class="button button-secondary-light" href=write_href>{move || t(locale.get(), "写一篇", "Write one")}</a>
            </div>
            {if guides.is_empty() {
                view! {
                    <div class="empty-state compact-empty">
                        <strong>{move || t(locale.get(), "这个地点还没有人写过", "Nobody has written about this place yet")}</strong>
                        <span>{move || t(locale.get(), "攻略只能在空间里创建：从这里开始写，它会自动挂在这个地点下。", "Guides are created inside a Space. Start here and the guide is attached to this place automatically.")}</span>
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
