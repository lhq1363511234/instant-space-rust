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
    let draft = initial.draft;
    let config = RwSignal::new(draft.clone());
    let saved_snapshot = RwSignal::new(draft);
    let published_version = RwSignal::new(initial.published_version);
    let version_reload = RwSignal::new(0u32);
    let notice = RwSignal::new(None::<String>);
    let error = RwSignal::new(None::<String>);
    let preview_device = RwSignal::new("desktop");
    let editing_locale = RwSignal::new(Locale::Zh);
    let selected = RwSignal::new("hero");

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
                    saved_snapshot.set(state.draft.clone());
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
                    saved_snapshot.set(state.draft.clone());
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
                    saved_snapshot.set(state.draft.clone());
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
        <section class="admin-page-builder">
            <header class="admin-builder-topbar">
                <div class="admin-builder-title">
                    <span class="admin-builder-mark" aria-hidden="true"><BuilderIcon name="page" /></span>
                    <span>
                        <strong>{move || t(locale.get(), "首页编辑器", "Homepage editor")}</strong>
                        <small>{move || format!("inspace · {} v{}", t(locale.get(), "线上版本", "Published"), published_version.get())}</small>
                    </span>
                </div>
                <div class="admin-builder-status" role="status">
                    <span class=move || if config.get() != saved_snapshot.get() { "is-dirty" } else { "" } aria-hidden="true"></span>
                    {move || if config.get() != saved_snapshot.get() { t(locale.get(), "有未保存修改", "Unsaved changes") } else { t(locale.get(), "草稿已同步", "Draft synced") }}
                </div>
                <div class="admin-builder-actions">
                    <a class="button button-secondary" href="/inspace" target="_blank" rel="noreferrer">{move || t(locale.get(), "查看线上", "View live")}</a>
                    <button class="button button-secondary" type="button" disabled=move || config.get() == saved_snapshot.get() on:click=move |_| config.set(saved_snapshot.get())>{move || t(locale.get(), "撤销修改", "Discard")}</button>
                    <button class="button button-secondary" type="button" disabled=move || save.pending().get() || config.get() == saved_snapshot.get() on:click=move |_| { save.dispatch(config.get()); }>{move || if save.pending().get() { t(locale.get(), "保存中…", "Saving…") } else { t(locale.get(), "保存草稿", "Save draft") }}</button>
                    <button class="button button-primary" type="button" disabled=move || publish.pending().get() on:click=move |_| { publish.dispatch(config.get()); }>{move || if publish.pending().get() { t(locale.get(), "发布中…", "Publishing…") } else { t(locale.get(), "发布", "Publish") }}</button>
                </div>
            </header>

            {move || notice.get().map(|value| view! { <p class="admin-builder-toast success" role="status">{value}</p> })}
            {move || error.get().map(|value| view! { <p class="admin-builder-toast error" role="alert">{value}</p> })}

            <div class="admin-builder-workspace">
                <aside class="admin-builder-outline" aria-label=move || t(locale.get(), "页面结构", "Page structure")>
                    <div class="admin-builder-pane-head">
                        <strong>{move || t(locale.get(), "页面结构", "Page structure")}</strong>
                        <small>{move || t(locale.get(), "选择区块，调整顺序或显示状态", "Select, reorder, or hide sections")}</small>
                    </div>
                    <div class="admin-builder-section-list">
                        <BuilderModuleRow config=config selected=selected module="hero" number="01" label_zh="首屏主张" label_en="Hero" />
                        <BuilderModuleRow config=config selected=selected module="journey" number="02" label_zh="到达与互助" label_en="Journey" />
                        <BuilderModuleRow config=config selected=selected module="guide" number="03" label_zh="攻略价值" label_en="Guide value" />
                        <BuilderModuleRow config=config selected=selected module="host" number="04" label_zh="主理人入口" label_en="Host CTA" />
                    </div>
                    <div class="admin-builder-outline-settings">
                        <span>{move || t(locale.get(), "页面设置", "Page settings")}</span>
                        <BuilderSettingRow selected=selected panel="theme" label_zh="主题与版式" label_en="Theme & layout" icon="palette" />
                        <BuilderSettingRow selected=selected panel="seo" label_zh="导航与 SEO" label_en="Navigation & SEO" icon="search" />
                        <BuilderSettingRow selected=selected panel="history" label_zh="发布历史" label_en="Version history" icon="history" />
                    </div>
                    <p class="admin-builder-outline-note">{move || t(locale.get(), "首页只保留一条清晰叙事线。区块越少，重点越明确。", "Keep one clear narrative. Fewer sections make the focus stronger.")}</p>
                </aside>

                <section class="admin-builder-canvas-shell" aria-label=move || t(locale.get(), "页面画布", "Page canvas")>
                    <div class="admin-builder-canvasbar">
                        <div>
                            <strong>{move || t(locale.get(), "草稿画布", "Draft canvas")}</strong>
                            <small>{move || t(locale.get(), "点击画布中的区块即可编辑", "Click a section on the canvas to edit")}</small>
                        </div>
                        <div class="admin-builder-canvas-controls">
                            <div role="group" aria-label=move || t(locale.get(), "预览设备", "Preview device")>
                                <DeviceButton signal=preview_device value="desktop" label_zh="桌面" label_en="Desktop" icon="desktop" />
                                <DeviceButton signal=preview_device value="tablet" label_zh="平板" label_en="Tablet" icon="tablet" />
                                <DeviceButton signal=preview_device value="mobile" label_zh="手机" label_en="Mobile" icon="mobile" />
                            </div>
                            <div role="group" aria-label=move || t(locale.get(), "编辑语言", "Editing language")>
                                <button type="button" class=move || if editing_locale.get()==Locale::Zh { "is-active" } else { "" } aria-pressed=move || (editing_locale.get()==Locale::Zh).to_string() on:click=move |_| editing_locale.set(Locale::Zh)>"中"</button>
                                <button type="button" class=move || if editing_locale.get()==Locale::En { "is-active" } else { "" } aria-pressed=move || (editing_locale.get()==Locale::En).to_string() on:click=move |_| editing_locale.set(Locale::En)>"EN"</button>
                            </div>
                        </div>
                    </div>
                    <div class="admin-builder-stage">
                        <div class=move || format!("admin-builder-viewport is-{}", preview_device.get()) style=move || format!("--home-primary:{};--home-deep:{};--home-bg:{}", config.get().theme.primary, config.get().theme.deep, config.get().theme.background)>
                            <div class="admin-builder-page-preview">
                                <Show when=move || config.get().hero.visible>
                                    <button type="button" style=move || format!("order:{}", config.get().hero.order) class=move || if selected.get()=="hero" { "admin-canvas-section canvas-hero is-selected" } else { "admin-canvas-section canvas-hero" } on:click=move |_| selected.set("hero")>
                                        <span class="admin-canvas-section-tag">{move || t(locale.get(), "首屏", "Hero")}</span>
                                        <small>{move || localized(editing_locale.get(), &config.get().hero.eyebrow)}</small>
                                        <h2>{move || localized(editing_locale.get(), &config.get().hero.title)}</h2>
                                        <p>{move || localized(editing_locale.get(), &config.get().hero.body)}</p>
                                        <span class="admin-canvas-cta">{move || localized(editing_locale.get(), &config.get().hero.primary_label)}</span>
                                    </button>
                                </Show>
                                <Show when=move || config.get().journey.visible>
                                    <button type="button" style=move || format!("order:{}", config.get().journey.order) class=move || if selected.get()=="journey" { "admin-canvas-section canvas-journey is-selected" } else { "admin-canvas-section canvas-journey" } on:click=move |_| selected.set("journey")>
                                        <span class="admin-canvas-section-tag">{move || t(locale.get(), "旅程", "Journey")}</span>
                                        <small>{move || localized(editing_locale.get(), &config.get().journey.eyebrow)}</small>
                                        <h3>{move || localized(editing_locale.get(), &config.get().journey.title)}</h3>
                                        <div class="admin-canvas-steps"><span>{move || localized(editing_locale.get(), &config.get().journey.arrive_title)}</span><span>{move || localized(editing_locale.get(), &config.get().journey.guide_title)}</span><span>{move || localized(editing_locale.get(), &config.get().journey.help_title)}</span></div>
                                    </button>
                                </Show>
                                <Show when=move || config.get().guide.visible>
                                    <button type="button" style=move || format!("order:{}", config.get().guide.order) class=move || if selected.get()=="guide" { "admin-canvas-section canvas-guide is-selected" } else { "admin-canvas-section canvas-guide" } on:click=move |_| selected.set("guide")>
                                        <span class="admin-canvas-section-tag">{move || t(locale.get(), "攻略", "Guide")}</span>
                                        <small>{move || localized(editing_locale.get(), &config.get().guide.eyebrow)}</small>
                                        <h3>{move || localized(editing_locale.get(), &config.get().guide.title)}</h3>
                                        <p>{move || localized(editing_locale.get(), &config.get().guide.body)}</p>
                                        <div class="admin-canvas-record"><span>{move || localized(editing_locale.get(), &config.get().guide.visual_route)}</span><span>{move || localized(editing_locale.get(), &config.get().guide.visual_warning)}</span></div>
                                    </button>
                                </Show>
                                <Show when=move || config.get().host.visible>
                                    <button type="button" style=move || format!("order:{}", config.get().host.order) class=move || if selected.get()=="host" { "admin-canvas-section canvas-host is-selected" } else { "admin-canvas-section canvas-host" } on:click=move |_| selected.set("host")>
                                        <span class="admin-canvas-section-tag">{move || t(locale.get(), "创建入口", "Host CTA")}</span>
                                        <h3>{move || localized(editing_locale.get(), &config.get().host.title)}</h3>
                                        <p>{move || localized(editing_locale.get(), &config.get().host.body)}</p>
                                        <span class="admin-canvas-cta">{move || localized(editing_locale.get(), &config.get().host.cta_label)}</span>
                                    </button>
                                </Show>
                                <Show when=move || !config.get().hero.visible && !config.get().journey.visible && !config.get().guide.visible && !config.get().host.visible>
                                    <div class="admin-builder-empty-canvas"><BuilderIcon name="page" /><strong>{move || t(locale.get(), "首页没有可见区块", "No visible sections")}</strong><p>{move || t(locale.get(), "请从左侧结构树重新显示至少一个区块。", "Turn on at least one section from the structure panel.")}</p></div>
                                </Show>
                            </div>
                        </div>
                    </div>
                </section>

                <aside class="admin-builder-inspector" aria-label=move || t(locale.get(), "区块属性", "Section properties")>
                    <form on:submit=move |ev| ev.prevent_default()>
                        <Show when=move || selected.get()=="hero"><InspectorPanel title_zh="首屏主张" title_en="Hero" hint_zh="第一屏文案、行动与地点示例" hint_en="First-view copy, actions, and place sample" config=config editing_locale=editing_locale module=Some("hero")>
                            <LocalizedEditorField config=config language=editing_locale field="hero_eyebrow" label_zh="眉题" label_en="Eyebrow" multiline=false />
                            <LocalizedEditorField config=config language=editing_locale field="hero_title" label_zh="主标题" label_en="Headline" multiline=true />
                            <p class="admin-inspector-field-help">{move || t(locale.get(), "按 Enter 可控制标题换行；保存并发布后，首页会保留相同断行。", "Press Enter to control headline line breaks. The published homepage keeps the same breaks.")}</p>
                            <LocalizedEditorField config=config language=editing_locale field="hero_body" label_zh="说明文案" label_en="Body" multiline=true />
                            <LocalizedEditorField config=config language=editing_locale field="hero_note" label_zh="补充说明" label_en="Note" multiline=false />
                            <InspectorGroup title_zh="行动按钮" title_en="Actions">
                                <LocalizedEditorField config=config language=editing_locale field="hero_primary_label" label_zh="主按钮文字" label_en="Primary label" multiline=false />
                                <label>{move || t(locale.get(), "主按钮链接", "Primary URL")}<input prop:value=move || config.get().hero.primary_url on:input=move |ev| config.update(|value| value.hero.primary_url = event_target_value(&ev)) /></label>
                                <LocalizedEditorField config=config language=editing_locale field="hero_secondary_label" label_zh="次按钮文字" label_en="Secondary label" multiline=false />
                                <label>{move || t(locale.get(), "次按钮链接", "Secondary URL")}<input prop:value=move || config.get().hero.secondary_url on:input=move |ev| config.update(|value| value.hero.secondary_url = event_target_value(&ev)) /></label>
                            </InspectorGroup>
                            <InspectorGroup title_zh="地点示例" title_en="Place sample">
                                <LocalizedEditorField config=config language=editing_locale field="hero_sample_location" label_zh="地点" label_en="Location" multiline=false />
                                <LocalizedEditorField config=config language=editing_locale field="hero_sample_title" label_zh="标题" label_en="Title" multiline=true />
                                <LocalizedEditorField config=config language=editing_locale field="hero_sample_body" label_zh="说明" label_en="Body" multiline=true />
                                <LocalizedEditorField config=config language=editing_locale field="hero_sample_guide_label" label_zh="攻略标签" label_en="Guide label" multiline=false />
                                <LocalizedEditorField config=config language=editing_locale field="hero_sample_question" label_zh="现场问题" label_en="Live question" multiline=true />
                                <LocalizedEditorField config=config language=editing_locale field="hero_sample_presence" label_zh="在线状态" label_en="Presence" multiline=false />
                            </InspectorGroup>
                        </InspectorPanel></Show>

                        <Show when=move || selected.get()=="journey"><InspectorPanel title_zh="到达与互助" title_en="Journey" hint_zh="说明用户到达地点后的三步体验" hint_en="The three steps after arrival" config=config editing_locale=editing_locale module=Some("journey")>
                            <LocalizedEditorField config=config language=editing_locale field="journey_eyebrow" label_zh="眉题" label_en="Eyebrow" multiline=false />
                            <LocalizedEditorField config=config language=editing_locale field="journey_title" label_zh="模块标题" label_en="Section title" multiline=true />
                            <LocalizedEditorField config=config language=editing_locale field="journey_body" label_zh="模块说明" label_en="Section body" multiline=true />
                            <InspectorGroup title_zh="三步体验" title_en="Three-step experience">
                                <LocalizedEditorField config=config language=editing_locale field="arrive_title" label_zh="到达标题" label_en="Arrival title" multiline=false />
                                <LocalizedEditorField config=config language=editing_locale field="arrive_body" label_zh="到达说明" label_en="Arrival copy" multiline=true />
                                <LocalizedEditorField config=config language=editing_locale field="guide_step_title" label_zh="攻略标题" label_en="Guide title" multiline=false />
                                <LocalizedEditorField config=config language=editing_locale field="guide_step_body" label_zh="攻略说明" label_en="Guide copy" multiline=true />
                                <LocalizedEditorField config=config language=editing_locale field="help_title" label_zh="互助标题" label_en="Help title" multiline=false />
                                <LocalizedEditorField config=config language=editing_locale field="help_body" label_zh="互助说明" label_en="Help copy" multiline=true />
                            </InspectorGroup>
                        </InspectorPanel></Show>

                        <Show when=move || selected.get()=="guide"><InspectorPanel title_zh="攻略价值" title_en="Guide value" hint_zh="呈现空间攻略如何沉淀地点经验" hint_en="How guides preserve place knowledge" config=config editing_locale=editing_locale module=Some("guide")>
                            <LocalizedEditorField config=config language=editing_locale field="guide_eyebrow" label_zh="眉题" label_en="Eyebrow" multiline=false />
                            <LocalizedEditorField config=config language=editing_locale field="guide_title" label_zh="模块标题" label_en="Section title" multiline=true />
                            <LocalizedEditorField config=config language=editing_locale field="guide_body" label_zh="模块说明" label_en="Section body" multiline=true />
                            <InspectorGroup title_zh="攻略示例" title_en="Guide sample">
                                <LocalizedEditorField config=config language=editing_locale field="guide_visual_route" label_zh="路线提示" label_en="Route" multiline=true />
                                <LocalizedEditorField config=config language=editing_locale field="guide_visual_warning" label_zh="避坑提示" label_en="Warning" multiline=true />
                                <LocalizedEditorField config=config language=editing_locale field="guide_visual_live" label_zh="现场更新" label_en="Live update" multiline=true />
                            </InspectorGroup>
                            <LocalizedEditorField config=config language=editing_locale field="guide_cta_label" label_zh="按钮文字" label_en="Button label" multiline=false />
                            <label>{move || t(locale.get(), "按钮链接", "Button URL")}<input prop:value=move || config.get().guide.cta_url on:input=move |ev| config.update(|value| value.guide.cta_url = event_target_value(&ev)) /></label>
                        </InspectorPanel></Show>

                        <Show when=move || selected.get()=="host"><InspectorPanel title_zh="主理人入口" title_en="Host CTA" hint_zh="邀请熟悉地点的人创建空间" hint_en="Invite people to create a space" config=config editing_locale=editing_locale module=Some("host")>
                            <LocalizedEditorField config=config language=editing_locale field="host_title" label_zh="标题" label_en="Title" multiline=true />
                            <LocalizedEditorField config=config language=editing_locale field="host_body" label_zh="说明" label_en="Body" multiline=true />
                            <LocalizedEditorField config=config language=editing_locale field="host_cta_label" label_zh="按钮文字" label_en="Button label" multiline=false />
                        </InspectorPanel></Show>

                        <Show when=move || selected.get()=="theme"><InspectorPanel title_zh="主题与版式" title_en="Theme & layout" hint_zh="控制颜色、密度与首屏构图" hint_en="Color, density, and hero composition" config=config editing_locale=editing_locale module=None>
                            <div class="admin-theme-presets" role="group" aria-label=move || t(locale.get(), "主题预设", "Theme preset")>
                                <ThemeButton config=config value="sky-ocean" label_zh="月白与墨" label_en="Paper & ink" />
                                <ThemeButton config=config value="sky" label_zh="月白与青瓷" label_en="Paper & celadon" />
                                <ThemeButton config=config value="ocean" label_zh="深墨沉浸" label_en="Deep ink" />
                            </div>
                            <div class="admin-color-fields">
                                <label>{move || t(locale.get(), "主色", "Primary")}<span><input type="color" prop:value=move || config.get().theme.primary on:input=move |ev| config.update(|value| value.theme.primary = event_target_value(&ev)) /><code>{move || config.get().theme.primary}</code></span></label>
                                <label>{move || t(locale.get(), "深色", "Deep")}<span><input type="color" prop:value=move || config.get().theme.deep on:input=move |ev| config.update(|value| value.theme.deep = event_target_value(&ev)) /><code>{move || config.get().theme.deep}</code></span></label>
                                <label>{move || t(locale.get(), "背景色", "Background")}<span><input type="color" prop:value=move || config.get().theme.background on:input=move |ev| config.update(|value| value.theme.background = event_target_value(&ev)) /><code>{move || config.get().theme.background}</code></span></label>
                            </div>
                            <label>{move || t(locale.get(), "首屏版式", "Hero layout")}<select prop:value=move || config.get().theme.hero_layout on:change=move |ev| config.update(|value| value.theme.hero_layout = event_target_value(&ev))><option value="split">{move || t(locale.get(), "图文双栏", "Split")}</option><option value="centered">{move || t(locale.get(), "文字居中", "Centered")}</option></select></label>
                            <label>{move || t(locale.get(), "信息密度", "Density")}<select prop:value=move || config.get().theme.density on:change=move |ev| config.update(|value| value.theme.density = event_target_value(&ev))><option value="comfortable">{move || t(locale.get(), "舒适", "Comfortable")}</option><option value="compact">{move || t(locale.get(), "紧凑", "Compact")}</option></select></label>
                        </InspectorPanel></Show>

                        <Show when=move || selected.get()=="seo"><InspectorPanel title_zh="导航与 SEO" title_en="Navigation & SEO" hint_zh="控制导航名称和搜索结果摘要" hint_en="Navigation labels and search metadata" config=config editing_locale=editing_locale module=None>
                            <InspectorGroup title_zh="导航名称" title_en="Navigation labels">
                                <LocalizedEditorField config=config language=editing_locale field="nav_home" label_zh="首页" label_en="Home" multiline=false />
                                <LocalizedEditorField config=config language=editing_locale field="nav_map" label_zh="地图" label_en="Map" multiline=false />
                                <LocalizedEditorField config=config language=editing_locale field="nav_guides" label_zh="攻略" label_en="Guides" multiline=false />
                                <LocalizedEditorField config=config language=editing_locale field="nav_my_spaces" label_zh="我的空间" label_en="My spaces" multiline=false />
                            </InspectorGroup>
                            <InspectorGroup title_zh="搜索结果" title_en="Search result">
                                <LocalizedEditorField config=config language=editing_locale field="seo_title" label_zh="SEO 标题" label_en="SEO title" multiline=true />
                                <LocalizedEditorField config=config language=editing_locale field="seo_description" label_zh="SEO 描述" label_en="SEO description" multiline=true />
                                <div class="admin-search-snippet"><span>{move || t(locale.get(), "opctoai.com/inspace", "opctoai.com/inspace")}</span><strong>{move || localized(editing_locale.get(), &config.get().seo.title)}</strong><p>{move || localized(editing_locale.get(), &config.get().seo.description)}</p></div>
                            </InspectorGroup>
                        </InspectorPanel></Show>

                        <Show when=move || selected.get()=="history"><section class="admin-inspector-panel">
                            <header class="admin-inspector-head"><span class="admin-inspector-icon"><BuilderIcon name="history" /></span><div><h2>{move || t(locale.get(), "发布历史", "Version history")}</h2><p>{move || t(locale.get(), "将任一线上版本恢复为当前草稿", "Restore any published version as the current draft")}</p></div></header>
                            <Suspense fallback=move || view! { <p>{move || t(locale.get(), "正在加载…", "Loading…")}</p> }>
                                {move || Suspend::new(async move {
                                    let items = versions.await;
                                    if items.is_empty() {
                                        view! { <div class="admin-history-empty"><BuilderIcon name="history" /><p>{move || t(locale.get(), "还没有发布版本。", "No published versions yet.")}</p></div> }.into_any()
                                    } else {
                                        view! { <div class="admin-builder-history"><For each=move || items.clone() key=|item| item.id children=move |item| { let id=item.id.to_string(); view! { <div><span><strong>{format!("v{}", item.version)}</strong><small>{item.created_at}</small></span><button type="button" on:click=move |_| { restore.dispatch(id.clone()); }>{move || t(locale.get(), "恢复", "Restore")}</button></div> } } /></div> }.into_any()
                                    }
                                })}
                            </Suspense>
                        </section></Show>
                    </form>
                </aside>
            </div>
        </section>
    }
}

#[component]
fn BuilderModuleRow(
    config: RwSignal<HomePageConfig>,
    selected: RwSignal<&'static str>,
    module: &'static str,
    number: &'static str,
    label_zh: &'static str,
    label_en: &'static str,
) -> impl IntoView {
    let locale = use_i18n().locale;
    view! {
        <div class=move || {
            let mut class = String::from("admin-builder-module-row");
            if selected.get() == module { class.push_str(" is-selected"); }
            if !module_visible(&config.get(), module) { class.push_str(" is-hidden"); }
            class
        }>
            <button type="button" class="admin-builder-module-main" aria-pressed=move || (selected.get()==module).to_string() on:click=move |_| selected.set(module)>
                <span>{number}</span>
                <span><strong>{move || t(locale.get(), label_zh, label_en)}</strong><small>{move || if module_visible(&config.get(), module) { t(locale.get(), "显示中", "Visible") } else { t(locale.get(), "已隐藏", "Hidden") }}</small></span>
            </button>
            <div class="admin-builder-module-actions">
                <button type="button" title=move || t(locale.get(), "上移", "Move up") aria-label=move || format!("{} · {}", t(locale.get(), label_zh, label_en), t(locale.get(), "上移", "Move up")) on:click=move |_| move_module(config, module, -1)><BuilderIcon name="up" /></button>
                <button type="button" title=move || t(locale.get(), "下移", "Move down") aria-label=move || format!("{} · {}", t(locale.get(), label_zh, label_en), t(locale.get(), "下移", "Move down")) on:click=move |_| move_module(config, module, 1)><BuilderIcon name="down" /></button>
                <button type="button" class="visibility" title=move || if module_visible(&config.get(), module) { t(locale.get(), "隐藏", "Hide") } else { t(locale.get(), "显示", "Show") } aria-label=move || format!("{} · {}", t(locale.get(), label_zh, label_en), if module_visible(&config.get(), module) { t(locale.get(), "隐藏", "Hide") } else { t(locale.get(), "显示", "Show") }) on:click=move |_| toggle_module(config, module)>{move || if module_visible(&config.get(), module) { view! { <BuilderIcon name="eye" /> }.into_any() } else { view! { <BuilderIcon name="eye-off" /> }.into_any() }}</button>
            </div>
        </div>
    }
}

#[component]
fn BuilderSettingRow(
    selected: RwSignal<&'static str>,
    panel: &'static str,
    label_zh: &'static str,
    label_en: &'static str,
    icon: &'static str,
) -> impl IntoView {
    let locale = use_i18n().locale;
    view! { <button type="button" class=move || if selected.get()==panel { "is-selected" } else { "" } aria-pressed=move || (selected.get()==panel).to_string() on:click=move |_| selected.set(panel)><BuilderIcon name=icon /><span>{move || t(locale.get(), label_zh, label_en)}</span><BuilderIcon name="chevron" /></button> }
}

#[component]
fn DeviceButton(
    signal: RwSignal<&'static str>,
    value: &'static str,
    label_zh: &'static str,
    label_en: &'static str,
    icon: &'static str,
) -> impl IntoView {
    let locale = use_i18n().locale;
    view! { <button type="button" class=move || if signal.get()==value { "is-active" } else { "" } aria-pressed=move || (signal.get()==value).to_string() aria-label=move || t(locale.get(), label_zh, label_en) on:click=move |_| signal.set(value)><BuilderIcon name=icon /></button> }
}

#[component]
fn InspectorPanel(
    title_zh: &'static str,
    title_en: &'static str,
    hint_zh: &'static str,
    hint_en: &'static str,
    config: RwSignal<HomePageConfig>,
    editing_locale: RwSignal<Locale>,
    module: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    let locale = use_i18n().locale;
    view! {
        <section class="admin-inspector-panel">
            <header class="admin-inspector-head">
                <span class="admin-inspector-icon"><BuilderIcon name=if module.is_some() { "section" } else { "settings" } /></span>
                <div><h2>{move || t(locale.get(), title_zh, title_en)}</h2><p>{move || t(locale.get(), hint_zh, hint_en)}</p></div>
            </header>
            <div class="admin-inspector-language" role="group" aria-label=move || t(locale.get(), "内容语言", "Content language")>
                <button type="button" class=move || if editing_locale.get()==Locale::Zh { "is-active" } else { "" } aria-pressed=move || (editing_locale.get()==Locale::Zh).to_string() on:click=move |_| editing_locale.set(Locale::Zh)>{move || t(locale.get(), "中文内容", "Chinese")}</button>
                <button type="button" class=move || if editing_locale.get()==Locale::En { "is-active" } else { "" } aria-pressed=move || (editing_locale.get()==Locale::En).to_string() on:click=move |_| editing_locale.set(Locale::En)>{move || t(locale.get(), "英文内容", "English")}</button>
            </div>
            {module.map(|module| view! {
                <div class="admin-inspector-section-state">
                    <span><strong>{move || t(locale.get(), "区块状态", "Section status")}</strong><small>{move || if module_visible(&config.get(), module) { t(locale.get(), "当前在首页显示", "Visible on homepage") } else { t(locale.get(), "当前已从首页隐藏", "Hidden from homepage") }}</small></span>
                    <button type="button" role="switch" aria-checked=move || module_visible(&config.get(), module).to_string() class=move || if module_visible(&config.get(), module) { "is-on" } else { "" } on:click=move |_| toggle_module(config, module)><span></span></button>
                </div>
            })}
            <div class="admin-inspector-fields">{children()}</div>
        </section>
    }
}

#[component]
fn InspectorGroup(
    title_zh: &'static str,
    title_en: &'static str,
    children: Children,
) -> impl IntoView {
    let locale = use_i18n().locale;
    view! { <details class="admin-inspector-group" open><summary>{move || t(locale.get(), title_zh, title_en)}<BuilderIcon name="chevron" /></summary><div>{children()}</div></details> }
}

#[component]
fn ThemeButton(
    config: RwSignal<HomePageConfig>,
    value: &'static str,
    label_zh: &'static str,
    label_en: &'static str,
) -> impl IntoView {
    let locale = use_i18n().locale;
    view! { <button type="button" class=move || if config.get().theme.preset == value { "theme-preset is-active" } else { "theme-preset" } aria-pressed=move || if config.get().theme.preset == value { "true" } else { "false" } on:click=move |_| config.update(|config| config.theme.preset = value.to_string())><span aria-hidden="true"></span>{move || t(locale.get(), label_zh, label_en)}</button> }
}

#[component]
fn LocalizedEditorField(
    config: RwSignal<HomePageConfig>,
    language: RwSignal<Locale>,
    field: &'static str,
    label_zh: &'static str,
    label_en: &'static str,
    multiline: bool,
) -> impl IntoView {
    let locale = use_i18n().locale;
    let value = move || localized(language.get(), &field_value(&config.get(), field));
    let update = move |next: String| {
        let lang = if language.get() == Locale::Zh {
            "zh"
        } else {
            "en"
        };
        config.update(|config| set_field_value(config, field, lang, next));
    };
    view! {
        <label class="admin-editor-field">
            <span>{move || t(locale.get(), label_zh, label_en)}<small>{move || if language.get()==Locale::Zh { "中文" } else { "EN" }}</small></span>
            {if multiline { view! { <textarea prop:value=value on:input=move |ev| update(event_target_value(&ev))></textarea> }.into_any() } else { view! { <input prop:value=value on:input=move |ev| update(event_target_value(&ev)) /> }.into_any() }}
        </label>
    }
}

#[component]
fn BuilderIcon(#[prop(into)] name: Signal<&'static str>) -> impl IntoView {
    move || {
        match name.get() {
        "page" => view! { <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M6 3.5h9l3 3V20.5H6z"/><path d="M15 3.5v3h3M9 11h6M9 15h6"/></svg> }.into_any(),
        "palette" => view! { <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 3.5a8.5 8.5 0 0 0 0 17h1.2a1.8 1.8 0 0 0 0-3.6h-.7a1.8 1.8 0 0 1 0-3.6H16a4.5 4.5 0 0 0 0-9z"/><circle cx="8" cy="10" r=".7"/><circle cx="10" cy="7" r=".7"/><circle cx="14" cy="7" r=".7"/></svg> }.into_any(),
        "search" => view! { <svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="10.5" cy="10.5" r="6.5"/><path d="m15.5 15.5 5 5"/></svg> }.into_any(),
        "history" => view! { <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M4 8V4m0 0h4M4 4l3 3a8 8 0 1 1-2 8"/><path d="M12 8v5l3 2"/></svg> }.into_any(),
        "desktop" => view! { <svg viewBox="0 0 24 24" aria-hidden="true"><rect x="3" y="4" width="18" height="13" rx="1.5"/><path d="M8 21h8M12 17v4"/></svg> }.into_any(),
        "tablet" => view! { <svg viewBox="0 0 24 24" aria-hidden="true"><rect x="5" y="2.5" width="14" height="19" rx="2"/><path d="M11 18.5h2"/></svg> }.into_any(),
        "mobile" => view! { <svg viewBox="0 0 24 24" aria-hidden="true"><rect x="7" y="2" width="10" height="20" rx="2"/><path d="M11 18.5h2"/></svg> }.into_any(),
        "eye" => view! { <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M3 12s3.5-6 9-6 9 6 9 6-3.5 6-9 6-9-6-9-6z"/><circle cx="12" cy="12" r="2.5"/></svg> }.into_any(),
        "eye-off" => view! { <svg viewBox="0 0 24 24" aria-hidden="true"><path d="m4 4 16 16M10 6.3A9.8 9.8 0 0 1 12 6c5.5 0 9 6 9 6a16 16 0 0 1-2.2 3M6.2 7.5A16 16 0 0 0 3 12s3.5 6 9 6a9.8 9.8 0 0 0 2-.2"/><path d="M10.5 10.5A2 2 0 0 0 13.5 13.5"/></svg> }.into_any(),
        "up" => view! { <svg viewBox="0 0 24 24" aria-hidden="true"><path d="m7 14 5-5 5 5"/></svg> }.into_any(),
        "down" => view! { <svg viewBox="0 0 24 24" aria-hidden="true"><path d="m7 10 5 5 5-5"/></svg> }.into_any(),
        "chevron" => view! { <svg viewBox="0 0 24 24" aria-hidden="true"><path d="m9 6 6 6-6 6"/></svg> }.into_any(),
        "section" => view! { <svg viewBox="0 0 24 24" aria-hidden="true"><rect x="3.5" y="5" width="17" height="14" rx="1.5"/><path d="M7 9h10M7 13h7"/></svg> }.into_any(),
        _ => view! { <svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="8"/><path d="M12 8v4M12 16h.01"/></svg> }.into_any(),
    }
    }
}

fn module_visible(config: &HomePageConfig, module: &str) -> bool {
    match module {
        "hero" => config.hero.visible,
        "journey" => config.journey.visible,
        "guide" => config.guide.visible,
        _ => config.host.visible,
    }
}

fn toggle_module(config: RwSignal<HomePageConfig>, module: &'static str) {
    config.update(|value| match module {
        "hero" => value.hero.visible = !value.hero.visible,
        "journey" => value.journey.visible = !value.journey.visible,
        "guide" => value.guide.visible = !value.guide.visible,
        _ => value.host.visible = !value.host.visible,
    });
}

fn move_module(config: RwSignal<HomePageConfig>, module: &'static str, direction: i32) {
    config.update(|value| {
        let mut modules = vec![
            ("hero", value.hero.order),
            ("journey", value.journey.order),
            ("guide", value.guide.order),
            ("host", value.host.order),
        ];
        modules.sort_by_key(|(_, order)| *order);
        let Some(index) = modules.iter().position(|(name, _)| *name == module) else {
            return;
        };
        let next = index as i32 + direction;
        if next < 0 || next >= modules.len() as i32 {
            return;
        }
        modules.swap(index, next as usize);
        for (position, (name, _)) in modules.into_iter().enumerate() {
            let order = ((position + 1) * 10) as i32;
            match name {
                "hero" => value.hero.order = order,
                "journey" => value.journey.order = order,
                "guide" => value.guide.order = order,
                _ => value.host.order = order,
            }
        }
    });
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
