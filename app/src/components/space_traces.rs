use instant_domain::traces::{
    CapsuleOpenResult, CapsuleSealResult, CapsuleSummary, PresenceProof, SpaceChronicle, Trace,
    TRACE_MAX_CHARS,
};
use leptos::prelude::*;

use crate::components::presence::{
    detect_scan, request_location, CodeState, PresenceState, PresenceStatus,
};
use crate::i18n::{t, use_i18n, Locale};
use crate::server::traces::{
    check_onsite_code, leave_trace, list_capsules, list_traces, open_capsule, seal_capsule,
    TracePage,
};

const PAGE_SIZE: i32 = 12;

/// `2026-07-26 18:12:04.1 +00:00:00` -> `2026-07-26`.
fn stamp_date(value: impl ToString) -> String {
    value
        .to_string()
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_string()
}

fn proof_label(locale: Locale, proof: PresenceProof) -> &'static str {
    proof.label(locale == Locale::Zh)
}

/// The guest book, the wall, and the sealed letters: everything a place keeps
/// after the people leave.
#[component]
pub fn SpaceTraces(space_id: String, space_name: String) -> impl IntoView {
    let locale = use_i18n().locale;
    let presence = PresenceState::new();
    let refresh = RwSignal::new(0u32);
    let page = RwSignal::new(1i32);

    // A QR arrival is only knowable on the client, after hydration.
    Effect::new(move |_| {
        if detect_scan() {
            presence.scanned.set(true);
        }
    });

    let traces_id = space_id.clone();
    let traces = Resource::new(
        move || (traces_id.clone(), page.get(), refresh.get()),
        |(space_id, page, _)| async move {
            list_traces(space_id, page, PAGE_SIZE)
                .await
                .unwrap_or(TracePage {
                    items: Vec::new(),
                    total: 0,
                    chronicle: SpaceChronicle {
                        trace_count: 0,
                        on_site_count: 0,
                        capsule_count: 0,
                        capsule_opened_count: 0,
                        first_trace_at: None,
                        first_trace_author: None,
                        latest_trace_at: None,
                    },
                })
        },
    );

    let capsules_id = space_id.clone();
    let capsules = Resource::new(
        move || (capsules_id.clone(), refresh.get()),
        |(space_id, _)| async move { list_capsules(space_id).await.unwrap_or_default() },
    );

    view! {
        <section id="space-traces" class="space-traces" aria-label=move || t(locale.get(), "空间留痕", "What this place keeps")>
            <Suspense fallback=move || view! { <p class="space-section-loading">{move || t(locale.get(), "正在读取这个地方的记录…", "Reading this place’s record…")}</p> }>
                {move || Suspend::new(async move {
                    let page_data = traces.await;
                    view! { <Chronicle chronicle=page_data.chronicle /> }
                })}
            </Suspense>

            <PresenceBar presence=presence space_id=space_id.clone() />

            <TraceComposer
                space_id=space_id.clone()
                presence=presence
                on_written=Callback::new(move |_| {
                    page.set(1);
                    refresh.update(|value| *value += 1);
                })
            />

            <Suspense fallback=move || view! { <p class="space-section-loading">{move || t(locale.get(), "正在翻开留言本…", "Opening the guest book…")}</p> }>
                {move || Suspend::new(async move {
                    let page_data = traces.await;
                    view! { <TraceList page_data=page_data page=page /> }
                })}
            </Suspense>

            <Suspense fallback=|| ()>
                {move || {
                    let space_id = space_id.clone();
                    let space_name = space_name.clone();
                    Suspend::new(async move {
                        let items = capsules.await;
                        view! {
                            <CapsuleShelf
                                space_id=space_id
                                space_name=space_name
                                capsules=items
                                presence=presence
                                on_changed=Callback::new(move |_| refresh.update(|value| *value += 1))
                            />
                        }
                    })
                }}
            </Suspense>
        </section>
    }
}

/// The standing record of a place, written as a line of prose rather than a
/// dashboard. An empty Space says so plainly, because "nobody has written here
/// yet" is the most interesting thing it can say.
#[component]
fn Chronicle(chronicle: SpaceChronicle) -> impl IntoView {
    let locale = use_i18n().locale;
    let untouched = chronicle.is_untouched();
    let trace_count = chronicle.trace_count;
    let on_site_count = chronicle.on_site_count;
    let capsule_count = chronicle.capsule_count;
    let opened_count = chronicle.capsule_opened_count;
    let first_at = chronicle.first_trace_at.map(stamp_date);
    let first_author = chronicle.first_trace_author.clone();

    if untouched {
        return view! {
            <div class="space-chronicle is-untouched">
                <p class="space-chronicle-lead">
                    {move || t(locale.get(), "还没有人在这里留下任何东西。", "Nobody has left anything here yet.")}
                </p>
                <p class="space-chronicle-note">
                    {move || t(
                        locale.get(),
                        "你可以是第一个。第一条留痕会永久记在这个地点名下。",
                        "You could be the first. The first entry is recorded under this place forever.",
                    )}
                </p>
            </div>
        }
        .into_any();
    }

    view! {
        <div class="space-chronicle">
            <dl class="space-chronicle-figures">
                <div>
                    <dt>{move || t(locale.get(), "留痕", "Entries")}</dt>
                    <dd>{trace_count}</dd>
                </div>
                <div>
                    <dt>{move || t(locale.get(), "现场留下", "Left on site")}</dt>
                    <dd>{on_site_count}</dd>
                </div>
                {(capsule_count > 0).then(|| view! {
                    <div>
                        <dt>{move || t(locale.get(), "胶囊", "Capsules")}</dt>
                        <dd>{format!("{opened_count} / {capsule_count}")}</dd>
                    </div>
                })}
            </dl>
            {first_at.map(|date| {
                let author = first_author.clone().unwrap_or_default();
                view! {
                    <p class="space-chronicle-first">
                        {move || t(locale.get(), "第一个到这儿留下记录的是 ", "First recorded here by ")}
                        <strong>{author.clone()}</strong>
                        {format!("，{date}")}
                    </p>
                }
            })}
        </div>
    }
    .into_any()
}

/// Shows what we know about the visitor's position, and lets them improve it.
///
/// The strongest thing a visitor can offer is the code posted at the place, so
/// that is the field they see, and geolocation becomes the fallback rather than
/// the headline. A coordinate is asserted by the browser; the code is checked
/// against a hash the browser never sees.
#[component]
fn PresenceBar(presence: PresenceState, space_id: String) -> impl IntoView {
    let locale = use_i18n().locale;

    let check = Action::new(move |input: &(String, String)| {
        let (space_id, code) = input.clone();
        async move { check_onsite_code(space_id, code).await }
    });

    Effect::new(move |_| {
        if let Some(result) = check.value().get() {
            presence.code_state.set(match result {
                Ok(true) => CodeState::Accepted,
                Ok(false) => CodeState::Rejected,
                Err(_) => CodeState::Rejected,
            });
        }
    });

    Effect::new(move |_| {
        if check.pending().get() {
            presence.code_state.set(CodeState::Checking);
        }
    });

    view! {
        <div class="presence-bar" aria-live="polite">
            <div class="presence-state">
                {move || {
                    if presence.scanned.get() {
                        view! {
                            <span class="presence-badge is-strong">
                                {move || t(locale.get(), "扫码到场", "Scanned on site")}
                            </span>
                        }.into_any()
                    } else if presence.code_state.get() == CodeState::Accepted {
                        view! {
                            <span class="presence-badge is-strong">
                                {move || t(locale.get(), "现场口令已确认", "On-site code confirmed")}
                            </span>
                        }.into_any()
                    } else {
                        match presence.status.get() {
                            PresenceStatus::Located => view! {
                                <span class="presence-badge is-strong">
                                    {move || t(locale.get(), "已取得你的位置", "Location confirmed")}
                                </span>
                            }.into_any(),
                            PresenceStatus::Locating => view! {
                                <span class="presence-badge">
                                    {move || t(locale.get(), "正在定位…", "Locating…")}
                                </span>
                            }.into_any(),
                            PresenceStatus::Denied => view! {
                                <span class="presence-badge is-weak">
                                    {move || t(locale.get(), "定位被拒绝，可以照常留言", "Location denied — you can still write")}
                                </span>
                            }.into_any(),
                            PresenceStatus::Unavailable => view! {
                                <span class="presence-badge is-weak">
                                    {move || t(locale.get(), "这台设备无法定位", "This device cannot locate")}
                                </span>
                            }.into_any(),
                            PresenceStatus::Idle => view! {
                                <span class="presence-badge is-weak">
                                    {move || t(locale.get(), "未确认你是否在现场", "Not confirmed on site")}
                                </span>
                            }.into_any(),
                        }
                    }
                }}
            </div>
            <Show when=move || presence.code_state.get() != CodeState::Accepted>
                <form
                    class=move || if presence.code_state.get() == CodeState::Rejected {
                        "presence-code is-rejected"
                    } else {
                        "presence-code"
                    }
                    on:submit={
                        let space_id = space_id.clone();
                        move |ev| {
                            ev.prevent_default();
                            let code = presence.onsite_code.get().trim().to_string();
                            if !code.is_empty() {
                                check.dispatch((space_id.clone(), code));
                            }
                        }
                    }
                >
                    <label class="field-label">
                        <span>{move || t(locale.get(), "现场口令", "On-site code")}</span>
                        <input
                            type="text"
                            inputmode="numeric"
                            autocomplete="off"
                            maxlength="12"
                            prop:value=move || presence.onsite_code.get()
                            placeholder=move || t(locale.get(), "WiFi 名里的 6 位数字", "6 digits from the WiFi name")
                            on:input=move |ev| {
                                presence.onsite_code.set(event_target_value(&ev));
                                presence.code_state.set(CodeState::Untried);
                            }
                        />
                    </label>
                    <button
                        type="submit"
                        class="button button-secondary-light"
                        disabled=move || presence.onsite_code.get().trim().is_empty() || check.pending().get()
                    >
                        <span class=move || if check.pending().get() { "is-checking" } else { "" }>
                            {move || if check.pending().get() {
                                t(locale.get(), "核对中", "Checking")
                            } else {
                                t(locale.get(), "确认", "Confirm")
                            }}
                        </span>
                    </button>
                    <p class="presence-code-hint">
                        {move || match presence.code_state.get() {
                            CodeState::Rejected => t(
                                locale.get(),
                                "这个口令不对。它就写在这儿的 WiFi 名字里（InstantSpace_ 后面那六位）。",
                                "That code is wrong. It is in the WiFi name here — the six digits after InstantSpace_.",
                            ),
                            _ => t(
                                locale.get(),
                                "打开 WiFi 列表，找到 InstantSpace_ 开头的名字，后面六位就是。人不在这儿是看不到的。",
                                "Open your WiFi list and find the name starting with InstantSpace_ — the six digits are the code. You cannot see it from anywhere else.",
                            ),
                        }}
                    </p>
                </form>
            </Show>

            <div class="presence-actions">
                <Show when=move || presence.status.get() != PresenceStatus::Located>
                    <button
                        type="button"
                        class="button button-secondary-light"
                        on:click=move |_| request_location(presence)
                    >
                        {move || t(locale.get(), "验证当前位置", "Verify current location")}
                    </button>
                </Show>
                <label class="presence-discord">
                    <input
                        type="checkbox"
                        prop:checked=move || presence.discord_member.get()
                        on:change=move |ev| presence.discord_member.set(event_target_checked(&ev))
                    />
                    <span>{move || t(locale.get(), "我是这个空间 Discord 社群成员", "I’m in this Space’s Discord")}</span>
                </label>
            </div>
        </div>
    }
}

#[component]
fn TraceComposer(
    space_id: String,
    presence: PresenceState,
    on_written: Callback<()>,
) -> impl IntoView {
    let locale = use_i18n().locale;
    let body = RwSignal::new(String::new());
    let weather = RwSignal::new(String::new());
    let error = RwSignal::new(None::<String>);

    #[allow(clippy::type_complexity)]
    let write = Action::new(
        move |input: &(
            String,
            String,
            String,
            bool,
            Option<f64>,
            Option<f64>,
            bool,
            Option<String>,
        )| {
            let (space_id, body, weather, scanned, lat, lng, discord, code) = input.clone();
            async move {
                leave_trace(
                    space_id,
                    body,
                    (!weather.trim().is_empty()).then_some(weather),
                    scanned,
                    lat,
                    lng,
                    discord,
                    code,
                    None,
                )
                .await
            }
        },
    );

    Effect::new(move |_| {
        if let Some(result) = write.value().get() {
            match result {
                Ok(_) => {
                    body.set(String::new());
                    weather.set(String::new());
                    error.set(None);
                    on_written.run(());
                }
                Err(err) => {
                    let message = err.to_string();
                    error.set(Some(if message.contains("login required") {
                        t(
                            locale.get_untracked(),
                            "先登录再留下记录。",
                            "Sign in before writing.",
                        )
                        .to_string()
                    } else if message.contains("too long") {
                        t(
                            locale.get_untracked(),
                            "写得有点长了，精简一下。",
                            "That is a little long — trim it.",
                        )
                        .to_string()
                    } else {
                        message
                    }));
                }
            }
        }
    });

    let remaining = Memo::new(move |_| TRACE_MAX_CHARS as i64 - body.get().chars().count() as i64);

    view! {
        <form
            class="trace-composer"
            on:submit=move |ev| {
                ev.prevent_default();
                let text = body.get().trim().to_string();
                if text.is_empty() {
                    return;
                }
                write.dispatch((
                    space_id.clone(),
                    text,
                    weather.get(),
                    presence.scanned.get(),
                    presence.lat.get(),
                    presence.lng.get(),
                    presence.discord_member.get(),
                    presence.code_claim(),
                ));
            }
        >
            <label class="field-label">
                <span>{move || t(locale.get(), "在这里留下一句", "Leave a line here")}</span>
                <textarea
                    rows="3"
                    prop:value=move || body.get()
                    placeholder=move || t(
                        locale.get(),
                        "写给之后来这儿的人。你看见了什么，遇到了谁。",
                        "Write to whoever comes here next. What you saw, who you met.",
                    )
                    on:input=move |ev| body.set(event_target_value(&ev))
                ></textarea>
            </label>
            <div class="trace-composer-row">
                <label class="field-label trace-weather-field">
                    <span>{move || t(locale.get(), "此刻天气（可留空）", "Weather right now (optional)")}</span>
                    <input
                        type="text"
                        maxlength="40"
                        prop:value=move || weather.get()
                        placeholder=move || t(locale.get(), "例如：小雨，风很大", "e.g. light rain, very windy")
                        on:input=move |ev| weather.set(event_target_value(&ev))
                    />
                </label>
                <div class="trace-composer-submit">
                    <span class=move || if remaining.get() < 0 { "trace-remaining is-over" } else { "trace-remaining" }>
                        {move || remaining.get()}
                    </span>
                    <button
                        class="button button-primary"
                        type="submit"
                        disabled=move || body.get().trim().is_empty() || remaining.get() < 0 || write.pending().get()
                    >
                        {move || if write.pending().get() {
                            t(locale.get(), "正在留下…", "Writing…")
                        } else {
                            t(locale.get(), "留下", "Leave it")
                        }}
                    </button>
                </div>
            </div>
            {move || error.get().map(|message| view! { <p class="form-error">{message}</p> })}
        </form>
    }
}

#[component]
fn TraceList(page_data: TracePage, page: RwSignal<i32>) -> impl IntoView {
    let locale = use_i18n().locale;
    let total = page_data.total;
    let items = page_data.items;

    if items.is_empty() {
        return view! {
            <p class="trace-empty">
                {move || t(locale.get(), "这一页还没有内容。", "Nothing on this page yet.")}
            </p>
        }
        .into_any();
    }

    let has_more = i64::from(page.get_untracked() * PAGE_SIZE) < total;

    view! {
        <ol class="trace-list">
            {items.into_iter().map(|trace| view! { <TraceEntry trace=trace /> }).collect_view()}
        </ol>
        <div class="trace-pager">
            <button
                type="button"
                class="pagination-control"
                disabled=move || page.get() <= 1
                on:click=move |_| page.update(|value| *value = (*value - 1).max(1))
            >
                {move || t(locale.get(), "更近的", "Newer")}
            </button>
            <span class="trace-pager-count">
                {move || if locale.get() == Locale::Zh {
                    format!("共 {total} 条")
                } else {
                    format!("{total} in all")
                }}
            </span>
            <button
                type="button"
                class="pagination-control"
                disabled=!has_more
                on:click=move |_| page.update(|value| *value += 1)
            >
                {move || t(locale.get(), "更早的", "Older")}
            </button>
        </div>
    }
    .into_any()
}

#[component]
fn TraceEntry(trace: Trace) -> impl IntoView {
    let locale = use_i18n().locale;
    let on_site = trace.proof.is_on_site();
    let proof = trace.proof;
    let date = stamp_date(trace.created_at);
    let weather = trace.weather.clone();
    let author = trace.author_name.clone();
    let body = trace.body.clone();

    view! {
        <li class=if on_site { "trace-entry is-on-site" } else { "trace-entry" }>
            <p class="trace-body">{body}</p>
            <p class="trace-meta">
                <span class="trace-author">{author}</span>
                <span class="trace-date">{date}</span>
                {weather.map(|value| view! { <span class="trace-weather">{value}</span> })}
                <span class=if on_site { "trace-proof is-on-site" } else { "trace-proof" }>
                    {move || proof_label(locale.get(), proof)}
                </span>
            </p>
        </li>
    }
}

/// The sealed letters. A stranger sees who each one is for and nothing else.
#[component]
fn CapsuleShelf(
    space_id: String,
    space_name: String,
    capsules: Vec<CapsuleSummary>,
    presence: PresenceState,
    on_changed: Callback<()>,
) -> impl IntoView {
    let locale = use_i18n().locale;
    let composing = RwSignal::new(false);
    let sealed_count = capsules.iter().filter(|c| c.is_sealed()).count();

    view! {
        <section class="capsule-shelf" aria-label=move || t(locale.get(), "时间胶囊", "Time capsules")>
            <header class="capsule-shelf-head">
                <div>
                    <p class="survey-kicker">{move || t(locale.get(), "埋在这里", "Buried here")}</p>
                    <h3>{move || t(locale.get(), "时间胶囊", "Time capsules")}</h3>
                    <p class="capsule-shelf-note">
                        {move || t(
                            locale.get(),
                            "写一封信留在这个地点。收信人必须亲自站到这儿，并说出你私下告诉他的口令，才能打开。",
                            "Leave a letter at this place. To open it the recipient must stand here in person and say the passphrase you gave them privately.",
                        )}
                    </p>
                </div>
                <button
                    type="button"
                    class="button button-secondary-light"
                    on:click=move |_| composing.update(|value| *value = !*value)
                >
                    {move || if composing.get() {
                        t(locale.get(), "收起", "Close")
                    } else {
                        t(locale.get(), "埋一个胶囊", "Bury a capsule")
                    }}
                </button>
            </header>

            <Show when=move || composing.get()>
                <CapsuleComposer
                    space_id=space_id.clone()
                    space_name=space_name.clone()
                    presence=presence
                    on_sealed=Callback::new(move |_| {
                        composing.set(false);
                        on_changed.run(());
                    })
                />
            </Show>

            {if capsules.is_empty() {
                view! {
                    <p class="capsule-empty">
                        {move || t(locale.get(), "这里还没有人埋下东西。", "Nothing is buried here yet.")}
                    </p>
                }.into_any()
            } else {
                view! {
                    <ul class="capsule-list">
                        {capsules.into_iter().map(|capsule| view! {
                            <CapsuleCard capsule=capsule presence=presence on_changed=on_changed />
                        }).collect_view()}
                    </ul>
                    {(sealed_count > 0).then(|| view! {
                        <p class="capsule-hint">
                            {move || t(
                                locale.get(),
                                "如果其中一个是给你的，走到这个地点再来打开。",
                                "If one of these is for you, come to this place to open it.",
                            )}
                        </p>
                    })}
                }.into_any()
            }}
        </section>
    }
}

#[component]
fn CapsuleComposer(
    space_id: String,
    space_name: String,
    presence: PresenceState,
    on_sealed: Callback<()>,
) -> impl IntoView {
    let locale = use_i18n().locale;
    let recipient = RwSignal::new(String::new());
    let body = RwSignal::new(String::new());
    let passphrase = RwSignal::new(String::new());
    let radius = RwSignal::new(300i32);
    let opens_at = RwSignal::new(String::new());
    let error = RwSignal::new(None::<String>);

    let seal = Action::new(
        move |input: &(
            String,
            String,
            String,
            String,
            i32,
            String,
            Option<f64>,
            Option<f64>,
            Option<String>,
        )| {
            let (space_id, recipient, body, passphrase, radius, opens_at, lat, lng, onsite_code) =
                input.clone();
            async move {
                seal_capsule(
                    space_id,
                    recipient,
                    body,
                    passphrase,
                    radius,
                    (!opens_at.trim().is_empty()).then_some(opens_at),
                    lat,
                    lng,
                    onsite_code,
                )
                .await
            }
        },
    );

    Effect::new(move |_| {
        if let Some(result) = seal.value().get() {
            match result {
                Ok(CapsuleSealResult::Sealed { .. }) => {
                    recipient.set(String::new());
                    body.set(String::new());
                    passphrase.set(String::new());
                    opens_at.set(String::new());
                    error.set(None);
                    on_sealed.run(());
                }
                Ok(CapsuleSealResult::OnsiteCodeRequired) => error.set(Some(
                    t(
                        locale.get_untracked(),
                        "先输入并验证写在现场 Wi-Fi 名称里的口令。",
                        "First verify the code shown in the on-site Wi-Fi name.",
                    )
                    .to_string(),
                )),
                Ok(CapsuleSealResult::LocationRequired) => error.set(Some(
                    t(
                        locale.get_untracked(),
                        "还需要允许定位，确认你确实站在这个地点附近。",
                        "Location is also required to confirm you are near this place.",
                    )
                    .to_string(),
                )),
                Ok(CapsuleSealResult::TooFar { distance_m, radius_m }) => {
                    error.set(Some(if locale.get_untracked() == Locale::Zh {
                        format!("你距离这里约 {distance_m:.0} 米，需要走到 {radius_m} 米以内才能埋下胶囊。")
                    } else {
                        format!("You are about {distance_m:.0} m away. Get within {radius_m} m to bury the capsule.")
                    }));
                }
                Ok(CapsuleSealResult::SpaceLocationUnavailable) => error.set(Some(
                    t(
                        locale.get_untracked(),
                        "这个空间还没有可验证的位置，请联系主理人修正坐标。",
                        "This Space has no verifiable location yet. Ask its host to correct the coordinates.",
                    )
                    .to_string(),
                )),
                Err(err) => {
                    let message = err.to_string();
                    error.set(Some(if message.contains("login required") {
                        t(
                            locale.get_untracked(),
                            "先登录才能埋胶囊。",
                            "Sign in to bury a capsule.",
                        )
                        .to_string()
                    } else if message.contains("passphrase too short") {
                        t(
                            locale.get_untracked(),
                            "口令太短了。",
                            "That passphrase is too short.",
                        )
                        .to_string()
                    } else {
                        message
                    }));
                }
            }
        }
    });

    view! {
        <form
            class="capsule-composer"
            on:submit=move |ev| {
                ev.prevent_default();
                seal.dispatch((
                    space_id.clone(),
                    recipient.get().trim().to_string(),
                    body.get().trim().to_string(),
                    passphrase.get().trim().to_string(),
                    radius.get(),
                    opens_at.get(),
                    presence.lat.get(),
                    presence.lng.get(),
                    presence.code_claim(),
                ));
            }
        >
            <section class="capsule-bury-gate" aria-labelledby="capsule-bury-gate-title">
                <div>
                    <p class="survey-kicker" id="capsule-bury-gate-title">
                        {move || t(locale.get(), "先证明你正在这里", "First prove you are here")}
                    </p>
                    <p>
                        {move || t(
                            locale.get(),
                            "埋胶囊会成为这个地点的一段历史，因此必须同时通过现场 Wi-Fi 口令和本地定位。",
                            "Burying a capsule becomes part of this place's history, so both the on-site Wi-Fi code and a local location fix are required.",
                        )}
                    </p>
                </div>
                <ul>
                    <li class=move || if presence.code_state.get() == CodeState::Accepted { "is-ready" } else { "" }>
                        <strong>{move || t(locale.get(), "Wi-Fi 现场口令", "On-site Wi-Fi code")}</strong>
                        <span>{move || if presence.code_state.get() == CodeState::Accepted {
                            t(locale.get(), "已验证", "Verified")
                        } else {
                            t(locale.get(), "请在上方输入并验证", "Verify it above")
                        }}</span>
                    </li>
                    <li class=move || if presence.has_fix() { "is-ready" } else { "" }>
                        <strong>{move || t(locale.get(), "本地定位", "Local location")}</strong>
                        <span>{move || if presence.has_fix() {
                            t(locale.get(), "已取得位置", "Location received")
                        } else {
                            t(locale.get(), "尚未取得位置", "Location not received")
                        }}</span>
                    </li>
                </ul>
                <Show when=move || !presence.has_fix()>
                    <button type="button" class="button button-secondary-light" on:click=move |_| request_location(presence)>
                        {move || if presence.status.get() == PresenceStatus::Locating {
                            t(locale.get(), "正在定位…", "Locating…")
                        } else {
                            t(locale.get(), "验证当前位置", "Verify current location")
                        }}
                    </button>
                </Show>
            </section>

            <label class="field-label">
                <span>{move || t(locale.get(), "这封信是给谁的", "Who is this for")}</span>
                <input
                    type="text"
                    maxlength="80"
                    prop:value=move || recipient.get()
                    placeholder=move || t(locale.get(), "写一个只有对方看得懂的称呼", "A name only they would recognise")
                    on:input=move |ev| recipient.set(event_target_value(&ev))
                />
            </label>
            <label class="field-label">
                <span>{move || t(locale.get(), "信的内容", "The letter")}</span>
                <textarea
                    rows="5"
                    prop:value=move || body.get()
                    on:input=move |ev| body.set(event_target_value(&ev))
                ></textarea>
            </label>
            <label class="field-label">
                <span>{move || t(locale.get(), "口令", "Passphrase")}</span>
                <input
                    type="text"
                    prop:value=move || passphrase.get()
                    placeholder=move || t(locale.get(), "你要亲口告诉他的那句话", "The words you will tell them yourself")
                    on:input=move |ev| passphrase.set(event_target_value(&ev))
                />
            </label>
            <p class="capsule-warning">
                {move || t(
                    locale.get(),
                    "口令只存哈希，服务器也读不出来。忘了就永远打不开——这正是埋下去的意思。",
                    "Only a hash of the passphrase is stored; this server cannot read it back. If it is lost the capsule stays shut — that is what burying means.",
                )}
            </p>
            <div class="capsule-composer-row">
                <label class="field-label">
                    <span>{move || t(locale.get(), "允许多远算到场", "How close counts as here")}</span>
                    <select
                        prop:value=move || radius.get().to_string()
                        on:change=move |ev| {
                            if let Ok(value) = event_target_value(&ev).parse::<i32>() {
                                radius.set(value);
                            }
                        }
                    >
                        <option value="100">{move || t(locale.get(), "100 米 · 就在眼前", "100 m · right at it")}</option>
                        <option value="300">{move || t(locale.get(), "300 米 · 到了这块地方", "300 m · at the place")}</option>
                        <option value="1000">{move || t(locale.get(), "1 公里 · 到了这一带", "1 km · in the area")}</option>
                    </select>
                </label>
                <label class="field-label">
                    <span>{move || t(locale.get(), "在此日期之后才能开（可留空）", "Not before this date (optional)")}</span>
                    <input
                        type="date"
                        prop:value=move || opens_at.get()
                        on:input=move |ev| opens_at.set(event_target_value(&ev))
                    />
                </label>
            </div>
            <p class="capsule-place-note">
                {move || t(locale.get(), "埋在：", "Buried at: ")}
                <strong>{space_name.clone()}</strong>
            </p>
            {move || error.get().map(|message| view! { <p class="form-error">{message}</p> })}
            <button
                class="button button-primary"
                type="submit"
                disabled=move || {
                    recipient.get().trim().is_empty()
                        || body.get().trim().is_empty()
                        || passphrase.get().trim().chars().count() < 2
                        || presence.code_state.get() != CodeState::Accepted
                        || !presence.has_fix()
                        || seal.pending().get()
                }
            >
                {move || if seal.pending().get() {
                    t(locale.get(), "正在封存…", "Sealing…")
                } else {
                    t(locale.get(), "封存并埋下", "Seal and bury")
                }}
            </button>
        </form>
    }
}

#[component]
fn CapsuleCard(
    capsule: CapsuleSummary,
    presence: PresenceState,
    on_changed: Callback<()>,
) -> impl IntoView {
    let locale = use_i18n().locale;
    let capsule_id = capsule.id.to_string();
    let author = capsule.author_name.clone();
    let recipient = capsule.recipient_hint.clone();
    let created = stamp_date(capsule.created_at);
    let radius_m = capsule.radius_m;
    let sealed = capsule.is_sealed();
    let opened_at = capsule.opened_at.map(stamp_date);
    let opened_by = capsule.opened_by_name.clone();
    let opens_at = capsule.opens_at.map(stamp_date);

    let passphrase = RwSignal::new(String::new());
    let attempting = RwSignal::new(false);
    let outcome = RwSignal::new(None::<CapsuleOpenResult>);

    let open = Action::new(
        move |input: &(
            String,
            String,
            Option<f64>,
            Option<f64>,
            bool,
            Option<String>,
        )| {
            let (capsule_id, passphrase, lat, lng, scanned, code) = input.clone();
            async move { open_capsule(capsule_id, passphrase, lat, lng, scanned, code).await }
        },
    );

    // Deliberately does not refresh the shelf. Reloading the list would rebuild
    // this card and throw away the letter the reader just earned; a stale count
    // in the chronicle is a far smaller cost than the letter disappearing the
    // instant it opens.
    let _ = on_changed;
    Effect::new(move |_| {
        if let Some(Ok(result)) = open.value().get() {
            outcome.set(Some(result));
        }
    });

    view! {
        <li class=if sealed { "capsule-card is-sealed" } else { "capsule-card is-open" }>
            <div class="capsule-card-head">
                <span class="capsule-state">
                    {move || if sealed {
                        t(locale.get(), "未开启", "Sealed")
                    } else {
                        t(locale.get(), "已被取走", "Taken")
                    }}
                </span>
                <span class="capsule-date">{created.clone()}</span>
            </div>
            <p class="capsule-recipient">
                <span>{move || t(locale.get(), "给", "For")}</span>
                <strong>{recipient.clone()}</strong>
            </p>
            <p class="capsule-author">
                {move || t(locale.get(), "留下的人：", "Left by ")}
                {author.clone()}
            </p>

            {if sealed {
                let opens_note = opens_at.clone();
                view! {
                    <>
                    {opens_note.map(|date| view! {
                        <p class="capsule-note">
                            {move || t(locale.get(), "此日之后才能开启：", "Not before ")}
                            {date.clone()}
                        </p>
                    })}
                    <Show
                        when=move || attempting.get()
                        fallback=move || view! {
                            <button
                                type="button"
                                class="button button-secondary-light"
                                on:click=move |_| attempting.set(true)
                            >
                                {move || t(locale.get(), "这是给我的", "This one is for me")}
                            </button>
                        }
                    >
                        <div class="capsule-attempt">
                            <PresenceRequirement presence=presence radius_m=radius_m />
                            <label class="field-label">
                                <span>{move || t(locale.get(), "口令", "Passphrase")}</span>
                                <input
                                    type="text"
                                    prop:value=move || passphrase.get()
                                    on:input=move |ev| passphrase.set(event_target_value(&ev))
                                />
                            </label>
                            <button
                                type="button"
                                class="button button-primary"
                                disabled=move || passphrase.get().trim().is_empty() || open.pending().get()
                                on:click={
                                    let capsule_id = capsule_id.clone();
                                    move |_| {
                                        open.dispatch((
                                            capsule_id.clone(),
                                            passphrase.get().trim().to_string(),
                                            presence.lat.get(),
                                            presence.lng.get(),
                                            presence.scanned.get(),
                                            presence.code_claim(),
                                        ));
                                    }
                                }
                            >
                                {move || if open.pending().get() {
                                    t(locale.get(), "正在打开…", "Opening…")
                                } else {
                                    t(locale.get(), "打开", "Open it")
                                }}
                            </button>
                        </div>
                    </Show>
                    {move || outcome.get().map(|result| view! { <CapsuleOutcome result=result /> })}
                    </>
                }.into_any()
            } else {
                view! {
                    <p class="capsule-taken">
                        {opened_by.clone().map(|name| view! { <strong>{name}</strong> })}
                        {opened_at.clone().map(|date| {
                            view! {
                                <span>{format!(" · {date}")}</span>
                            }
                        })}
                    </p>
                }.into_any()
            }}
        </li>
    }
}

/// Tells the visitor, before they type anything, whether being here is settled.
#[component]
fn PresenceRequirement(presence: PresenceState, radius_m: i32) -> impl IntoView {
    let locale = use_i18n().locale;

    view! {
        <div class="capsule-presence">
            {move || {
                if presence.code_state.get() == CodeState::Accepted {
                    view! {
                        <p class="capsule-presence-ok">
                            {move || t(locale.get(), "现场口令已确认。还差他给你的那句话。", "On-site code confirmed. Now the words he gave you.")}
                        </p>
                    }.into_any()
                } else {
                    view! {
                        <div class="capsule-presence-need">
                            <p>
                                {move || t(
                                    locale.get(),
                                    "这封信上着两把锁：一把证明你到了这里，一把证明你是他等的人。",
                                    "This letter has two locks: one proves you got here, one proves you are the person he waited for.",
                                )}
                            </p>
                            <p class="capsule-presence-code">
                                {move || if locale.get() == Locale::Zh {
                                    format!("第一把锁是上面的现场口令——它写在这儿的 WiFi 名字里，站在 {radius_m} 米外都读不到。第二把锁是下面这栏，只有他私下告诉过你。")
                                } else {
                                    format!("The first lock is the on-site code above — it is in the WiFi name here, unreadable from {radius_m} m away. The second is the field below, told to you privately.")
                                }}
                            </p>
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}

#[component]
fn CapsuleOutcome(result: CapsuleOpenResult) -> impl IntoView {
    let locale = use_i18n().locale;

    match result {
        CapsuleOpenResult::Opened {
            body,
            author_name,
            created_at,
        } => view! {
            <div class="capsule-letter">
                <p class="capsule-letter-head">
                    {move || t(locale.get(), "这封信埋于 ", "Buried on ")}
                    {created_at.clone()}
                </p>
                <p class="capsule-letter-body">{body}</p>
                <p class="capsule-letter-sign">{author_name}</p>
            </div>
        }
        .into_any(),
        CapsuleOpenResult::TooFar { distance_m, radius_m } => view! {
            <p class="capsule-result is-far">
                {move || if locale.get() == Locale::Zh {
                    format!("你还在 {distance_m:.0} 米外，得走到 {radius_m} 米以内，才读得到现场口令。")
                } else {
                    format!("You are {distance_m:.0} m away; get within {radius_m} m to read the on-site code.")
                }}
            </p>
        }
        .into_any(),
        CapsuleOpenResult::WrongPassphrase => view! {
            <p class="capsule-result is-wrong">
                {move || t(locale.get(), "你站对了地方，但这不是那句话。", "You are standing in the right place, but those are not the words.")}
            </p>
        }
        .into_any(),
        CapsuleOpenResult::NotYet { opens_at } => view! {
            <p class="capsule-result">
                {move || t(locale.get(), "还不到时候，要等到 ", "Not yet — not before ")}
                {opens_at.clone()}
            </p>
        }
        .into_any(),
        CapsuleOpenResult::PresenceRequired => view! {
            <p class="capsule-result">
                {move || t(locale.get(), "还差现场口令——先在上面把这个地点的口令填对。", "The on-site code is still missing — fill in this place's code above first.")}
            </p>
        }
        .into_any(),
        CapsuleOpenResult::AlreadyOpened { opened_at, opened_by_name } => view! {
            <p class="capsule-result">
                {move || t(locale.get(), "已经被取走了：", "Already taken: ")}
                {opened_by_name.clone().unwrap_or_default()}
                {format!(" · {opened_at}")}
            </p>
        }
        .into_any(),
        CapsuleOpenResult::Locked => view! {
            <p class="capsule-result is-wrong">
                {move || t(locale.get(), "试错太多次，这个胶囊不再回应了。", "Too many wrong guesses; this capsule has stopped answering.")}
            </p>
        }
        .into_any(),
    }
}

/// Lifts one chat line out of the transcript and into the permanent record.
///
/// Discussion is about right now and scrolls away; a carved line is addressed
/// to people who are not here yet. Anything carved this way is marked remote
/// unless the reader happens to be standing at the place, because carving
/// somebody else's words is not evidence that you were there.
#[component]
pub fn CarveButton(space_id: String, message_id: String, body: String) -> impl IntoView {
    let locale = use_i18n().locale;
    let carved = RwSignal::new(false);
    let failed = RwSignal::new(false);

    let carve = Action::new(move |input: &(String, String, String)| {
        let (space_id, body, message_id) = input.clone();
        async move {
            leave_trace(
                space_id,
                body,
                None,
                false,
                None,
                None,
                false,
                None,
                Some(message_id),
            )
            .await
        }
    });

    Effect::new(move |_| {
        if let Some(result) = carve.value().get() {
            match result {
                Ok(_) => {
                    carved.set(true);
                    failed.set(false);
                }
                Err(_) => failed.set(true),
            }
        }
    });

    view! {
        <button
            type="button"
            class=move || if carved.get() { "chat-carve is-carved" } else { "chat-carve" }
            disabled=move || carved.get() || carve.pending().get()
            title=move || if carved.get() {
                t(locale.get(), "已刻进留痕", "Carved into the record")
            } else if failed.get() {
                t(locale.get(), "刻不上去，先登录", "Could not carve — sign in first")
            } else {
                t(locale.get(), "把这句刻进留痕", "Carve this line into the record")
            }
            aria-label=move || t(locale.get(), "把这句刻进留痕", "Carve this line into the record")
            on:click={
                let space_id = space_id.clone();
                let body = body.clone();
                let message_id = message_id.clone();
                move |_| {
                    carve.dispatch((space_id.clone(), body.clone(), message_id.clone()));
                }
            }
        >
            <svg viewBox="0 0 24 24" aria-hidden="true" focusable="false">
                <path d="M4 20l3-1 11-11-2-2L5 17l-1 3z" />
                <path d="M15 6l3 3" />
            </svg>
        </button>
    }
}
