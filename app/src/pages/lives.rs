//! 数字生命：云上家 · 在侧 · 追远。
//!
//! 万物有灵，皆有归处。文案遵循 life-distill 的宋式笔法：白描、短句、
//! 留白、时节与地点意象、哀而不伤。

use instant_domain::lives::{CompanionState, LifePrayer};
use leptos::prelude::*;
use leptos_meta::{Meta, Title};
use leptos_router::hooks::use_params_map;

use crate::i18n::{t, use_i18n, Locale};
use crate::server::auth::current_session;
use crate::server::lives::{
    create_companion, create_digital_life, get_digital_life, get_my_cloud_home, leave_prayer,
    list_digital_lives, list_my_companions, list_prayers, update_my_cloud_home, visit_cloud_home,
};

// ---------------------------------------------------------------------------
// 目录页：在侧 / 追远
// ---------------------------------------------------------------------------

#[component]
pub fn LivesPage() -> impl IntoView {
    let locale = use_i18n().locale;
    let tab = RwSignal::new(0u8);
    let reload = RwSignal::new(0u32);
    let page = RwSignal::new(1i64);
    let total_pages = RwSignal::new(1i64);

    let home = Resource::new(
        move || reload.get(),
        |_| async move { get_my_cloud_home().await.ok() },
    );
    let companions = Resource::new(
        move || reload.get(),
        |_| async move { list_my_companions().await.ok().unwrap_or_default() },
    );
    let show_pager = Signal::derive(move || total_pages.get() > 1);
    let can_prev = Signal::derive(move || page.get() > 1);
    let can_next = Signal::derive(move || page.get() < total_pages.get());

    let lives = Resource::new(
        move || (reload.get(), page.get()),
        move |(_, p)| async move {
            let data = list_digital_lives(Some(24), Some((p - 1).max(0) * 24))
                .await
                .ok();
            if let Some(ref d) = data {
                let pages = (d.total / 24 + if d.total % 24 == 0 { 0 } else { 1 }).max(1);
                total_pages.set(pages);
            }
            data
        },
    );

    // 在侧：收留一位家人
    let c_name = RwSignal::new(String::new());
    let c_species = RwSignal::new(String::new());
    let c_birth = RwSignal::new(String::new());
    let c_creating = RwSignal::new(false);
    let c_error = RwSignal::new(String::new());
    let do_create = move || {
        leptos::task::spawn_local(async move {
            c_creating.set(true);
            c_error.set(String::new());
            let name = c_name.get();
            if name.trim().is_empty() {
                c_error.set(t(locale.get(), "给个名字", "Give it a name").to_string());
                c_creating.set(false);
                return;
            }
            match create_companion(
                name,
                Some(c_species.get()),
                None,
                None,
                (!c_birth.get().trim().is_empty()).then(|| c_birth.get()),
                None,
            )
            .await
            {
                Ok(_) => {
                    c_name.set(String::new());
                    c_species.set(String::new());
                    c_birth.set(String::new());
                    reload.update(|v| *v += 1);
                }
                Err(e) => c_error.set(e.to_string()),
            }
            c_creating.set(false);
        });
    };

    // 追远：为离世的家人蒸馏
    let distill_open = RwSignal::new(false);
    let d_companion = RwSignal::new(String::new());
    let d_death = RwSignal::new(String::new());
    let d_epitaph = RwSignal::new(String::new());
    let d_inscription = RwSignal::new(String::new());
    let d_biography = RwSignal::new(String::new());
    let d_lifemap = RwSignal::new(String::new());
    let d_error = RwSignal::new(String::new());
    let d_saving = RwSignal::new(false);

    let open_distill = move |id: String| {
        distill_open.set(true);
        d_companion.set(id);
    };

    let do_distill = move || {
        leptos::task::spawn_local(async move {
            d_saving.set(true);
            d_error.set(String::new());
            if d_death.get().trim().is_empty() {
                d_error.set(t(locale.get(), "记下离世之日", "A date is needed").to_string());
                d_saving.set(false);
                return;
            }
            let chapters = d_biography
                .get()
                .split("\n\n")
                .map(|p| p.trim().to_string())
                .filter(|p| !p.is_empty())
                .enumerate()
                .map(|(i, body)| instant_domain::lives::BiographyChapter {
                    title: format!("{} {}", t(locale.get(), "小传", "Life"), i + 1),
                    body,
                })
                .collect::<Vec<_>>();
            let life_map = d_lifemap
                .get()
                .lines()
                .filter_map(|line| {
                    let mut parts = line.split('｜');
                    let place = parts.next()?.trim().to_string();
                    if place.is_empty() {
                        return None;
                    }
                    let season = parts.next().map(|s| s.trim().to_string());
                    let deed = parts
                        .next()
                        .map(|s| s.trim().to_string())
                        .unwrap_or_default();
                    Some(instant_domain::lives::LifeMapEntry {
                        place,
                        season,
                        deed,
                    })
                })
                .collect::<Vec<_>>();
            match create_digital_life(
                d_companion.get(),
                d_death.get(),
                d_epitaph.get(),
                chapters,
                d_inscription.get(),
                life_map,
                None,
                1,
            )
            .await
            {
                Ok(life) => {
                    distill_open.set(false);
                    d_companion.set(String::new());
                    d_death.set(String::new());
                    d_epitaph.set(String::new());
                    d_inscription.set(String::new());
                    d_biography.set(String::new());
                    d_lifemap.set(String::new());
                    reload.update(|v| *v += 1);
                    let url = format!("/inspace/lives/{}", life.id);
                    let _ = leptos::prelude::window().location().set_href(&url);
                }
                Err(e) => d_error.set(e.to_string()),
            }
            d_saving.set(false);
        });
    };

    view! {
        <Title text=move || t(locale.get(), "数字生命｜万物有灵，皆有归处", "Digital lives | Every soul has a home") />
        <Meta name="description" content=move || t(
            locale.get(),
            "在侧，是与你同行的家人；追远，是慎终追远的一份纪念。inspace 数字生命，为每一段相伴留下归处。",
            "Beside you, the family that travels with you; in memory, a place to remember. inspace digital lives give every bond a home.",
        ) />
        <main id="main-content" class="page lives-page">
            <header class="lives-hero">
                <p class="lives-kicker">{move || t(locale.get(), "数字生命", "Digital lives")}</p>
                <h1>{move || t(locale.get(), "万物有灵，皆有归处。", "Every soul has a home.")}</h1>
                <p class="lives-hero-sub">{move || t(locale.get(), "在侧，随你走过千山；追远，留一份慎终追远。", "Beside you on the road; remembered in stillness.")}</p>
                <div class="lives-tabs" role="tablist" aria-label=move || t(locale.get(), "在侧与追远", "Beside and in memory")>
                    <button type="button" role="tab" aria-selected=move || tab.get() == 0
                        class=move || if tab.get() == 0 { "lives-tab is-active" } else { "lives-tab" }
                        on:click=move |_| tab.set(0)>
                        {move || t(locale.get(), "在侧", "Beside you")}
                    </button>
                    <button type="button" role="tab" aria-selected=move || tab.get() == 1
                        class=move || if tab.get() == 1 { "lives-tab is-active" } else { "lives-tab" }
                        on:click=move |_| { tab.set(1); page.set(1); }>
                        {move || t(locale.get(), "追远", "In memory")}
                    </button>
                </div>
            </header>

            <Show when=move || tab.get() == 0>
                <section class="lives-section" aria-label=move || t(locale.get(), "云上家", "Cloud home")>
                    <Suspense fallback=move || view! { <p class="lives-muted">{move || t(locale.get(), "叩门中…", "Knocking…")}</p> }>
                        {move || Suspend::new(async move {
                            match home.await {
                                Some(h) => {
                                    let hid = h.id.to_string();
                                    let hname = h.name.clone();
                                    let hmotto = h.motto.clone().unwrap_or_else(|| t(locale.get(), "开门，是归处", "A door that leads home").to_string());
                                    view! {
                                        <a class="lives-home-card" href=format!("/inspace/homes/{}", hid)>
                                            <span class="lives-home-name">{hname}</span>
                                            <span class="lives-home-motto">{hmotto}</span>
                                            <span class="lives-home-hint">{move || t(locale.get(), "进云上家看看 →", "Visit the cloud home →")}</span>
                                        </a>
                                    }.into_any()
                                }
                                None => view! { <p class="lives-muted">{move || t(locale.get(), "请先登录", "Sign in first")}</p> }.into_any(),
                            }
                        })}
                    </Suspense>
                </section>

                <section class="lives-section" aria-label=move || t(locale.get(), "在侧的家人", "Companions beside you")>
                    <div class="lives-section-head">
                        <h2>{move || t(locale.get(), "在侧的家人", "Companions beside you")}</h2>
                    </div>

                    <form class="lives-adopt" on:submit=move |ev| { ev.prevent_default(); do_create(); }>
                        <input aria-label=move || t(locale.get(), "名字", "Name") placeholder=move || t(locale.get(), "名字", "Name")
                            prop:value=move || c_name.get() on:input=move |ev| c_name.set(event_target_value(&ev)) required=true />
                        <input aria-label=move || t(locale.get(), "物种", "Species") placeholder=move || t(locale.get(), "物种，如 猫 / 犬", "Species, e.g. cat / dog")
                            prop:value=move || c_species.get() on:input=move |ev| c_species.set(event_target_value(&ev)) />
                        <input aria-label=move || t(locale.get(), "生辰", "Born") type="date"
                            prop:value=move || c_birth.get() on:input=move |ev| c_birth.set(event_target_value(&ev)) />
                        <button type="submit" class="lives-button-primary" disabled=move || c_creating.get()>
                            {move || if c_creating.get() { t(locale.get(), "安顿中…", "Settling…") } else { t(locale.get(), "收留", "Take in") }}
                        </button>
                        <Show when=move || !c_error.get().is_empty()>
                            <p class="lives-error">{move || c_error.get()}</p>
                        </Show>
                    </form>

                    <Suspense fallback=move || view! { <p class="lives-muted">{move || t(locale.get(), "归置中…", "Settling…")}</p> }>
                        {move || Suspend::new(async move {
                            let list = companions.await;
                            if list.is_empty() {
                                view! {
                                    <p class="lives-empty">{move || t(locale.get(), "檐下还空着。收留一位，门庭便有了声息。", "The eaves are empty yet. Take one in.")}</p>
                                }.into_any()
                            } else {
                                view! {
                                    <ul class="lives-companion-list">
                                        {list.iter().map(move |c| {
                                            let id = c.id.to_string();
                                            let name = c.name.clone();
                                            let species = c.species.clone().unwrap_or_default();
                                            let state = c.state;
                                            let birth = c.birth_at.map(|d| d.to_string()).unwrap_or_default();
                                            let trail = c.trail_count;
                                            let zh = matches!(locale.get(), Locale::Zh);
                                            let state_text = state.label(zh).to_string();
                                            let has_birth = !birth.is_empty();
                                            let is_memorial = state == CompanionState::Memorial;
                                            view! {
                                                <li class="lives-companion-card">
                                                    <div class="lives-companion-avatar" aria-hidden="true">{name.chars().next().unwrap_or('·')}</div>
                                                    <div class="lives-companion-body">
                                                        <span class="lives-companion-name">{name.clone()}</span>
                                                        <span class="lives-companion-meta">
                                                            {species.clone()}
                                                            <Show when=move || has_birth> {format!(" · {}", birth)} </Show>
                                                        </span>
                                                        <span class="lives-companion-state {state.as_db()}">{state_text.clone()}</span>
                                                        <span class="lives-companion-trail">
                                                            {move || t(locale.get(), "足迹", "Footprints")}: {trail}
                                                        </span>
                                                    </div>
                                                    <button type="button" class=move || if is_memorial { "lives-distill-button is-hidden" } else { "lives-distill-button" } on:click=move |_| { open_distill(id.clone()); }>
                                                        {move || t(locale.get(), "追远", "Remember")}
                                                    </button>
                                                </li>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </ul>
                                }.into_any()
                            }
                        })}
                    </Suspense>
                </section>

                <Show when=move || distill_open.get()>
                    <section class="lives-distill" aria-label=move || t(locale.get(), "为它蒸馏一段小传", "Distill its life")>
                        <h2>{move || t(locale.get(), "为它蒸馏一段小传", "Distill its life")}</h2>
                        <p class="lives-distill-note">{move || t(locale.get(), "白描、留白、说清生卒与归处即可。", "Plain words; clear span; a resting place.")}</p>
                        <div class="lives-distill-grid">
                            <label class="field-label">
                                <span>{move || t(locale.get(), "离世之日", "Passing date")}</span>
                                <input type="date" prop:value=move || d_death.get() on:input=move |ev| d_death.set(event_target_value(&ev)) required=true />
                            </label>
                            <label class="field-label">
                                <span>{move || t(locale.get(), "碑额", "Epitaph title")}</span>
                                <input placeholder=move || t(locale.get(), "如：阿青之碑", "e.g. Epitaph") prop:value=move || d_epitaph.get() on:input=move |ev| d_epitaph.set(event_target_value(&ev)) />
                            </label>
                        </div>
                        <label class="field-label">
                            <span>{move || t(locale.get(), "铭文", "Inscription")}</span>
                            <input placeholder=move || t(locale.get(), "檐下生，庭前葬。看雪看月，不问归期。", "Born under the eaves, buried by the tree.") prop:value=move || d_inscription.get() on:input=move |ev| d_inscription.set(event_target_value(&ev)) />
                        </label>
                        <label class="field-label">
                            <span>{move || t(locale.get(), "小传（空行分段）", "Life chapters (blank-line separated)")}</span>
                            <textarea rows="5" prop:value=move || d_biography.get() on:input=move |ev| d_biography.set(event_target_value(&ev))></textarea>
                        </label>
                        <label class="field-label">
                            <span>{move || t(locale.get(), "生命地图（每行：地点｜时节｜一事）", "Life map (per line: place｜season｜deed)")}</span>
                            <textarea rows="4" placeholder="窗台｜冬｜看雪，一卧半日&#10;庭前桂树｜秋｜葬于此，花开如见" prop:value=move || d_lifemap.get() on:input=move |ev| d_lifemap.set(event_target_value(&ev))></textarea>
                        </label>
                        <div class="lives-distill-actions">
                            <button type="button" class="lives-button-primary" disabled=move || d_saving.get() on:click=move |_| { do_distill(); }>
                                {move || t(locale.get(), "落成", "Seal it")}
                            </button>
                            <button type="button" class="lives-button-quiet" on:click=move |_| distill_open.set(false)>
                                {move || t(locale.get(), "再想想", "Later")}
                            </button>
                        </div>
                        <Show when=move || !d_error.get().is_empty()>
                            <p class="lives-error">{move || d_error.get()}</p>
                        </Show>
                    </section>
                </Show>
            </Show>

            <Show when=move || tab.get() == 1>
                <section class="lives-section" aria-label=move || t(locale.get(), "追远", "In memory")>
                    <Suspense fallback=move || view! { <p class="lives-muted">{move || t(locale.get(), "翻卷中…", "Turning the pages…")}</p> }>
                        {move || Suspend::new(async move {
                            let Some(data) = lives.await else {
                                return view! { <p class="lives-muted">{move || t(locale.get(), "暂无记录", "Nothing yet")}</p> }.into_any();
                            };
                            if data.items.is_empty() {
                                view! {
                                    <p class="lives-empty">{move || t(locale.get(), "追远台前尚空。每一段相伴，都值得一处归处。", "The memorial hall is quiet yet. Every bond deserves a home.")}</p>
                                }.into_any()
                            } else {
                                view! {
                                    <ul class="lives-memorial-grid">
                                        {data.items.iter().map(move |life| {
                                            let id = life.id.to_string();
                                            let name = life.name.clone();
                                            let epitaph = life.epitaph.clone();
                                            let incense = life.incense_count;
                                            let date = life.memorial_date.map(|d| d.to_string()).unwrap_or_default();
                                            let has_epitaph = !epitaph.is_empty();
                                            let has_incense = incense > 0;
                                            view! {
                                                <li>
                                                    <a class="lives-memorial-card" href=format!("/inspace/lives/{}", id)>
                                                        <span class="lives-memorial-name">{name.clone()}</span>
                                                        <Show when=move || has_epitaph>
                                                            <span class="lives-memorial-epitaph">{epitaph.clone()}</span>
                                                        </Show>
                                                        <span class="lives-memorial-meta">
                                                            {date}
                                                            <Show when=move || has_incense> {format!(" · {} {}", incense, t(locale.get(), "炷香", "incense"))} </Show>
                                                        </span>
                                                    </a>
                                                </li>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </ul>
                                }.into_any()
                            }
                        })}
                    </Suspense>
                    <Show when=move || show_pager.get()>
                        <nav class="lives-pager" aria-label=move || t(locale.get(), "翻页", "Pagination")>
                            <button type="button" disabled=move || !can_prev.get() on:click=move |_| page.update(|p| *p = (*p - 1).max(1))>
                                {move || t(locale.get(), "上一页", "Previous")}
                            </button>
                            <span>{move || format!("{} / {}", page.get(), total_pages.get())}</span>
                            <button type="button" disabled=move || !can_next.get() on:click=move |_| page.update(|p| *p += 1)>
                                {move || t(locale.get(), "下一页", "Next")}
                            </button>
                        </nav>
                    </Show>
                </section>
            </Show>
        </main>
    }
}

// ---------------------------------------------------------------------------
// 云上家：叩门而入
// ---------------------------------------------------------------------------

#[component]
pub fn CloudHomePage() -> impl IntoView {
    let locale = use_i18n().locale;
    let params = use_params_map();
    let home_id = Memo::new(move |_| params.with(|p| p.get("home_id").unwrap_or_default()));
    let reload = RwSignal::new(0u32);
    let knock_attempt = RwSignal::new(0u32);
    let passphrase = RwSignal::new(String::new());
    let knock_error = RwSignal::new(String::new());

    let visit = Resource::new(
        move || (reload.get(), home_id.get(), knock_attempt.get()),
        move |(_, id, attempt)| async move {
            if attempt == 0 {
                visit_cloud_home(id, None).await.ok()
            } else {
                visit_cloud_home(id, Some(passphrase.get())).await.ok()
            }
        },
    );

    // 主人布置
    let h_name = RwSignal::new(String::new());
    let h_motto = RwSignal::new(String::new());
    let h_door = RwSignal::new(String::new());
    let h_pass = RwSignal::new(String::new());
    let h_error = RwSignal::new(String::new());

    let do_knock = move || {
        knock_error.set(String::new());
        knock_attempt.update(|v| *v += 1);
    };

    let save_home = move |clear: bool| {
        leptos::task::spawn_local(async move {
            h_error.set(String::new());
            let pass = h_pass.get();
            let want_key = !clear && !pass.trim().is_empty();
            match update_my_cloud_home(
                h_name.get(),
                Some(h_motto.get()),
                Some(h_door.get()),
                want_key.then(|| pass),
                clear,
            )
            .await
            {
                Ok(_) => {
                    h_pass.set(String::new());
                    reload.update(|v| *v += 1);
                }
                Err(e) => h_error.set(e.to_string()),
            }
        });
    };

    view! {
        <Title text=move || t(locale.get(), "云上家｜万物有灵，皆有归处", "Cloud home | Every soul has a home") />
        <main id="main-content" class="page cloud-home-page">
            <Suspense fallback=move || view! { <p class="lives-muted">{move || t(locale.get(), "叩门中…", "Knocking…")}</p> }>
                {move || Suspend::new(async move {
                    let Some(v) = visit.await else {
                        return view! { <p class="lives-empty">{move || t(locale.get(), "没有这扇门。", "No such door.")}</p> }.into_any();
                    };
                    let hname = v.home.name.clone();
                    let home_space_id = v.home.space_id.to_string();
                    let home_world_href = format!("/inspace/world/{home_space_id}?via=home&spawn=gate");
                    let home_space_label = home_space_id.clone();
                    let hmotto = v.home.motto.clone().unwrap_or_else(|| t(locale.get(), "开门见山，进门是家。", "Beyond the door, home.").to_string());
                    let hdoor = v.home.door_note.clone().unwrap_or_default();
                    let has_key = v.home.has_passphrase;
                    let entered = v.entered;
                    let show_inside = entered || !has_key;
                    let locked = has_key && !entered;
                    let companions_empty = v.companions.is_empty();
                    let hdoor_empty = hdoor.is_empty();
                    view! {
                        <div class="cloud-home-door">
                            <p class="lives-kicker">{move || t(locale.get(), "云上家", "Cloud home")}</p>
                            <h1 class="cloud-home-plaque" aria-label=hname.clone()>
                                {hname.chars().map(|c| view! { <span class="cloud-home-plaque-char">{c}</span> }).collect::<Vec<_>>()}
                                <span class="cloud-home-seal">家</span>
                            </h1>
                            <p class="cloud-home-motto">{hmotto.clone()}</p>
                            <p class="cloud-home-space-identity">
                                <span>{move || t(locale.get(), "家空间", "Home Space")}</span>
                                <code>{format!("SpaceID {}", home_space_label)}</code>
                            </p>
                            <Show when=move || show_inside>
                                <a class="cloud-home-world-entry" href=home_world_href.clone()>
                                    <span>{move || t(locale.get(), "走入庭院", "Enter the courtyard")}</span>
                                    <span aria-hidden="true">"→"</span>
                                </a>
                            </Show>
                            <Show when=move || !hdoor_empty>
                                <p class="cloud-home-door-note">{hdoor.clone()}</p>
                            </Show>
                            <Show when=move || locked>
                                <form class="cloud-home-knock" on:submit=move |ev| { ev.prevent_default(); do_knock(); }>
                                    <input type="password" autocomplete="current-password"
                                        aria-label=move || t(locale.get(), "进门口令", "Door key")
                                        placeholder=move || t(locale.get(), "进门口令", "Door key")
                                        prop:value=move || passphrase.get()
                                        on:input=move |ev| passphrase.set(event_target_value(&ev)) />
                                    <button type="submit" class="lives-button-primary">{move || t(locale.get(), "叩门", "Knock")}</button>
                                    <Show when=move || !knock_error.get().is_empty()>
                                        <p class="lives-error">{move || knock_error.get()}</p>
                                    </Show>
                                </form>
                            </Show>
                            <Show when=move || show_inside>
                                <p class="cloud-home-open">{move || t(locale.get(), "门已开。", "The door is open.")}</p>
                            </Show>
                        </div>

                        <Show when=move || show_inside>
                            <section id="companions-at-home" class="lives-section" aria-label=move || t(locale.get(), "在家的家人", "Companions at home")>
                                <h2>{move || t(locale.get(), "在家的家人", "Companions at home")}</h2>
                                <Show when=move || companions_empty>
                                    <p class="lives-empty">{move || t(locale.get(), "此间尚静，未见家人。", "Quiet inside, for now.")}</p>
                                </Show>
                                <ul class="lives-companion-list">
                                    {v.companions.iter().map(move |c| {
                                        let name = c.name.clone();
                                        let species = c.species.clone().unwrap_or_default();
                                        let state = c.state;
                                        let zh = matches!(locale.get(), Locale::Zh);
                                        let state_text = state.label(zh).to_string();
                                        view! {
                                            <li class="lives-companion-card">
                                                <div class="lives-companion-avatar" aria-hidden="true">{name.chars().next().unwrap_or('·')}</div>
                                                <div class="lives-companion-body">
                                                    <span class="lives-companion-name">{name.clone()}</span>
                                                    <span class="lives-companion-meta">{species.clone()}</span>
                                                    <span class="lives-companion-state {state.as_db()}">{state_text.clone()}</span>
                                                </div>
                                            </li>
                                        }
                                    }).collect::<Vec<_>>()}
                                </ul>
                            </section>
                        </Show>
                    }.into_any()
                })}
            </Suspense>

            <section id="arrange-home" class="lives-section cloud-home-settings" aria-label=move || t(locale.get(), "布置云上家", "Arrange the home")>
                <h2>{move || t(locale.get(), "布置云上家", "Arrange the home")}</h2>
                <Suspense fallback=move || view! { <p class="lives-muted">{ "…" }</p> }>
                    {move || Suspend::new(async move {
                        let mine = get_my_cloud_home().await.ok();
                        let viewer = current_session().await.ok().flatten();
                        let is_owner = match (&mine, viewer) {
                            (Some(h), Some(u)) => h.owner_id == u.id,
                            _ => false,
                        };
                        if !is_owner {
                            return view! { <p class="lives-muted">{move || t(locale.get(), "此非你家，不可布置。", "Not your door to arrange.")}</p> }.into_any();
                        }
                        let current = mine.unwrap();
                        let placeholder_name = current.name.clone();
                        let placeholder_motto = current.motto.clone().unwrap_or_default();
                        let placeholder_door = current.door_note.clone().unwrap_or_default();
                        view! {
                            <div class="lives-distill-grid">
                                <label class="field-label">
                                    <span>{move || t(locale.get(), "家名", "Home name")}</span>
                                    <input prop:value=move || h_name.get() on:input=move |ev| h_name.set(event_target_value(&ev)) placeholder=placeholder_name />
                                </label>
                                <label class="field-label">
                                    <span>{move || t(locale.get(), "门联", "Motto")}</span>
                                    <input prop:value=move || h_motto.get() on:input=move |ev| h_motto.set(event_target_value(&ev)) placeholder=placeholder_motto />
                                </label>
                                <label class="field-label">
                                    <span>{move || t(locale.get(), "门牌小字", "Door note")}</span>
                                    <input prop:value=move || h_door.get() on:input=move |ev| h_door.set(event_target_value(&ev)) placeholder=placeholder_door />
                                </label>
                                <label class="field-label">
                                    <span>{move || t(locale.get(), "进门口令（留空则不换）", "Door key (blank keeps current)")}</span>
                                    <input type="password" prop:value=move || h_pass.get() on:input=move |ev| h_pass.set(event_target_value(&ev)) />
                                </label>
                            </div>
                            <div class="lives-distill-actions">
                                <button type="button" class="lives-button-primary" on:click=move |_| { save_home(false); }>
                                    {move || t(locale.get(), "落锁 / 更名", "Lock / rename")}
                                </button>
                                <button type="button" class="lives-button-quiet" on:click=move |_| { save_home(true); }>
                                    {move || t(locale.get(), "卸下口令", "Remove the key")}
                                </button>
                            </div>
                            <Show when=move || !h_error.get().is_empty()>
                                <p class="lives-error">{move || h_error.get()}</p>
                            </Show>
                        }.into_any()
                    })}
                </Suspense>
            </section>
        </main>
    }
}

// 云上家的可行走场景已统一迁移到 `pages/world.rs` 的 Phaser Runtime。
// 此页只保留门禁、家庭成员记录和主人布置，避免维护两套互相冲突的交互。

#[component]
pub fn MemorialPage() -> impl IntoView {
    let locale = use_i18n().locale;
    let params = use_params_map();
    let life_id = Memo::new(move |_| params.with(|p| p.get("life_id").unwrap_or_default()));
    let reload = RwSignal::new(0u32);

    let life = Resource::new(
        move || (reload.get(), life_id.get()),
        |(_, id)| async move { get_digital_life(id).await.ok() },
    );
    let prayers = Resource::new(
        move || (reload.get(), life_id.get()),
        |(_, id)| async move { list_prayers(id, Some(50)).await.ok().unwrap_or_default() },
    );

    let prayer_kind = RwSignal::new(String::from("incense"));
    let prayer_message = RwSignal::new(String::new());
    let prayer_error = RwSignal::new(String::new());

    let offer = move |id: String| {
        leptos::task::spawn_local(async move {
            prayer_error.set(String::new());
            let message = prayer_message.get();
            match leave_prayer(
                id,
                prayer_kind.get(),
                (!message.trim().is_empty()).then(|| message),
            )
            .await
            {
                Ok(_) => {
                    prayer_message.set(String::new());
                    reload.update(|v| *v += 1);
                }
                Err(e) => prayer_error.set(e.to_string()),
            }
        });
    };

    view! {
        <Title text=move || t(locale.get(), "追远｜慎终追远", "In memory | Remembered") />
        <main id="main-content" class="page memorial-page">
            <Suspense fallback=move || view! { <p class="lives-muted">{move || t(locale.get(), "翻开卷册…", "Turning pages…")}</p> }>
                {move || Suspend::new(async move {
                    let Some(l) = life.await else {
                        return view! { <p class="lives-empty">{move || t(locale.get(), "没有这卷小传。", "No such life.")}</p> }.into_any();
                    };
                    let life_id_value = l.id.to_string();
                    let name = l.name.clone();
                    let epitaph = l.epitaph.clone();
                    let inscription = l.inscription.clone();
                    let biography = l.biography.clone();
                    let life_map = l.life_map.clone();
                    let memorial = l.memorial_date.map(|d| d.to_string()).unwrap_or_default();
                    let incense = l.incense_count;
                    let visitors = l.visitor_count;
                    let has_epitaph = !epitaph.is_empty();
                    let has_meta = !memorial.is_empty() || incense > 0 || visitors > 0;
                    let has_biography = !biography.is_empty();
                    let has_inscription = !inscription.is_empty();
                    let has_map = !life_map.is_empty();
                    let has_incense = incense > 0;
                    let has_visitors = visitors > 0;
                    view! {
                        <article class="memorial-sheet">
                            <p class="lives-kicker">{move || t(locale.get(), "追远", "In memory")}</p>
                            <Show when=move || has_epitaph>
                                <h1 class="memorial-epitaph">{epitaph.clone()}</h1>
                            </Show>
                            <h2 class="memorial-name">{name.clone()}</h2>
                            <Show when=move || has_meta>
                                <p class="memorial-meta">
                                    {memorial.clone()}
                                    <Show when=move || has_incense> {format!(" · {} {}", incense, t(locale.get(), "炷香", "incense"))} </Show>
                                    <Show when=move || has_visitors> {format!(" · {} {}", visitors, t(locale.get(), "次回望", "visits"))} </Show>
                                </p>
                            </Show>

                            <Show when=move || has_biography>
                                <section class="memorial-chapter" aria-label=move || t(locale.get(), "小传", "Life")>
                                    <h3>{move || t(locale.get(), "小传", "Life")}</h3>
                                    {biography.iter().map(move |chapter| {
                                        let body = chapter.body.clone();
                                        view! { <p class="memorial-prose">{body}</p> }
                                    }).collect::<Vec<_>>()}
                                </section>
                            </Show>

                            <Show when=move || has_inscription>
                                <section class="memorial-inscription" aria-label=move || t(locale.get(), "铭文", "Inscription")>
                                    <h3>{move || t(locale.get(), "铭文", "Inscription")}</h3>
                                    <p class="memorial-inscription-text">{inscription.clone()}</p>
                                </section>
                            </Show>

                            <Show when=move || has_map>
                                <section class="memorial-map" aria-label=move || t(locale.get(), "生命地图", "Life map")>
                                    <h3>{move || t(locale.get(), "生命地图", "Life map")}</h3>
                                    <ul class="memorial-map-list">
                                        {life_map.iter().map(move |entry| {
                                            let place = entry.place.clone();
                                            let season = entry.season.clone().unwrap_or_default();
                                            let deed = entry.deed.clone();
                                            let has_season = !season.is_empty();
                                            view! {
                                                <li>
                                                    <b>{place}</b>
                                                    <Show when=move || has_season> <i>{season.clone()}</i> </Show>
                                                    <span>{deed.clone()}</span>
                                                </li>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </ul>
                                </section>
                            </Show>

                            <section class="memorial-prayers" aria-label=move || t(locale.get(), "祈福", "Offerings")>
                                <h3>{move || t(locale.get(), "祈福", "Offerings")}</h3>
                                <div class="memorial-prayer-kinds" role="group" aria-label=move || t(locale.get(), "一炷香 / 一枝花 / 一盏灯 / 留字", "Incense / flower / lantern / words")>
                                    <button type="button" class=move || if prayer_kind.get() == "incense" { "is-active" } else { "" } on:click=move |_| prayer_kind.set("incense".into())>{move || t(locale.get(), "一炷香", "Incense")}</button>
                                    <button type="button" class=move || if prayer_kind.get() == "flower" { "is-active" } else { "" } on:click=move |_| prayer_kind.set("flower".into())>{move || t(locale.get(), "一枝花", "Flower")}</button>
                                    <button type="button" class=move || if prayer_kind.get() == "lantern" { "is-active" } else { "" } on:click=move |_| prayer_kind.set("lantern".into())>{move || t(locale.get(), "一盏灯", "Lantern")}</button>
                                    <button type="button" class=move || if prayer_kind.get() == "word" { "is-active" } else { "" } on:click=move |_| prayer_kind.set("word".into())>{move || t(locale.get(), "留字", "Words")}</button>
                                </div>
                                <div class="memorial-prayer-form">
                                    <textarea rows="2" placeholder=move || t(locale.get(), "留下想说的话", "Leave a few words") prop:value=move || prayer_message.get() on:input=move |ev| prayer_message.set(event_target_value(&ev))></textarea>
                                    <button type="button" class="lives-button-primary" on:click=move |_| { offer(life_id_value.clone()); }>
                                        {move || t(locale.get(), "奉上", "Offer")}
                                    </button>
                                </div>
                                <Show when=move || !prayer_error.get().is_empty()>
                                    <p class="lives-error">{move || prayer_error.get()}</p>
                                </Show>
                            </section>

                            <section class="memorial-ledger" aria-label=move || t(locale.get(), "来客留名", "Visitors")>
                                <h3>{move || t(locale.get(), "来客留名", "Visitors")}</h3>
                                <Suspense fallback=move || view! { <p class="lives-muted">{ "…" }</p> }>
                                    {move || Suspend::new(async move {
                                        let list = prayers.await;
                                        if list.is_empty() {
                                            view! { <p class="lives-muted">{move || t(locale.get(), "此间尚静。", "Quiet, for now.")}</p> }.into_any()
                                        } else {
                                            view! {
                                                <ul class="memorial-ledger-list">
                                                    {list.iter().map(move |p| {
                                                        let name = p.visitor_name.clone();
                                                        let kind = prayer_kind_label(&p, locale.get());
                                                        let message = p.message.clone().unwrap_or_default();
                                                        let date = p.created_at.to_string();
                                                        let has_message = !message.is_empty();
                                                        view! {
                                                            <li>
                                                                <span class="memorial-ledger-kind">{kind}</span>
                                                                <b>{name}</b>
                                                                <Show when=move || has_message> <span class="memorial-ledger-msg">{message.clone()}</span> </Show>
                                                                <small>{date.clone()}</small>
                                                            </li>
                                                        }
                                                    }).collect::<Vec<_>>()}
                                                </ul>
                                            }.into_any()
                                        }
                                    })}
                                </Suspense>
                            </section>
                        </article>
                    }.into_any()
                })}
            </Suspense>
        </main>
    }
}

fn prayer_kind_label(p: &LifePrayer, locale: Locale) -> &'static str {
    match (p.kind, locale) {
        (instant_domain::lives::PrayerKind::Incense, Locale::Zh) => "一炷香",
        (instant_domain::lives::PrayerKind::Flower, Locale::Zh) => "一枝花",
        (instant_domain::lives::PrayerKind::Lantern, Locale::Zh) => "一盏灯",
        (instant_domain::lives::PrayerKind::Word, Locale::Zh) => "留字",
        (instant_domain::lives::PrayerKind::Incense, _) => "Incense",
        (instant_domain::lives::PrayerKind::Flower, _) => "Flower",
        (instant_domain::lives::PrayerKind::Lantern, _) => "Lantern",
        (instant_domain::lives::PrayerKind::Word, _) => "Words",
    }
}
