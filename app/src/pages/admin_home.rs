use instant_domain::site::{HomePageAdminState, HomePageConfig, LocalizedText};
use leptos::prelude::*;

use crate::{
    components::admin_nav::AdminNav,
    i18n::{t, use_i18n, Locale},
    server::{
        auth::current_session,
        site::{
            get_admin_home_config, list_home_versions, publish_home_config, restore_home_version,
            save_home_draft,
        },
    },
};

#[component]
pub fn AdminHomePage() -> impl IntoView {
    let locale = use_i18n().locale;
    let session = Resource::new(
        || (),
        |_| async move { current_session().await.ok().flatten() },
    );
    let state = Resource::new(
        || (),
        |_| async move { get_admin_home_config().await.unwrap_or_default() },
    );

    view! {
        <main id="main-content" class="page admin-layout admin-site-editor-page">
            <Suspense fallback=move || view! { <p>{move || t(locale.get(), "正在检查管理员权限", "Checking admin access")}</p> }>
                {move || Suspend::new(async move {
                    let user = session.await;
                    if !user.as_ref().is_some_and(|u| u.role.is_admin()) {
                        return view! {
                            <section class="form"><h1>{move || t(locale.get(), "需要管理员登录", "Admin sign-in required")}</h1><a class="button button-primary" href="/inspace/login">{move || t(locale.get(), "去登录", "Go to sign in")}</a></section>
                        }.into_any();
                    }
                    let initial = state.await;
                    view! { <AdminNav /><HomeEditor initial=initial /> }.into_any()
                })}
            </Suspense>
        </main>
    }
}

#[component]
fn HomeEditor(initial: HomePageAdminState) -> impl IntoView {
    let locale = use_i18n().locale;
    let config = RwSignal::new(initial.draft);
    let published_version = RwSignal::new(initial.published_version);
    let version_reload = RwSignal::new(0u32);
    let notice = RwSignal::new(None::<String>);
    let error = RwSignal::new(None::<String>);

    let versions = Resource::new(
        move || version_reload.get(),
        |_| async move { list_home_versions().await.unwrap_or_default() },
    );
    let save = Action::new(move |value: &HomePageConfig| {
        let value = value.clone();
        async move { save_home_draft(value).await }
    });
    let publish = Action::new(move |value: &HomePageConfig| {
        let value = value.clone();
        async move { publish_home_config(value).await }
    });
    let restore = Action::new(move |version_id: &String| {
        let version_id = version_id.clone();
        async move { restore_home_version(version_id).await }
    });

    Effect::new(move |_| {
        if let Some(result) = save.value().get() {
            match result {
                Ok(state) => {
                    config.set(state.draft);
                    published_version.set(state.published_version);
                    error.set(None);
                    notice.set(Some(
                        t(locale.get(), "草稿已保存", "Draft saved").to_string(),
                    ));
                }
                Err(err) => {
                    notice.set(None);
                    error.set(Some(err.to_string()));
                }
            }
        }
    });
    Effect::new(move |_| {
        if let Some(result) = publish.value().get() {
            match result {
                Ok(state) => {
                    config.set(state.draft);
                    published_version.set(state.published_version);
                    version_reload.update(|value| *value += 1);
                    error.set(None);
                    notice.set(Some(format!(
                        "{} v{}",
                        t(locale.get(), "首页已发布", "Homepage published"),
                        state.published_version
                    )));
                }
                Err(err) => {
                    notice.set(None);
                    error.set(Some(err.to_string()));
                }
            }
        }
    });
    Effect::new(move |_| {
        if let Some(result) = restore.value().get() {
            match result {
                Ok(state) => {
                    config.set(state.draft);
                    error.set(None);
                    notice.set(Some(
                        t(
                            locale.get(),
                            "历史版本已恢复为草稿，请确认后发布",
                            "Version restored to draft; review before publishing",
                        )
                        .to_string(),
                    ));
                }
                Err(err) => {
                    notice.set(None);
                    error.set(Some(err.to_string()));
                }
            }
        }
    });

    view! {
        <section class="admin-site-editor">
            <header class="admin-site-editor-head">
                <div>
                    <p class="eyebrow">"SITE SETTINGS"</p>
                    <h1>{move || t(locale.get(), "首页编辑器", "Homepage editor")}</h1>
                    <p>{move || t(locale.get(), "整页模块、双语文案、按钮、导航、SEO 与受控主题都可以在这里修改。先保存草稿，再预览和发布。", "Edit modules, bilingual copy, buttons, navigation, SEO, and controlled theme tokens. Save a draft before previewing and publishing.")}</p>
                </div>
                <div class="admin-site-version"><span>{move || t(locale.get(), "线上版本", "Published version")}</span><strong>{move || format!("v{}", published_version.get())}</strong></div>
            </header>

            <div class="admin-site-grid">
                <form class="admin-site-form" on:submit=move |ev| ev.prevent_default()>
                    <EditorSection title_zh="主题与版式" title_en="Theme and layout">
                        <div class="admin-theme-presets" role="group" aria-label="Theme preset">
                            <ThemeButton config=config value="sky-ocean" label_zh="天空与深海" label_en="Sky & ocean" />
                            <ThemeButton config=config value="sky" label_zh="纯净天空" label_en="Clear sky" />
                            <ThemeButton config=config value="ocean" label_zh="深海沉浸" label_en="Deep ocean" />
                        </div>
                        <div class="admin-field-grid three">
                            <label>{move || t(locale.get(), "主色", "Primary")}<input type="color" prop:value=move || config.get().theme.primary on:input=move |ev| config.update(|value| value.theme.primary = event_target_value(&ev)) /></label>
                            <label>{move || t(locale.get(), "深海色", "Deep")}<input type="color" prop:value=move || config.get().theme.deep on:input=move |ev| config.update(|value| value.theme.deep = event_target_value(&ev)) /></label>
                            <label>{move || t(locale.get(), "背景色", "Background")}<input type="color" prop:value=move || config.get().theme.background on:input=move |ev| config.update(|value| value.theme.background = event_target_value(&ev)) /></label>
                        </div>
                        <div class="admin-field-grid two">
                            <label>{move || t(locale.get(), "首屏版式", "Hero layout")}<select prop:value=move || config.get().theme.hero_layout on:change=move |ev| config.update(|value| value.theme.hero_layout = event_target_value(&ev))><option value="split">{move || t(locale.get(), "图文双栏", "Split")}</option><option value="centered">{move || t(locale.get(), "文字居中", "Centered")}</option></select></label>
                            <label>{move || t(locale.get(), "信息密度", "Density")}<select prop:value=move || config.get().theme.density on:change=move |ev| config.update(|value| value.theme.density = event_target_value(&ev))><option value="comfortable">{move || t(locale.get(), "舒适", "Comfortable")}</option><option value="compact">{move || t(locale.get(), "紧凑", "Compact")}</option></select></label>
                        </div>
                    </EditorSection>

                    <EditorSection title_zh="首屏主张" title_en="Hero">
                        <ModuleControls config=config module="hero" />
                        <LocalizedFields config=config field="hero_eyebrow" label_zh="眉题" label_en="Eyebrow" multiline=false />
                        <LocalizedFields config=config field="hero_title" label_zh="主标题" label_en="Headline" multiline=false />
                        <LocalizedFields config=config field="hero_body" label_zh="说明文案" label_en="Body" multiline=true />
                        <LocalizedFields config=config field="hero_note" label_zh="首屏补充说明" label_en="Hero note" multiline=false />
                        <LocalizedFields config=config field="hero_primary_label" label_zh="主按钮文字" label_en="Primary button" multiline=false />
                        <div class="admin-field-grid two"><label>{move || t(locale.get(), "主按钮链接", "Primary URL")}<input prop:value=move || config.get().hero.primary_url on:input=move |ev| config.update(|value| value.hero.primary_url = event_target_value(&ev)) /></label><label>{move || t(locale.get(), "次按钮链接", "Secondary URL")}<input prop:value=move || config.get().hero.secondary_url on:input=move |ev| config.update(|value| value.hero.secondary_url = event_target_value(&ev)) /></label></div>
                        <LocalizedFields config=config field="hero_secondary_label" label_zh="次按钮文字" label_en="Secondary button" multiline=false />
                        <LocalizedFields config=config field="hero_sample_location" label_zh="示例地点" label_en="Sample location" multiline=false />
                        <LocalizedFields config=config field="hero_sample_title" label_zh="示例标题" label_en="Sample title" multiline=false />
                        <LocalizedFields config=config field="hero_sample_body" label_zh="示例说明" label_en="Sample body" multiline=false />
                        <LocalizedFields config=config field="hero_sample_guide_label" label_zh="示例攻略标签" label_en="Sample guide label" multiline=false />
                        <LocalizedFields config=config field="hero_sample_question" label_zh="示例问题" label_en="Sample question" multiline=false />
                        <LocalizedFields config=config field="hero_sample_presence" label_zh="示例在线状态" label_en="Sample presence" multiline=false />
                    </EditorSection>

                    <EditorSection title_zh="到达—攻略—互助" title_en="Journey">
                        <ModuleControls config=config module="journey" />
                        <LocalizedFields config=config field="journey_eyebrow" label_zh="眉题" label_en="Eyebrow" multiline=false />
                        <LocalizedFields config=config field="journey_title" label_zh="模块标题" label_en="Section title" multiline=false />
                        <LocalizedFields config=config field="journey_body" label_zh="模块说明" label_en="Section body" multiline=true />
                        <LocalizedFields config=config field="arrive_title" label_zh="到达标题" label_en="Arrival title" multiline=false />
                        <LocalizedFields config=config field="arrive_body" label_zh="到达说明" label_en="Arrival copy" multiline=false />
                        <LocalizedFields config=config field="guide_step_title" label_zh="攻略标题" label_en="Guide title" multiline=false />
                        <LocalizedFields config=config field="guide_step_body" label_zh="攻略说明" label_en="Guide copy" multiline=false />
                        <LocalizedFields config=config field="help_title" label_zh="互助标题" label_en="Help title" multiline=false />
                        <LocalizedFields config=config field="help_body" label_zh="互助说明" label_en="Help copy" multiline=false />
                    </EditorSection>

                    <EditorSection title_zh="空间攻略价值" title_en="Guide value">
                        <ModuleControls config=config module="guide" />
                        <LocalizedFields config=config field="guide_eyebrow" label_zh="眉题" label_en="Eyebrow" multiline=false />
                        <LocalizedFields config=config field="guide_title" label_zh="模块标题" label_en="Section title" multiline=false />
                        <LocalizedFields config=config field="guide_body" label_zh="模块说明" label_en="Section body" multiline=true />
                        <LocalizedFields config=config field="guide_visual_route" label_zh="路线视觉文案" label_en="Route visual copy" multiline=false />
                        <LocalizedFields config=config field="guide_visual_warning" label_zh="避坑视觉文案" label_en="Warning visual copy" multiline=false />
                        <LocalizedFields config=config field="guide_visual_live" label_zh="现场视觉文案" label_en="Live visual copy" multiline=false />
                        <LocalizedFields config=config field="guide_cta_label" label_zh="按钮文字" label_en="Button label" multiline=false />
                        <div class="admin-field-grid two"><label>{move || t(locale.get(), "按钮链接", "Button URL")}<input prop:value=move || config.get().guide.cta_url on:input=move |ev| config.update(|value| value.guide.cta_url = event_target_value(&ev)) /></label></div>
                    </EditorSection>

                    <EditorSection title_zh="主理人入口" title_en="Host call to action">
                        <ModuleControls config=config module="host" />
                        <LocalizedFields config=config field="host_title" label_zh="标题" label_en="Title" multiline=false />
                        <LocalizedFields config=config field="host_body" label_zh="说明" label_en="Body" multiline=true />
                        <LocalizedFields config=config field="host_cta_label" label_zh="按钮文字" label_en="Button label" multiline=false />
                    </EditorSection>

                    <EditorSection title_zh="导航与 SEO" title_en="Navigation and SEO">
                        <LocalizedFields config=config field="nav_home" label_zh="首页导航" label_en="Home label" multiline=false />
                        <LocalizedFields config=config field="nav_map" label_zh="地图导航" label_en="Map label" multiline=false />
                        <LocalizedFields config=config field="nav_guides" label_zh="攻略导航" label_en="Guide label" multiline=false />
                        <LocalizedFields config=config field="nav_my_spaces" label_zh="我的空间导航" label_en="My spaces label" multiline=false />
                        <LocalizedFields config=config field="seo_title" label_zh="SEO 标题" label_en="SEO title" multiline=false />
                        <LocalizedFields config=config field="seo_description" label_zh="SEO 描述" label_en="SEO description" multiline=true />
                    </EditorSection>

                    <div class="admin-site-actions">
                        <a class="button button-secondary" href="/inspace" target="_blank" rel="noreferrer">{move || t(locale.get(), "打开线上首页", "Open published home")}</a>
                        <button class="button button-secondary" type="button" disabled=move || save.pending().get() on:click=move |_| { save.dispatch(config.get()); }>{move || if save.pending().get() { t(locale.get(), "保存中…", "Saving…") } else { t(locale.get(), "保存草稿", "Save draft") }}</button>
                        <button class="button button-primary" type="button" disabled=move || publish.pending().get() on:click=move |_| { publish.dispatch(config.get()); }>{move || if publish.pending().get() { t(locale.get(), "发布中…", "Publishing…") } else { t(locale.get(), "发布首页", "Publish homepage") }}</button>
                    </div>
                    {move || notice.get().map(|value| view! { <p class="form-success" role="status">{value}</p> })}
                    {move || error.get().map(|value| view! { <p class="form-error" role="alert">{value}</p> })}
                </form>

                <aside class="admin-home-preview" style=move || format!("--home-primary:{};--home-deep:{};--home-bg:{}", config.get().theme.primary, config.get().theme.deep, config.get().theme.background)>
                    <div class="admin-home-preview-sticky">
                        <small>{move || t(locale.get(), "草稿实时预览", "Live draft preview")}</small>
                        <h2>{move || localized(locale.get(), &config.get().hero.title)}</h2>
                        <p>{move || localized(locale.get(), &config.get().hero.body)}</p>
                        <span class="admin-preview-button">{move || localized(locale.get(), &config.get().hero.primary_label)}</span>
                        <div class="admin-preview-flow"><span>01 {move || localized(locale.get(), &config.get().journey.arrive_title)}</span><span>02 {move || localized(locale.get(), &config.get().journey.guide_title)}</span><span>03 {move || localized(locale.get(), &config.get().journey.help_title)}</span></div>
                    </div>
                    <section class="admin-version-list">
                        <h3>{move || t(locale.get(), "发布历史", "Version history")}</h3>
                        <Suspense fallback=move || view! { <p>{move || t(locale.get(), "正在加载…", "Loading…")}</p> }>
                            {move || Suspend::new(async move {
                                let items = versions.await;
                                if items.is_empty() {
                                    view! { <p class="muted">{move || t(locale.get(), "还没有发布版本。", "No published versions yet.")}</p> }.into_any()
                                } else {
                                    view! { <div class="admin-version-items"><For each=move || items.clone() key=|item| item.id children=move |item| { let id=item.id.to_string(); view! { <div><span><strong>{format!("v{}", item.version)}</strong><small>{item.created_at}</small></span><button type="button" on:click=move |_| { restore.dispatch(id.clone()); }>{move || t(locale.get(), "恢复为草稿", "Restore draft")}</button></div> } } /></div> }.into_any()
                                }
                            })}
                        </Suspense>
                    </section>
                </aside>
            </div>
        </section>
    }
}

#[component]
fn EditorSection(
    title_zh: &'static str,
    title_en: &'static str,
    children: Children,
) -> impl IntoView {
    let locale = use_i18n().locale;
    view! { <fieldset class="admin-editor-section"><legend>{move || t(locale.get(), title_zh, title_en)}</legend>{children()}</fieldset> }
}

#[component]
fn ThemeButton(
    config: RwSignal<HomePageConfig>,
    value: &'static str,
    label_zh: &'static str,
    label_en: &'static str,
) -> impl IntoView {
    let locale = use_i18n().locale;
    view! { <button type="button" class=move || if config.get().theme.preset == value { "theme-preset is-active" } else { "theme-preset" } aria-pressed=move || if config.get().theme.preset == value { "true" } else { "false" } on:click=move |_| config.update(|config| config.theme.preset = value.to_string())>{move || t(locale.get(), label_zh, label_en)}</button> }
}

#[component]
fn ModuleControls(config: RwSignal<HomePageConfig>, module: &'static str) -> impl IntoView {
    let locale = use_i18n().locale;
    let visible = move || match module {
        "hero" => config.get().hero.visible,
        "journey" => config.get().journey.visible,
        "guide" => config.get().guide.visible,
        _ => config.get().host.visible,
    };
    let order = move || match module {
        "hero" => config.get().hero.order,
        "journey" => config.get().journey.order,
        "guide" => config.get().guide.order,
        _ => config.get().host.order,
    };
    view! { <div class="module-controls"><label class="module-visible"><input type="checkbox" prop:checked=visible on:change=move |ev| { let checked=event_target_checked(&ev); config.update(|value| match module { "hero" => value.hero.visible=checked, "journey" => value.journey.visible=checked, "guide" => value.guide.visible=checked, _ => value.host.visible=checked }); } />{move || t(locale.get(), "显示此模块", "Show module")}</label><label>{move || t(locale.get(), "顺序", "Order")}<input type="number" min="0" max="100" prop:value=move || order().to_string() on:input=move |ev| { if let Ok(next)=event_target_value(&ev).parse::<i32>() { config.update(|value| match module { "hero" => value.hero.order=next, "journey" => value.journey.order=next, "guide" => value.guide.order=next, _ => value.host.order=next }); } } /></label></div> }
}

#[component]
fn LocalizedFields(
    config: RwSignal<HomePageConfig>,
    field: &'static str,
    label_zh: &'static str,
    label_en: &'static str,
    multiline: bool,
) -> impl IntoView {
    let locale = use_i18n().locale;
    let get_value = move || field_value(&config.get(), field);
    let update = move |lang: &'static str, value: String| {
        config.update(|config| set_field_value(config, field, lang, value))
    };
    view! {
        <div class="localized-field">
            <span>{move || t(locale.get(), label_zh, label_en)}</span>
            <div class="admin-field-grid two">
                <label>"中文"{if multiline { view! { <textarea prop:value=move || get_value().zh on:input=move |ev| update("zh", event_target_value(&ev))></textarea> }.into_any() } else { view! { <input prop:value=move || get_value().zh on:input=move |ev| update("zh", event_target_value(&ev)) /> }.into_any() }}</label>
                <label>"English"{if multiline { view! { <textarea prop:value=move || get_value().en on:input=move |ev| update("en", event_target_value(&ev))></textarea> }.into_any() } else { view! { <input prop:value=move || get_value().en on:input=move |ev| update("en", event_target_value(&ev)) /> }.into_any() }}</label>
            </div>
        </div>
    }
}

fn field_value(config: &HomePageConfig, field: &str) -> LocalizedText {
    match field {
        "hero_eyebrow" => config.hero.eyebrow.clone(),
        "hero_title" => config.hero.title.clone(),
        "hero_body" => config.hero.body.clone(),
        "hero_note" => config.hero.note.clone(),
        "hero_primary_label" => config.hero.primary_label.clone(),
        "hero_secondary_label" => config.hero.secondary_label.clone(),
        "hero_sample_location" => config.hero.sample_location.clone(),
        "hero_sample_title" => config.hero.sample_title.clone(),
        "hero_sample_body" => config.hero.sample_body.clone(),
        "hero_sample_guide_label" => config.hero.sample_guide_label.clone(),
        "hero_sample_question" => config.hero.sample_question.clone(),
        "hero_sample_presence" => config.hero.sample_presence.clone(),
        "journey_eyebrow" => config.journey.eyebrow.clone(),
        "journey_title" => config.journey.title.clone(),
        "arrive_title" => config.journey.arrive_title.clone(),
        "guide_step_title" => config.journey.guide_title.clone(),
        "help_title" => config.journey.help_title.clone(),
        "journey_body" => config.journey.body.clone(),
        "arrive_body" => config.journey.arrive_body.clone(),
        "guide_step_body" => config.journey.guide_body.clone(),
        "help_body" => config.journey.help_body.clone(),
        "guide_eyebrow" => config.guide.eyebrow.clone(),
        "guide_title" => config.guide.title.clone(),
        "guide_visual_route" => config.guide.visual_route.clone(),
        "guide_visual_warning" => config.guide.visual_warning.clone(),
        "guide_visual_live" => config.guide.visual_live.clone(),
        "guide_cta_label" => config.guide.cta_label.clone(),
        "guide_body" => config.guide.body.clone(),
        "host_title" => config.host.title.clone(),
        "host_body" => config.host.body.clone(),
        "host_cta_label" => config.host.cta_label.clone(),
        "nav_home" => config.nav.home.clone(),
        "nav_map" => config.nav.map.clone(),
        "nav_guides" => config.nav.guides.clone(),
        "nav_my_spaces" => config.nav.my_spaces.clone(),
        "seo_title" => config.seo.title.clone(),
        _ => config.seo.description.clone(),
    }
}

fn set_field_value(config: &mut HomePageConfig, field: &str, lang: &str, value: String) {
    let target = match field {
        "hero_eyebrow" => &mut config.hero.eyebrow,
        "hero_title" => &mut config.hero.title,
        "hero_body" => &mut config.hero.body,
        "hero_note" => &mut config.hero.note,
        "hero_primary_label" => &mut config.hero.primary_label,
        "hero_secondary_label" => &mut config.hero.secondary_label,
        "hero_sample_location" => &mut config.hero.sample_location,
        "hero_sample_title" => &mut config.hero.sample_title,
        "hero_sample_body" => &mut config.hero.sample_body,
        "hero_sample_guide_label" => &mut config.hero.sample_guide_label,
        "hero_sample_question" => &mut config.hero.sample_question,
        "hero_sample_presence" => &mut config.hero.sample_presence,
        "journey_eyebrow" => &mut config.journey.eyebrow,
        "journey_title" => &mut config.journey.title,
        "arrive_title" => &mut config.journey.arrive_title,
        "guide_step_title" => &mut config.journey.guide_title,
        "help_title" => &mut config.journey.help_title,
        "journey_body" => &mut config.journey.body,
        "arrive_body" => &mut config.journey.arrive_body,
        "guide_step_body" => &mut config.journey.guide_body,
        "help_body" => &mut config.journey.help_body,
        "guide_eyebrow" => &mut config.guide.eyebrow,
        "guide_title" => &mut config.guide.title,
        "guide_visual_route" => &mut config.guide.visual_route,
        "guide_visual_warning" => &mut config.guide.visual_warning,
        "guide_visual_live" => &mut config.guide.visual_live,
        "guide_cta_label" => &mut config.guide.cta_label,
        "guide_body" => &mut config.guide.body,
        "host_title" => &mut config.host.title,
        "host_body" => &mut config.host.body,
        "host_cta_label" => &mut config.host.cta_label,
        "nav_home" => &mut config.nav.home,
        "nav_map" => &mut config.nav.map,
        "nav_guides" => &mut config.nav.guides,
        "nav_my_spaces" => &mut config.nav.my_spaces,
        "seo_title" => &mut config.seo.title,
        _ => &mut config.seo.description,
    };
    if lang == "zh" {
        target.zh = value;
    } else {
        target.en = value;
    }
}

fn localized(locale: Locale, value: &LocalizedText) -> String {
    match locale {
        Locale::Zh => value.zh.clone(),
        Locale::En => value.en.clone(),
    }
}
