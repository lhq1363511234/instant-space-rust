use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use crate::components::private_verify::PrivateVerify;
use crate::components::space_share::SpaceSharePanel;
use crate::i18n::{localize_optional, t, use_i18n};
use crate::server::chat::{check_space_access, list_chat_messages, send_chat_message};
use crate::server::guides::list_space_guides;

#[component]
pub fn SpacePage() -> impl IntoView {
    let locale = use_i18n().locale;
    let params = use_params_map();
    let space_id =
        Memo::new(move |_| params.with(|params| params.get("space_id").unwrap_or_default()));
    let refresh = RwSignal::new(0u32);
    let message_body = RwSignal::new(String::new());
    let send_error = RwSignal::new(None::<String>);
    let active_space_tab = RwSignal::new("chat");

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
        <main class="page space-chat-page">
            <Suspense fallback=move || view! { <p>{move || t(locale.get(), "正在检查空间访问权限", "Checking space access")}</p> }>
                {move || Suspend::new(async move {
                    match access.await {
                        Some(state) if state.allowed => {
                            let space_name = state.space_name.clone();
                            let community_space_name = space_name.clone();
                            let verify_space_name = space_name.clone();
                            let share_space_name = space_name.clone();
                            let title_space_name = space_name.clone();
                            let is_public = state.is_public;
                            let id = state.space_id.to_string();
                            let messages_space_id = id.clone();
                            let guide_href = format!("/inspace/guides/new?space_id={id}");
                            view! {
                                <section class="chat-shell" aria-label="Space chat">
                                    <div class="page-head space-app-header">
                                        <div>
                                            <p class="eyebrow">{move || if is_public { t(locale.get(), "公共空间", "Public space") } else { t(locale.get(), "私密空间", "Private space") }}</p>
                                            <h1>{title_space_name}</h1>
                                            <p>{move || t(locale.get(), "这是空间内的实时讨论区，消息会即时同步给当前在线成员。", "This is a realtime room; messages sync instantly with everyone currently online.")}</p>
                                        </div>
                                        <div class="page-head-actions">
                                            <span class="chat-realtime-status" data-realtime-status="connecting">
                                                {move || t(locale.get(), "正在连接实时房间…", "Connecting to realtime room…")}
                                            </span>
                                            <span class="chat-online-count" data-realtime-online="0">0 online</span>
                                            <a class="button button-primary" href=guide_href>{move || t(locale.get(), "写空间攻略", "Write guide")}</a>
                                            <a class="button button-secondary-light" href="/inspace">{move || t(locale.get(), "返回地图", "Back to map")}</a>
                                        </div>
                                    </div>

                                    <Suspense fallback=move || view! { <p>{move || t(locale.get(), "正在加载消息", "Loading messages")}</p> }>
                                        {move || {
                                            let realtime_space_id = messages_space_id.clone();
                                            Suspend::new(async move {
                                            let items = messages.await;
                                            view! {
                                                <div
                                                    id="space-chat"
                                                    class="chat-message-list"
                                                    aria-label="Chat messages"
                                                    data-realtime-messages="true"
                                                    data-space-id=realtime_space_id
                                                >
                                                    {if items.is_empty() {
                                                        view! { <p class="empty-state"><span>{move || t(locale.get(), "还没有消息，发第一条攻略线索吧。", "No messages yet. Share the first guide tip.")}</span></p> }.into_any()
                                                    } else {
                                                        view! {
                                                            <For
                                                                each=move || items.clone()
                                                                key=|message| message.id
                                                                children=move |message| view! {
                                                                    <article class="chat-message" attr:data-message-id=message.id.to_string()>
                                                                        <strong>{message.sender}</strong>
                                                                        <p>{message.body}</p>
                                                                        <time>{message.created_at.to_string()}</time>
                                                                    </article>
                                                                }
                                                            />
                                                        }.into_any()
                                                    }}
                                                </div>
                                            }
                                        })}}
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
                                            <span>{move || t(locale.get(), "消息 / 攻略线索", "Message / guide tip")}</span>
                                            <textarea
                                                rows="3"
                                                aria-label=move || t(locale.get(), "聊天消息", "Chat message")
                                                data-chat-input="true"
                                                placeholder=move || t(locale.get(), "例如：这里傍晚 18:30 光线最好，从 3 号口出来最近。", "e.g. Best light is around 18:30; Exit 3 is closest.")
                                                prop:value=move || message_body.get()
                                                on:input=move |ev| message_body.set(event_target_value(&ev))
                                            ></textarea>
                                        </label>
                                        <button class="button button-primary" type="submit">{move || t(locale.get(), "发送", "Send")}</button>
                                        {move || send_error.get().map(|message| view! { <p class="error">{message}</p> })}
                                    </form>
                                    {(!is_public).then(|| view! {
                                        <div class="realtime-reverify" data-private-reverify="true" style="display:none">
                                            <PrivateVerify space_id=id.clone() space_name=verify_space_name.clone() />
                                        </div>
                                    })}

                                    <Suspense fallback=move || view! { <p>{move || t(locale.get(), "正在加载空间攻略", "Loading space guides")}</p> }>
                                        {move || Suspend::new(async move {
                                            let items = guides.await;
                                            view! { <SpaceGuideList guides=items /> }
                                        })}
                                    </Suspense>

                                    <section id="space-share" class="space-app-section space-app-share" aria-label=move || t(locale.get(), "分享空间", "Share space")>
                                        <SpaceSharePanel
                                            space_id=id.clone()
                                            space_name=share_space_name
                                            compact=false
                                        />
                                        <CommunityLinks space_name=community_space_name />
                                    </section>

                                    <nav class="space-app-bottom-nav" aria-label=move || t(locale.get(), "空间快捷导航", "Space quick navigation")>
                                        <a
                                            href="#space-chat"
                                            class=move || if active_space_tab.get() == "chat" { "is-active" } else { "" }
                                            on:click=move |_| active_space_tab.set("chat")
                                        >
                                            <span aria-hidden="true">"●"</span>
                                            <b>{move || t(locale.get(), "讨论", "Chat")}</b>
                                        </a>
                                        <a
                                            href="#space-guides"
                                            class=move || if active_space_tab.get() == "guides" { "is-active" } else { "" }
                                            on:click=move |_| active_space_tab.set("guides")
                                        >
                                            <span aria-hidden="true">"◇"</span>
                                            <b>{move || t(locale.get(), "攻略", "Guides")}</b>
                                        </a>
                                        <a
                                            href="#space-share"
                                            class=move || if active_space_tab.get() == "share" { "is-active" } else { "" }
                                            on:click=move |_| active_space_tab.set("share")
                                        >
                                            <span aria-hidden="true">"↗"</span>
                                            <b>{move || t(locale.get(), "分享", "Share")}</b>
                                        </a>
                                        <a href="/inspace">
                                            <span aria-hidden="true">"⌖"</span>
                                            <b>{move || t(locale.get(), "地图", "Map")}</b>
                                        </a>
                                    </nav>
                                </section>
                            }.into_any()
                        }
                        Some(state) => {
                            let space_name = state.space_name.clone();
                            let community_space_name = space_name.clone();
                            let id = state.space_id.to_string();
                            view! {
                                <section class="form">
                                    <h1>{move || t(locale.get(), "需要访问码", "Access code required")}</h1>
                                    <p>{move || t(locale.get(), "验证后刷新页面也会短期保留访问权限。", "After verification, access is kept briefly even after refresh.")}</p>
                                    <PrivateVerify space_id=id space_name=space_name />
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

#[component]
fn SpaceGuideList(guides: Vec<instant_domain::guides::GuideSummary>) -> impl IntoView {
    let locale = use_i18n().locale;
    view! {
        <section id="space-guides" class="space-guides-card" aria-label="Space guides">
            <div class="card-head-inline">
                <div>
                    <h2>{move || t(locale.get(), "空间攻略", "Space guides")}</h2>
                    <p>{move || t(locale.get(), "一个空间可以对应多篇攻略；编辑已有攻略会更新原文，写新攻略会新增一篇。", "A space can have multiple guides; editing updates an existing guide, while writing a new guide creates another one.")}</p>
                </div>
            </div>
            {if guides.is_empty() {
                view! {
                    <div class="empty-state compact-empty">
                        <span>{move || t(locale.get(), "还没有已发布攻略。", "No published guides yet.")}</span>
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
