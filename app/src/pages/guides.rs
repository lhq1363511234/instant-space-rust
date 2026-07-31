use instant_domain::guides::{GuideSection, GuideStatus};
use leptos::prelude::*;
use leptos_router::hooks::{use_location, use_params_map};
use url::Url;

#[cfg(feature = "hydrate")]
use wasm_bindgen::JsCast;

use crate::components::guide_browser::GuideBrowser;
use crate::i18n::{localize_optional, t, use_i18n};
use crate::server::{
    auth::current_session,
    guides::{
        create_guide_draft, delete_guide, get_guide_detail, get_guide_for_edit, list_guide_versions,
        restore_guide_version, update_guide,
    },
    spaces::{get_space_for_guide, list_spaces, SpaceMarker},
};

#[component]
pub fn GuidesPage() -> impl IntoView {
    view! {
        <main id="main-content" class="page">
            <GuideBrowser />
        </main>
    }
}

#[component]
pub fn GuideDetailPage() -> impl IntoView {
    let locale = use_i18n().locale;
    let params = use_params_map();
    let guide_id =
        Memo::new(move |_| params.with(|params| params.get("guide_id").unwrap_or_default()));
    let guide = Resource::new(
        move || guide_id.get(),
        |guide_id| async move {
            if guide_id.is_empty() {
                None
            } else {
                get_guide_detail(guide_id).await.ok().flatten()
            }
        },
    );

    view! {
        <main id="main-content" class="page guide-detail-page">
            <Suspense fallback=move || view! { <p>{move || t(locale.get(), "正在加载攻略", "Loading guide")}</p> }>
                {move || Suspend::new(async move {
                    match guide.await {
                        Some(guide) => {
                            let title_zh = guide.title_zh.clone();
                            let title_en = guide.title_en.clone();
                            let summary_zh = guide.summary_zh.clone().unwrap_or_default();
                            let summary_en = guide.summary_en.clone();
                            let content_zh = guide.content_zh.clone().unwrap_or_default();
                            let content_en = guide.content_en.clone();
                            let location = guide_location(
                                &guide.province,
                                &guide.city,
                                guide.district.as_deref(),
                                guide.spot_name.as_deref(),
                            );
                            let sections = guide.sections.clone();
                            let images = guide.images.clone();
                            let cover = guide.cover_image_url.clone();
                            let author = guide.author_name.clone();
                            let guide_type = guide.guide_type.clone();
                            let category = guide.category.clone();
                            let community_name = guide
                                .spot_name
                                .clone()
                                .unwrap_or_else(|| title_zh.clone());
                            let space_href = guide.space_id.map(|id| format!("/inspace/spaces/{id}"));
                            let edit_href = format!("/inspace/guides/{}/edit", guide.id);
                            let can_edit = guide.can_edit;

                            view! {
                                <article class="guide-detail" aria-label="Guide detail">
                                    <header class="guide-detail-hero">
                                        <div>
                                            <a class="back-link" href="/inspace/guides">{move || t(locale.get(), "返回攻略", "Back to guides")}</a>
                                            <p class="eyebrow">{move || t(locale.get(), "全球攻略", "Global guide")}</p>
                                            <h1>{move || localize_optional(locale.get(), &title_zh, title_en.as_deref())}</h1>
                                            <p class="guide-detail-location">{location}</p>
                                            <div class="guide-detail-meta">
                                                <span>{guide_type}</span>
                                                {category.map(|value| view! { <span>{value}</span> })}
                                                {author.map(|value| view! { <span>{value}</span> })}
                                            </div>
                                        </div>
                                        {cover.filter(|url| !url.trim().is_empty()).map(|url| view! {
                                            <img class="guide-cover" src=url alt="" loading="lazy" />
                                        })}
                                    </header>

                                    {(!summary_zh.is_empty() || summary_en.as_deref().is_some_and(|value| !value.trim().is_empty())).then(|| view! {
                                        <section class="guide-summary">
                                            <p>{move || localize_optional(locale.get(), &summary_zh, summary_en.as_deref())}</p>
                                        </section>
                                    })}

                                    {(!content_zh.is_empty() || content_en.as_deref().is_some_and(|value| !value.trim().is_empty())).then(|| view! {
                                        <section class="guide-content">
                                            <p>{move || localize_optional(locale.get(), &content_zh, content_en.as_deref())}</p>
                                        </section>
                                    })}

                                    <section class="guide-sections" aria-label="Guide sections">
                                        <h2>{move || t(locale.get(), "攻略内容", "Guide sections")}</h2>
                                        {if sections.is_empty() {
                                            view! {
                                                <div class="empty-state">
                                                    <strong>{move || t(locale.get(), "暂无结构化内容", "No structured sections yet")}</strong>
                                                    <span>{move || t(locale.get(), "后续 Phase 4.2 会加入结构化编辑器。", "Phase 4.2 will add the structured editor.")}</span>
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <div class="guide-section-list">
                                                    <For
                                                        each=move || sections.clone()
                                                        key=|section| section.id.clone()
                                                        children=move |section| {
                                                            let title_zh = section.title_zh.clone();
                                                            let title_en = section.title_en.clone();
                                                            let content_zh = section.content_zh.clone();
                                                            let content_en = section.content_en.clone();
                                                            let section_images = section.images.clone();
                                                            view! {
                                                                <section class="guide-section-card">
                                                                    <span class="guide-section-type">{section.section_type}</span>
                                                                    {(!title_zh.is_empty() || title_en.as_deref().is_some_and(|value| !value.trim().is_empty())).then(|| view! {
                                                                        <h3>{move || localize_optional(locale.get(), &title_zh, title_en.as_deref())}</h3>
                                                                    })}
                                                                    {(!content_zh.is_empty() || content_en.as_deref().is_some_and(|value| !value.trim().is_empty())).then(|| view! {
                                                                        <p>{move || localize_optional(locale.get(), &content_zh, content_en.as_deref())}</p>
                                                                    })}
                                                                    {(!section_images.is_empty()).then(|| view! {
                                                                        <div class="guide-image-grid">
                                                                            <For
                                                                                each=move || section_images.clone()
                                                                                key=|url| url.clone()
                                                                                children=move |url| view! { <img src=url alt="" loading="lazy" /> }
                                                                            />
                                                                        </div>
                                                                    })}
                                                                </section>
                                                            }
                                                        }
                                                    />
                                                </div>
                                            }.into_any()
                                        }}
                                    </section>

                                    {(!images.is_empty()).then(|| view! {
                                        <section class="guide-images" aria-label="Guide images">
                                            <h2>{move || t(locale.get(), "图片", "Images")}</h2>
                                            <div class="guide-image-grid">
                                                <For
                                                    each=move || images.clone()
                                                    key=|url| url.clone()
                                                    children=move |url| view! { <img src=url alt="" loading="lazy" /> }
                                                />
                                            </div>
                                        </section>
                                    })}

                                    <GuideCommunityLinks space_name=community_name />

                                    <footer class="guide-detail-actions">
                                        {space_href.map(|href| view! {
                                            <a class="button button-primary" href=href>{move || t(locale.get(), "打开关联空间", "Open linked space")}</a>
                                        })}
                                        {can_edit.then(|| view! {
                                            <a class="button button-secondary-light" href=edit_href>{move || t(locale.get(), "编辑攻略", "Edit guide")}</a>
                                        })}
                                        <a class="button button-secondary-light" href="/inspace/guides">{move || t(locale.get(), "继续浏览攻略", "Browse more guides")}</a>
                                    </footer>
                                </article>
                            }.into_any()
                        }
                        None => view! {
                            <section class="empty-state">
                                <strong>{move || t(locale.get(), "攻略不存在或未发布", "Guide not found or unpublished")}</strong>
                                <a class="button button-primary" href="/inspace/guides">{move || t(locale.get(), "返回攻略", "Back to guides")}</a>
                            </section>
                        }.into_any(),
                    }
                })}
            </Suspense>
        </main>
    }
}

#[component]
fn GuideCommunityLinks(space_name: String) -> impl IntoView {
    let locale = use_i18n().locale;
    view! {
        <aside class="community-links guide-community-links" aria-label=move || t(locale.get(), "攻略社群链接", "Guide community links")>
            <div>
                <strong>{move || t(locale.get(), "社群里的空间讨论组", "Space group in community")}</strong>
                <p>
                    {move || format!(
                        "{}「{}」{}",
                        t(locale.get(), "在 Discord / QQ 频道内搜索", "Search"),
                        space_name,
                        t(locale.get(), "，加入对应讨论组获取实时密码、路线更新和现场提醒。", " in Discord / QQ to join the matching group for live passwords, route updates, and on-site tips.")
                    )}
                </p>
            </div>
            <div class="community-link-actions">
                <a class="button button-secondary-light" href="https://discord.gg/zsmYWvXyy" target="_blank" rel="noreferrer">"Discord 社群"</a>
                <a class="button button-secondary-light" href="https://pd.qq.com/s/8ru51ih0m?b=9" target="_blank" rel="noreferrer">"QQ 频道【即时空间】"</a>
            </div>
        </aside>
    }
}

fn guide_location(
    province: &str,
    city: &str,
    district: Option<&str>,
    spot_name: Option<&str>,
) -> String {
    [Some(province), Some(city), district, spot_name]
        .into_iter()
        .flatten()
        .filter(|value| !value.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" / ")
}

#[derive(Debug, Clone, PartialEq)]
struct GuideDraft {
    guide_type: String,
    category: String,
    title_zh: String,
    title_en: String,
    summary_zh: String,
    summary_en: String,
    province: String,
    city: String,
    district: String,
    spot_name: String,
    space_id: Option<String>,
    images: Vec<String>,
    sections: Vec<GuideSection>,
    status: GuideStatus,
    featured: bool,
}

impl Default for GuideDraft {
    fn default() -> Self {
        Self {
            guide_type: "attraction".to_string(),
            category: String::new(),
            title_zh: String::new(),
            title_en: String::new(),
            summary_zh: String::new(),
            summary_en: String::new(),
            province: String::new(),
            city: String::new(),
            district: String::new(),
            spot_name: String::new(),
            space_id: None,
            images: Vec::new(),
            sections: vec![new_section(0)],
            status: GuideStatus::Draft,
            featured: false,
        }
    }
}

#[component]
pub fn GuideEditorPage() -> impl IntoView {
    let locale = use_i18n().locale;
    let location = use_location();
    let params = use_params_map();
    let edit_guide_id =
        Memo::new(move |_| params.with(|params| params.get("guide_id").unwrap_or_default()));
    let is_admin_mode = Memo::new(move |_| location.pathname.get().contains("/admin/"));
    let linked_space_id = Memo::new(move |_| {
        location.query.with(|query| {
            query
                .get("space_id")
                .filter(|value| !value.trim().is_empty())
        })
    });
    let session = Resource::new(
        || (),
        |_| async move { current_session().await.ok().flatten() },
    );
    let spaces = Resource::new(
        || (),
        |_| async move {
            list_spaces(None, None, None, None, None)
                .await
                .unwrap_or_default()
        },
    );
    let guide = RwSignal::new(GuideDraft::default());
    let saved_guide_id = RwSignal::new(None::<String>);
    let edit_guide = Resource::new(
        move || edit_guide_id.get(),
        |guide_id| async move {
            if guide_id.is_empty() {
                None
            } else {
                get_guide_for_edit(guide_id).await.ok().flatten()
            }
        },
    );
    let linked_space = Resource::new(
        move || linked_space_id.get(),
        |space_id| async move {
            match space_id {
                Some(space_id) => get_space_for_guide(space_id).await.ok().flatten(),
                None => None,
            }
        },
    );
    let did_prefill = RwSignal::new(false);

    Effect::new(move |_| {
        if did_prefill.get_untracked() {
            return;
        }
        if let Some(Some(existing)) = edit_guide.get() {
            saved_guide_id.set(Some(existing.id.to_string()));
            guide.set(guide_draft_from_detail(existing));
            did_prefill.set(true);
            return;
        }
        if !edit_guide_id.get_untracked().is_empty() {
            return;
        }
        if let Some(Some(space)) = linked_space.get() {
            guide.update(|draft| {
                apply_space_to_draft(draft, &space);
            });
            did_prefill.set(true);
        }
    });

    let save = Action::new(move |target_status: &GuideStatus| {
        let mut draft = guide.get();
        draft.status = *target_status;
        let edit_id = saved_guide_id.get().unwrap_or_else(|| edit_guide_id.get());
        async move {
            if edit_id.is_empty() {
                create_guide_draft(
                    draft.title_zh,
                    optional_text(draft.title_en),
                    optional_text(draft.summary_zh),
                    optional_text(draft.summary_en),
                    None,
                    None,
                    draft.guide_type,
                    optional_text(draft.category),
                    draft.province,
                    draft.city,
                    optional_text(draft.district),
                    optional_text(draft.spot_name),
                    draft.space_id,
                    draft.images.first().cloned(),
                    draft.images,
                    draft.sections,
                    draft.status,
                    draft.featured,
                )
                .await
            } else {
                update_guide(
                    edit_id,
                    draft.title_zh,
                    optional_text(draft.title_en),
                    optional_text(draft.summary_zh),
                    optional_text(draft.summary_en),
                    None,
                    None,
                    draft.guide_type,
                    optional_text(draft.category),
                    draft.province,
                    draft.city,
                    optional_text(draft.district),
                    optional_text(draft.spot_name),
                    draft.space_id,
                    draft.images.first().cloned(),
                    draft.images,
                    draft.sections,
                    draft.status,
                    draft.featured,
                )
                .await
            }
        }
    });
    let save_feedback = save;
    let delete = Action::new(move |_: &()| {
        let guide_id = saved_guide_id.get().unwrap_or_else(|| edit_guide_id.get());
        async move {
            if guide_id.is_empty() {
                Err(ServerFnError::new("guide not saved yet"))
            } else {
                delete_guide(guide_id).await
            }
        }
    });
    let delete_feedback = delete;
    let versions_reload = RwSignal::new(0u32);
    let version_guide_id = Memo::new(move |_| {
        saved_guide_id.get().unwrap_or_else(|| edit_guide_id.get())
    });

    Effect::new(move |_| {
        if let Some(Ok(summary)) = save_feedback.value().get() {
            saved_guide_id.set(Some(summary.id.to_string()));
            versions_reload.set(versions_reload.get() + 1);
        }
    });

    view! {
        <main id="main-content" class="page guide-editor-page">
            <Suspense fallback=move || view! { <p>{move || t(locale.get(), "正在检查登录状态", "Checking session")}</p> }>
                {move || Suspend::new(async move {
                    let user = session.await;
                    if user.is_none() {
                        view! {
                            <section class="form">
                                <h1>{move || t(locale.get(), "请先登录", "Sign in first")}</h1>
                                <p>{move || t(locale.get(), "登录后可以提交攻略草稿。", "Sign in to submit guide drafts.")}</p>
                                <a class="button button-primary" href="/inspace/login">{move || t(locale.get(), "去登录", "Go to sign in")}</a>
                            </section>
                        }.into_any()
                    } else {
                        let show_featured = user.as_ref().is_some_and(|user| user.role.is_admin()) && is_admin_mode.get();
                        let space_options = spaces.await;
                        view! {
                            <GuideEditor
                                guide=guide
                                is_edit=!edit_guide_id.get().is_empty()
                                space_options=space_options
                                show_featured=show_featured
                                on_save=Callback::new(move |status| {
                                    save.dispatch(status);
                                })
                                on_delete=Callback::new(move |_| {
                                    delete.dispatch(());
                                })
                            />
                            <GuideVersionHistory guide_id=version_guide_id reload=versions_reload />
                        }.into_any()
                    }
                })}
            </Suspense>
            {move || save_feedback.value().get().map(|result| match result {
                Ok(guide) => view! {
                    <div class="form-success">
                        {move || t(locale.get(), "攻略已保存。", "Guide saved.")}
                        " ID: "
                        <code>{guide.id.to_string()}</code>
                    </div>
                }.into_any(),
                Err(err) => view! { <p class="error">{err.to_string()}</p> }.into_any(),
            })}
            {move || delete_feedback.value().get().map(|result| match result {
                Ok(_) => view! {
                    <div class="form-success">
                        {move || t(locale.get(), "攻略已删除。", "Guide deleted.")}
                    </div>
                }.into_any(),
                Err(err) => view! { <p class="error">{err.to_string()}</p> }.into_any(),
            })}
        </main>
    }
}

#[component]
fn GuideEditor(
    guide: RwSignal<GuideDraft>,
    is_edit: bool,
    space_options: Vec<SpaceMarker>,
    show_featured: bool,
    on_save: Callback<GuideStatus>,
    on_delete: Callback<()>,
) -> impl IntoView {
    let locale = use_i18n().locale;
    let confirm_delete = RwSignal::new(false);

    let update_text = move |apply: fn(&mut GuideDraft, String), value: String| {
        guide.update(|draft| apply(draft, value));
    };
    let add_section = move |_| {
        guide.update(|draft| {
            let index = draft.sections.len();
            draft.sections.push(new_section(index));
        });
    };
    let images_change = Callback::new(move |images: Vec<String>| {
        guide.update(|draft| draft.images = images);
    });
    let selectable_spaces = space_options.clone();
    let linked_space_names = space_options
        .iter()
        .map(|space| (space.id.clone(), space_option_label(space)))
        .collect::<Vec<_>>();

    view! {
        <section class="guide-editor form" aria-label="Structured guide editor">
            <div class="page-head">
                <div>
                    <p class="survey-kicker">{move || t(locale.get(), "写一篇攻略", "Write a guide")}</p>
                    <h1>{move || if is_edit {
                        t(locale.get(), "编辑攻略", "Edit guide")
                    } else {
                        t(locale.get(), "写下这个地方", "Write down this place")
                    }}</h1>
                    <p>{move || t(locale.get(), "先选它属于哪个空间，地点信息会自动带入。然后按板块写：怎么去、什么时候来、花多少、哪里会踩坑。", "Pick the Space it belongs to and the place details fill in automatically. Then write it in sections: how to get there, when to come, what it costs, where people get caught out.")}</p>
                </div>
                <a class="button button-secondary-light" href="/inspace/guides">{move || t(locale.get(), "返回攻略", "Back to guides")}</a>
            </div>

            <form class="guide-editor-form" on:submit=move |ev| ev.prevent_default()>
                <section class="guide-editor-card">
                    <h2>{move || t(locale.get(), "基本信息", "Basic information")}</h2>
                    <label class="field-label">
                        <span>{move || t(locale.get(), "关联空间（可选）", "Linked space (optional)")}</span>
                        <select
                            aria-label="Guide linked space"
                            prop:value=move || guide.get().space_id.unwrap_or_default()
                            on:change=move |ev| {
                                let value = event_target_value(&ev);
                                guide.update(|draft| {
                                    if value.is_empty() {
                                        draft.space_id = None;
                                    } else if let Some(space) = selectable_spaces.iter().find(|space| space.id == value) {
                                        apply_space_to_draft(draft, space);
                                    } else {
                                        draft.space_id = Some(value.clone());
                                    }
                                });
                            }
                        >
                            <option value="">{move || t(locale.get(), "先不关联（之后仍可挂到空间下）", "Not linked yet (you can attach it to a Space later)")}</option>
                            <For
                                each=move || space_options.clone()
                                key=|space| space.id.clone()
                                children=move |space| {
                                    let label = space_option_label(&space);
                                    view! { <option value=space.id.clone()>{label}</option> }
                                }
                            />
                        </select>
                    </label>
                    {move || guide.get().space_id.map(|space_id| {
                        let href = format!("/inspace/spaces/{space_id}");
                        let label = linked_space_names
                            .iter()
                            .find(|(id, _)| *id == space_id)
                            .map(|(_, label)| label.clone())
                            .unwrap_or_else(|| space_id.clone());
                        view! {
                            <p class="guide-linked-space">
                                {move || t(locale.get(), "这篇攻略会挂在：", "This guide will live under: ")}
                                <a href=href>{label}</a>
                            </p>
                        }
                    })}
                    <div class="form-grid">
                        <label class="field-label">
                            <span>{move || t(locale.get(), "攻略类型", "Guide type")}</span>
                            <select
                                aria-label="Guide type"
                                prop:value=move || guide.get().guide_type
                                on:change=move |ev| update_text(|draft, value| draft.guide_type = value, event_target_value(&ev))
                            >
                                <option value="attraction">{move || t(locale.get(), "景点", "Attraction")}</option>
                                <option value="city">{move || t(locale.get(), "城市", "City")}</option>
                                <option value="district">{move || t(locale.get(), "区域", "District")}</option>
                            </select>
                        </label>
                        <label class="field-label">
                            <span>{move || t(locale.get(), "分类", "Category")}</span>
                            <input aria-label="Guide category" placeholder=move || t(locale.get(), "例如：亲子、摄影、徒步", "e.g. family, photo, walk") prop:value=move || guide.get().category on:input=move |ev| update_text(|draft, value| draft.category = value, event_target_value(&ev)) />
                        </label>
                        <label class="field-label">
                            <span>{move || t(locale.get(), "中文标题", "Chinese title")}</span>
                            <input aria-label="Guide Chinese title" required=true placeholder=move || t(locale.get(), "例如：外滩半日攻略", "e.g. Half-day guide to the Bund") prop:value=move || guide.get().title_zh on:input=move |ev| update_text(|draft, value| draft.title_zh = value, event_target_value(&ev)) />
                        </label>
                        <label class="field-label">
                            <span>{move || t(locale.get(), "英文标题", "English title")}</span>
                            <input aria-label="Guide English title" placeholder="Optional English title" prop:value=move || guide.get().title_en on:input=move |ev| update_text(|draft, value| draft.title_en = value, event_target_value(&ev)) />
                        </label>
                        <label class="field-label">
                            <span>{move || t(locale.get(), "省 / 州", "Province / State")}</span>
                            <input aria-label="Guide province" required=true placeholder="上海市 / Guangdong / California" prop:value=move || guide.get().province on:input=move |ev| update_text(|draft, value| draft.province = value, event_target_value(&ev)) />
                        </label>
                        <label class="field-label">
                            <span>{move || t(locale.get(), "城市", "City")}</span>
                            <input aria-label="Guide city" required=true placeholder="上海市 / Guangzhou / Los Angeles" prop:value=move || guide.get().city on:input=move |ev| update_text(|draft, value| draft.city = value, event_target_value(&ev)) />
                        </label>
                        <label class="field-label">
                            <span>{move || t(locale.get(), "区县", "District")}</span>
                            <input aria-label="Guide district" placeholder=move || t(locale.get(), "可选，例如：黄浦区", "Optional, e.g. Huangpu") prop:value=move || guide.get().district on:input=move |ev| update_text(|draft, value| draft.district = value, event_target_value(&ev)) />
                        </label>
                        <label class="field-label">
                            <span>{move || t(locale.get(), "地点 / 景点", "Spot")}</span>
                            <input aria-label="Guide spot" placeholder=move || t(locale.get(), "可选，例如：外滩", "Optional, e.g. The Bund") prop:value=move || guide.get().spot_name on:input=move |ev| update_text(|draft, value| draft.spot_name = value, event_target_value(&ev)) />
                        </label>
                    </div>
                    {show_featured.then(|| view! {
                        <label class="field-label guide-featured-toggle">
                            <span>{move || t(locale.get(), "首页推荐", "Featured on home")}</span>
                            <input type="checkbox" aria-label="Guide featured" prop:checked=move || guide.get().featured on:change=move |ev| guide.update(|draft| draft.featured = event_target_checked(&ev)) />
                        </label>
                    })}
                </section>

                <section class="guide-editor-card">
                    <h2>{move || t(locale.get(), "摘要", "Summary")}</h2>
                    <label class="field-label">
                        <span>{move || t(locale.get(), "中文摘要", "Chinese summary")}</span>
                        <textarea aria-label="Guide Chinese summary" rows="3" placeholder=move || t(locale.get(), "一句话说明这篇攻略适合谁、怎么玩。", "One sentence about who this guide is for and how to use it.") prop:value=move || guide.get().summary_zh on:input=move |ev| update_text(|draft, value| draft.summary_zh = value, event_target_value(&ev))></textarea>
                    </label>
                    <label class="field-label">
                        <span>{move || t(locale.get(), "英文摘要", "English summary")}</span>
                        <textarea aria-label="Guide English summary" rows="3" placeholder="Optional English summary" prop:value=move || guide.get().summary_en on:input=move |ev| update_text(|draft, value| draft.summary_en = value, event_target_value(&ev))></textarea>
                    </label>
                </section>

                <section class="guide-editor-card guide-editor-sections" aria-label="Guide structured sections">
                    <div class="card-head-inline">
                        <div>
                            <h2>{move || t(locale.get(), "板块列表", "Sections")}</h2>
                            <p>{move || t(locale.get(), "不是平铺富文本，每个板块都有类型、标题、正文和配图。", "Not flat rich text: every section has its type, title, body, and images.")}</p>
                        </div>
                        <button class="button button-secondary-light" type="button" on:click=add_section>{move || t(locale.get(), "新增板块", "Add section")}</button>
                    </div>
                    <For
                        each=move || { guide.get().sections.into_iter().enumerate().collect::<Vec<_>>() }
                        key=|(_, section)| section.id.clone()
                        children=move |(index, section)| {
                            let section_id = section.id.clone();
                            let change = Callback::new(move |updated: GuideSection| {
                                guide.update(|draft| {
                                    if let Some(existing) = draft.sections.iter_mut().find(|item| item.id == section_id) {
                                        *existing = updated;
                                    }
                                });
                            });
                            let delete_id = section.id.clone();
                            let delete = Callback::new(move |_| {
                                guide.update(|draft| draft.sections.retain(|item| item.id != delete_id));
                            });
                            view! {
                                <SectionEditor
                                    index=index
                                    section=section
                                    on_change=change
                                    on_delete=delete
                                />
                            }
                        }
                    />
                </section>

                <section class="guide-editor-card">
                    <h2>{move || t(locale.get(), "封面与图片", "Cover and images")}</h2>
                    <p>{move || t(locale.get(), "第一张图片会作为封面图；只保存 URL，上传能力后续接入对象存储。", "The first image is used as cover; URLs are stored now, object storage upload can be added later.")}</p>
                    {move || {
                        let images = guide.get().images;
                        view! { <ImageManager images=images on_change=images_change aria_label="Guide image URL".to_string() /> }
                    }}
                </section>

                <div class="form-actions guide-editor-actions">
                    <button class="button button-secondary-light" type="button" on:click=move |_| on_save.run(GuideStatus::Draft)>{move || t(locale.get(), "保存草稿", "Save draft")}</button>
                    <button class="button button-primary" type="button" on:click=move |_| on_save.run(GuideStatus::Published)>{move || t(locale.get(), "保存并发布", "Save and publish")}</button>
                    <button class="button button-danger" type="button" on:click=move |_| on_save.run(GuideStatus::Archived)>{move || t(locale.get(), "取消发布", "Unpublish")}</button>
                    {is_edit.then(|| view! {
                        <button
                            class="button button-danger"
                            type="button"
                            on:click=move |_| {
                                if confirm_delete.get() {
                                    confirm_delete.set(false);
                                    on_delete.run(());
                                } else {
                                    confirm_delete.set(true);
                                }
                            }
                        >
                            {move || if confirm_delete.get() {
                                t(locale.get(), "确认永久删除？", "Confirm permanent delete?").to_string()
                            } else {
                                t(locale.get(), "删除攻略", "Delete guide").to_string()
                            }}
                        </button>
                    })}
                </div>
            </form>
        </section>
    }
}

#[component]
fn SectionEditor(
    section: GuideSection,
    index: usize,
    on_change: Callback<GuideSection>,
    on_delete: Callback<()>,
) -> impl IntoView {
    let locale = use_i18n().locale;
    let number = index + 1;
    let heading_label = format!("Guide section {number} Chinese title");
    let body_label = format!("Guide section {number} Chinese content");
    let image_label = format!("Guide section {number} image URL");

    let type_section = section.clone();
    let title_zh_section = section.clone();
    let title_en_section = section.clone();
    let content_zh_section = section.clone();
    let content_en_section = section.clone();
    let images_section = section.clone();
    let section_images_change = Callback::new(move |images: Vec<String>| {
        let mut updated = images_section.clone();
        updated.images = images;
        on_change.run(updated);
    });

    view! {
        <fieldset class="guide-section-editor">
            <legend>{move || format!("{} {}", t(locale.get(), "板块", "Section"), number)}</legend>
            <div class="form-grid">
                <label class="field-label">
                    <span>{move || t(locale.get(), "类型", "Type")}</span>
                    <select
                        aria-label=format!("Guide section {number} type")
                        prop:value=section.section_type.clone()
                        on:change=move |ev| {
                            let mut updated = type_section.clone();
                            updated.section_type = event_target_value(&ev);
                            on_change.run(updated);
                        }
                    >
                        <option value="text">{move || t(locale.get(), "正文", "Text")}</option>
                        <option value="overview">{move || t(locale.get(), "概览", "Overview")}</option>
                        <option value="attractions">{move || t(locale.get(), "景点", "Attractions")}</option>
                        <option value="food">{move || t(locale.get(), "美食", "Food")}</option>
                        <option value="itinerary">{move || t(locale.get(), "路线", "Itinerary")}</option>
                        <option value="transport">{move || t(locale.get(), "交通", "Transport")}</option>
                        <option value="tips">{move || t(locale.get(), "提示", "Tips")}</option>
                    </select>
                </label>
                <label class="field-label">
                    <span>{move || t(locale.get(), "中文标题", "Chinese title")}</span>
                    <input
                        aria-label=heading_label
                        placeholder=move || t(locale.get(), "例如：怎么到达", "e.g. How to get there")
                        prop:value=section.title_zh.clone()
                        on:input=move |ev| {
                            let mut updated = title_zh_section.clone();
                            updated.title_zh = event_target_value(&ev);
                            on_change.run(updated);
                        }
                    />
                </label>
                <label class="field-label">
                    <span>{move || t(locale.get(), "英文标题", "English title")}</span>
                    <input
                        aria-label=format!("Guide section {number} English title")
                        placeholder="Optional English title"
                        prop:value=section.title_en.clone().unwrap_or_default()
                        on:input=move |ev| {
                            let mut updated = title_en_section.clone();
                            updated.title_en = optional_text(event_target_value(&ev));
                            on_change.run(updated);
                        }
                    />
                </label>
            </div>
            <label class="field-label">
                <span>{move || t(locale.get(), "中文内容", "Chinese content")}</span>
                <textarea
                    aria-label=body_label
                    rows="4"
                    placeholder=move || t(locale.get(), "写清楚步骤、建议和注意事项。", "Steps, tips, and cautions.")
                    prop:value=section.content_zh.clone()
                    on:input=move |ev| {
                        let mut updated = content_zh_section.clone();
                        updated.content_zh = event_target_value(&ev);
                        on_change.run(updated);
                    }
                ></textarea>
            </label>
            <label class="field-label">
                <span>{move || t(locale.get(), "英文内容", "English content")}</span>
                <textarea
                    aria-label=format!("Guide section {number} English content")
                    rows="4"
                    placeholder="Optional English content"
                    prop:value=section.content_en.clone().unwrap_or_default()
                    on:input=move |ev| {
                        let mut updated = content_en_section.clone();
                        updated.content_en = optional_text(event_target_value(&ev));
                        on_change.run(updated);
                    }
                ></textarea>
            </label>
            <ImageManager images=section.images.clone() on_change=section_images_change aria_label=image_label />
            <div class="form-actions section-actions">
                <button class="button button-danger" type="button" on:click=move |_| on_delete.run(())>{move || t(locale.get(), "删除板块", "Delete section")}</button>
            </div>
        </fieldset>
    }
}

#[component]
fn ImageManager(
    images: Vec<String>,
    on_change: Callback<Vec<String>>,
    aria_label: String,
) -> impl IntoView {
    let locale = use_i18n().locale;
    let url_input = RwSignal::new(String::new());
    let error = RwSignal::new(None::<String>);
    let upload_error = RwSignal::new(None::<String>);
    let uploading = RwSignal::new(false);
    let input_id = format!("guide-media-input-{}", uuid::Uuid::new_v4().simple());
    let add_images = images.clone();
    let preview_images = images.clone();
    let remove_source_images = images.clone();
    let upload_images = images.clone();
    let add = move |_| {
        let candidate = url_input.get().trim().to_string();
        if candidate.is_empty() {
            error.set(Some(
                t(locale.get(), "请输入图片 URL", "Enter an image URL").to_string(),
            ));
            return;
        }
        if Url::parse(&candidate).is_err() {
            error.set(Some(
                t(locale.get(), "图片 URL 格式不正确", "Invalid image URL").to_string(),
            ));
            return;
        }
        let mut next = add_images.clone();
        if !next.iter().any(|url| url == &candidate) {
            next.push(candidate);
        }
        error.set(None);
        url_input.set(String::new());
        on_change.run(next);
    };

    // Open the native file picker. The input is hidden but stays in the DOM so
    // the change event carries the chosen File object.
    let trigger_picker = {
        let _input_id = input_id.clone();
        move |_| {
            #[cfg(feature = "hydrate")]
            if let Some(el) = leptos::prelude::document().get_element_by_id(&_input_id) {
                let _ = el.unchecked_into::<web_sys::HtmlInputElement>().click();
            }
        }
    };

    view! {
        <div class="image-manager">
            <div class="image-manager-row">
                <input aria-label=aria_label.clone() placeholder="https://example.com/photo.jpg" prop:value=move || url_input.get() on:input=move |ev| url_input.set(event_target_value(&ev)) />
                <button class="button button-secondary-light" type="button" on:click=add>{move || t(locale.get(), "添加图片", "Add image")}</button>
                <button class="button button-secondary-light" type="button" on:click=trigger_picker>
                    {move || if uploading.get() {
                        t(locale.get(), "上传中…", "Uploading…").to_string()
                    } else {
                        t(locale.get(), "上传图片", "Upload image").to_string()
                    }}
                </button>
                <input
                    id=input_id.clone()
                    type="file"
                    accept="image/jpeg,image/png,image/webp,image/gif,image/avif"
                    class="visually-hidden"
                    aria-hidden="true"
                    tabindex="-1"
                    on:change=move |_ev| {
                        #[cfg(feature = "hydrate")]
                        {
                            let input = event_target::<web_sys::HtmlInputElement>(&_ev);
                            let Some(file) = input.files().and_then(|files| files.item(0)) else {
                                return;
                            };
                            if let Some(elem) = leptos::prelude::document().get_element_by_id(&input_id) {
                                elem.unchecked_into::<web_sys::HtmlInputElement>().set_value("");
                            }
                            upload_error.set(None);
                            uploading.set(true);
                            let images = upload_images.clone();
                            leptos::task::spawn_local(async move {
                                let outcome: Result<String, String> = async {
                                    let form = web_sys::FormData::new()
                                        .map_err(|_| t(locale.get(), "无法创建上传表单", "Cannot build upload form").to_string())?;
                                    form.append_with_blob("file", &file)
                                        .map_err(|_| t(locale.get(), "无法读取文件", "Cannot read file").to_string())?;
                                    let opts = web_sys::RequestInit::new();
                                    opts.set_method("POST");
                                    opts.set_body(&form.into());
                                    let request = web_sys::Request::new_with_str_and_init(
                                        "/inspace/api/media/upload",
                                        &opts,
                                    )
                                    .map_err(|_| t(locale.get(), "无法创建上传请求", "Cannot build upload request").to_string())?;
                                    let response = wasm_bindgen_futures::JsFuture::from(
                                        leptos::prelude::window().fetch_with_request(&request),
                                    )
                                    .await
                                    .map_err(|_| t(locale.get(), "网络错误，上传失败", "Network error during upload").to_string())?;
                                    let response: web_sys::Response = response.unchecked_into();
                                    let status = response.status();
                                    let text = wasm_bindgen_futures::JsFuture::from(
                                        response.text().map_err(|_| t(locale.get(), "读取响应失败", "Cannot read response").to_string())?,
                                    )
                                    .await
                                    .map_err(|_| t(locale.get(), "读取响应失败", "Cannot read response").to_string())?
                                    .as_string()
                                    .ok_or_else(|| t(locale.get(), "读取响应失败", "Cannot read response").to_string())?;
                                    if status != 200 {
                                        let message = serde_json::from_str::<serde_json::Value>(&text)
                                            .ok()
                                            .and_then(|value| value["error"].as_str().map(ToOwned::to_owned))
                                            .unwrap_or_else(|| t(locale.get(), "上传失败", "Upload failed").to_string());
                                        return Err(message);
                                    }
                                    let url = serde_json::from_str::<serde_json::Value>(&text)
                                        .ok()
                                        .and_then(|value| value["url"].as_str().map(ToOwned::to_owned))
                                        .ok_or_else(|| t(locale.get(), "上传响应缺少地址", "Upload response missing URL").to_string())?;
                                    Ok(url)
                                }
                                .await;
                                uploading.set(false);
                                match outcome {
                                    Ok(url) => {
                                        let mut next = images.clone();
                                        if !next.iter().any(|existing| existing == &url) {
                                            next.push(url);
                                        }
                                        on_change.run(next);
                                    }
                                    Err(message) => upload_error.set(Some(message)),
                                }
                            });
                        }
                    }
                />
            </div>
            {move || error.get().map(|message| view! { <p class="error">{message}</p> })}
            {move || upload_error.get().map(|message| view! { <p class="error">{message}</p> })}
            {if images.is_empty() {
                view! {
                    <div class="empty-state compact-empty">
                        <span>{move || t(locale.get(), "暂无图片，添加一个 URL 后会显示预览。", "No images yet. Add a URL to preview it.")}</span>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="image-preview-grid">
                        <For
                            each=move || { preview_images.clone().into_iter().enumerate().collect::<Vec<_>>() }
                            key=|(_, url)| url.clone()
                            children=move |(index, url)| {
                                let remove_images = remove_source_images.clone();
                                let remove = on_change;
                                view! {
                                    <figure class="image-preview-card">
                                        <img src=url.clone() alt="" loading="lazy" />
                                        <button
                                            class="image-preview-remove"
                                            type="button"
                                            aria-label=move || t(locale.get(), "删除图片", "Remove image")
                                            on:click=move |_| {
                                                let mut next = remove_images.clone();
                                                if index < next.len() {
                                                    next.remove(index);
                                                }
                                                remove.run(next);
                                            }
                                        >"×"</button>
                                    </figure>
                                }
                            }
                        />
                    </div>
                }.into_any()
            }}
        </div>
    }
}

fn new_section(index: usize) -> GuideSection {
    GuideSection {
        id: format!("sec_{}_{}", uuid::Uuid::new_v4(), index + 1),
        section_type: "text".to_string(),
        title_zh: String::new(),
        title_en: None,
        content_zh: String::new(),
        content_en: None,
        images: Vec::new(),
    }
}

fn guide_draft_from_detail(guide: instant_domain::guides::GuideDetail) -> GuideDraft {
    GuideDraft {
        guide_type: guide.guide_type,
        category: guide.category.unwrap_or_default(),
        title_zh: guide.title_zh,
        title_en: guide.title_en.unwrap_or_default(),
        summary_zh: guide.summary_zh.unwrap_or_default(),
        summary_en: guide.summary_en.unwrap_or_default(),
        province: guide.province,
        city: guide.city,
        district: guide.district.unwrap_or_default(),
        spot_name: guide.spot_name.unwrap_or_default(),
        space_id: guide.space_id.map(|id| id.to_string()),
        images: guide.images,
        sections: if guide.sections.is_empty() {
            vec![new_section(0)]
        } else {
            guide.sections
        },
        status: guide.status,
        featured: guide.featured,
    }
}

fn apply_space_to_draft(draft: &mut GuideDraft, space: &SpaceMarker) {
    draft.space_id = Some(space.id.clone());
    draft.province = space.province.clone().unwrap_or_default();
    draft.city = space.city.clone().unwrap_or_default();
    draft.district = space.district.clone().unwrap_or_default();
    draft.spot_name = space
        .spot_name
        .clone()
        .unwrap_or_else(|| space.name_zh.clone());
    if draft.title_zh.trim().is_empty() {
        draft.title_zh = format!("{}攻略", space.name_zh);
    }
    if draft.title_en.trim().is_empty() {
        draft.title_en = space
            .name_en
            .as_ref()
            .map(|name| format!("{name} Guide"))
            .unwrap_or_default();
    }
}

fn space_option_label(space: &SpaceMarker) -> String {
    [
        Some(space.name_zh.as_str()),
        space.province.as_deref(),
        space.city.as_deref(),
        space.district.as_deref(),
    ]
    .into_iter()
    .flatten()
    .filter(|value| !value.trim().is_empty())
    .collect::<Vec<_>>()
    .join(" / ")
}

fn optional_text(value: String) -> Option<String> {
    let value = value.trim().to_string();
    (!value.is_empty()).then_some(value)
}

/// Phase 4 content versioning: show a guide's snapshot history and let the
/// editor restore any version. Restoring snapshots the pre-restore state, so
/// nothing is lost.
#[component]
fn GuideVersionHistory(
    guide_id: Memo<String>,
    reload: RwSignal<u32>,
) -> impl IntoView {
    let locale = use_i18n().locale;
    let versions = Resource::new(
        move || (guide_id.get(), reload.get()),
        |(id, _)| async move {
            if id.is_empty() {
                Vec::new()
            } else {
                list_guide_versions(id).await.unwrap_or_default()
            }
        },
    );
    let restore = Action::new(move |version_no: &i32| {
        let guide_id = guide_id.get_untracked();
        let version_no = *version_no;
        async move { restore_guide_version(guide_id, version_no).await }
    });
    let restore_feedback = restore;

    view! {
        <section class="guide-editor-card guide-versions" aria-label=move || t(locale.get(), "攻略版本历史", "Guide version history")>
            <h2>{move || t(locale.get(), "版本历史", "Version history")}</h2>
            <p>{move || t(locale.get(), "每次保存都会冻结一份快照，编辑失误可随时回退。", "Every save freezes a snapshot; revert any mistake at any time.")}</p>
            <Suspense fallback=move || view! { <div class="space-list-skeleton"><span></span><span></span></div> }>
                {move || Suspend::new(async move {
                    let items = versions.await;
                    if items.is_empty() {
                        view! { <div class="empty-state compact-empty"><span>{move || t(locale.get(), "暂无版本记录。", "No versions yet.")}</span></div> }.into_any()
                    } else {
                        view! {
                            <ol class="guide-version-list">
                                <For
                                    each=move || items.clone()
                                    key=|version| format!("{}-{}", version.id, version.version_no)
                                    children=move |version| {
                                        let version_for_button = version.version_no;
                                        let title = if version.title_zh.trim().is_empty() {
                                            t(locale.get(), "(无标题)", "(untitled)").to_string()
                                        } else {
                                            version.title_zh.clone()
                                        };
                                        view! {
                                            <li class="guide-version-row">
                                                <span class="guide-version-no">"v" {version.version_no}</span>
                                                <div class="guide-version-main">
                                                    <strong>{title}</strong>
                                                    <span class="guide-version-meta">
                                                        {move || {
                                                            let mut parts = Vec::new();
                                                            if let Some(name) = version.edited_by_name.clone() {
                                                                parts.push(name);
                                                            }
                                                            parts.push(version.created_at.clone());
                                                            parts.join(" · ")
                                                        }}
                                                    </span>
                                                </div>
                                                <button
                                                    class="button button-secondary-light"
                                                    type="button"
                                                    on:click=move |_| { restore.dispatch(version_for_button); }
                                                >
                                                    {move || t(locale.get(), "恢复此版本", "Restore")}
                                                </button>
                                            </li>
                                        }
                                    }
                                />
                            </ol>
                        }.into_any()
                    }
                })}
            </Suspense>
            {move || restore_feedback.value().get().map(|result| match result {
                Ok(_summary) => view! {
                    <div class="form-success">
                        {move || t(locale.get(), "已恢复到该版本。", "Restored to that version.")}
                        " "
                        {move || t(locale.get(), "当前页面内容已过期，请刷新查看。", "The editor still shows the old content; refresh to reload it.")}
                    </div>
                }.into_any(),
                Err(err) => view! { <p class="error">{err.to_string()}</p> }.into_any(),
            })}
        </section>
    }
}
