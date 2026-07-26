use instant_domain::site::{HomePageConfig, LocalizedText};
use leptos::prelude::*;
use leptos_meta::{Meta, Title};

use crate::{
    components::space_form::{provide_create_space_modal, use_create_space_modal},
    i18n::{use_i18n, Locale},
    server::site::get_public_home_config,
};

/// Public product homepage. Content and controlled design tokens come from the
/// published site-page configuration; code defaults keep the page available if
/// the database or configuration is unavailable.
#[component]
pub fn HomePage() -> impl IntoView {
    let config = Resource::new(
        || (),
        |_| async move { get_public_home_config().await.unwrap_or_default() },
    );

    view! {
        <Suspense fallback=move || view! { <HomePageContent config=HomePageConfig::default() /> }>
            {move || Suspend::new(async move {
                view! { <HomePageContent config=config.await /> }
            })}
        </Suspense>
    }
}

#[component]
fn HomePageContent(config: HomePageConfig) -> impl IntoView {
    let locale = use_i18n().locale;
    let create_modal = use_create_space_modal().unwrap_or_else(provide_create_space_modal);

    let theme_style = format!(
        "--home-primary:{};--home-deep:{};--home-bg:{};",
        config.theme.primary, config.theme.deep, config.theme.background
    );
    let page_class = format!(
        "page inspace-home theme-{} density-{} hero-layout-{}",
        config.theme.preset, config.theme.density, config.theme.hero_layout
    );

    let seo_title = config.seo.title.clone();
    let seo_description = config.seo.description.clone();
    let hero = config.hero;
    let journey = config.journey;
    let guide = config.guide;
    let host = config.host;

    view! {
        <Title text=move || localize(locale.get(), &seo_title) />
        <Meta name="description" content=move || localize(locale.get(), &seo_description) />
        <main id="main-content" class=page_class style=theme_style>
            <div class="inspace-home-modules">
                {hero.visible.then(|| {
                    let eyebrow = hero.eyebrow.clone();
                    let title = hero.title.clone();
                    let body = hero.body.clone();
                    let note = hero.note.clone();
                    let sample_location = hero.sample_location.clone();
                    let sample_title = hero.sample_title.clone();
                    let sample_body = hero.sample_body.clone();
                    let sample_guide_label = hero.sample_guide_label.clone();
                    let sample_question = hero.sample_question.clone();
                    let sample_presence = hero.sample_presence.clone();
                    let primary_label = hero.primary_label.clone();
                    let secondary_label = hero.secondary_label.clone();
                    let primary_url = hero.primary_url.clone();
                    let secondary_url = hero.secondary_url.clone();
                    view! {
                        <section class="survey-hero" style=format!("order:{}", hero.order) aria-labelledby="inspace-home-title">
                            <div class="survey-hero-copy">
                                <p class="survey-kicker">
                                    <span class="survey-kicker-mark" aria-hidden="true"></span>
                                    {move || localize(locale.get(), &eyebrow)}
                                </p>
                                <h1 id="inspace-home-title">{move || localize(locale.get(), &title)}</h1>
                                <p class="survey-lede">{move || localize(locale.get(), &body)}</p>
                                <div class="survey-actions">
                                    <a class="button button-primary" href=primary_url>{move || localize(locale.get(), &primary_label)}</a>
                                    <a class="button button-secondary" href=secondary_url>{move || localize(locale.get(), &secondary_label)}</a>
                                </div>
                                <p class="survey-note">{move || localize(locale.get(), &note)}</p>
                            </div>
                            <figure class="survey-sheet" aria-label=move || match locale.get() { Locale::Zh => "空间记录示例（示意内容）", Locale::En => "Example space record (illustrative)" }>
                                <figcaption class="survey-sheet-head">
                                    <span class="survey-sheet-ref">{move || localize(locale.get(), &sample_location)}</span>
                                    <span class="survey-sheet-stamp">{move || match locale.get() { Locale::Zh => "示例", Locale::En => "SAMPLE" }}</span>
                                </figcaption>
                                <div class="survey-sheet-body">
                                    <h2 class="survey-sheet-title">{move || localize(locale.get(), &sample_title)}</h2>
                                    <p class="survey-sheet-lede">{move || localize(locale.get(), &sample_body)}</p>
                                    <dl class="survey-record">
                                        <div>
                                            <dt>{move || match locale.get() { Locale::Zh => "攻略", Locale::En => "Guide" }}</dt>
                                            <dd>{move || localize(locale.get(), &sample_guide_label)}</dd>
                                        </div>
                                        <div>
                                            <dt>{move || match locale.get() { Locale::Zh => "现场提问", Locale::En => "Asked on site" }}</dt>
                                            <dd>{move || localize(locale.get(), &sample_question)}</dd>
                                        </div>
                                        <div>
                                            <dt>{move || match locale.get() { Locale::Zh => "此刻", Locale::En => "Right now" }}</dt>
                                            <dd>{move || localize(locale.get(), &sample_presence)}</dd>
                                        </div>
                                    </dl>
                                </div>
                            </figure>
                        </section>
                    }
                })}

                {journey.visible.then(|| {
                    let title = journey.title.clone();
                    let body = journey.body.clone();
                    let arrive_title = journey.arrive_title.clone();
                    let arrive_body = journey.arrive_body.clone();
                    let guide_title = journey.guide_title.clone();
                    let guide_body = journey.guide_body.clone();
                    let help_title = journey.help_title.clone();
                    let help_body = journey.help_body.clone();
                    view! {
                        <section class="survey-passage" style=format!("order:{}", journey.order) aria-labelledby="inspace-journey-title">
                            <header class="survey-passage-head">
                                <h2 id="inspace-journey-title">{move || localize(locale.get(), &title)}</h2>
                                <p>{move || localize(locale.get(), &body)}</p>
                            </header>
                            <dl class="survey-stages">
                                <div><dt>{move || localize(locale.get(), &arrive_title)}</dt><dd>{move || localize(locale.get(), &arrive_body)}</dd></div>
                                <div><dt>{move || localize(locale.get(), &guide_title)}</dt><dd>{move || localize(locale.get(), &guide_body)}</dd></div>
                                <div><dt>{move || localize(locale.get(), &help_title)}</dt><dd>{move || localize(locale.get(), &help_body)}</dd></div>
                            </dl>
                        </section>
                    }
                })}

                {guide.visible.then(|| {
                    let eyebrow = guide.eyebrow.clone();
                    let title = guide.title.clone();
                    let body = guide.body.clone();
                    let visual_route = guide.visual_route.clone();
                    let visual_warning = guide.visual_warning.clone();
                    let visual_live = guide.visual_live.clone();
                    let cta_label = guide.cta_label.clone();
                    let cta_url = guide.cta_url.clone();
                    view! {
                        <section class="survey-plate" style=format!("order:{}", guide.order) aria-labelledby="inspace-guide-title">
                            <div class="survey-plate-copy">
                                <p class="survey-kicker"><span class="survey-kicker-mark" aria-hidden="true"></span>{move || localize(locale.get(), &eyebrow)}</p>
                                <h2 id="inspace-guide-title">{move || localize(locale.get(), &title)}</h2>
                                <p>{move || localize(locale.get(), &body)}</p>
                                <a class="button button-primary" href=cta_url>{move || localize(locale.get(), &cta_label)}</a>
                            </div>
                            <table class="survey-log">
                                <caption>{move || match locale.get() { Locale::Zh => "一份空间攻略的实际条目（示意）", Locale::En => "Entries from one space guide (illustrative)" }}</caption>
                                <tbody>
                                    <tr><th scope="row">{move || match locale.get() { Locale::Zh => "路线", Locale::En => "Route" }}</th><td>{move || localize(locale.get(), &visual_route)}</td></tr>
                                    <tr><th scope="row">{move || match locale.get() { Locale::Zh => "避坑", Locale::En => "Warning" }}</th><td>{move || localize(locale.get(), &visual_warning)}</td></tr>
                                    <tr><th scope="row">{move || match locale.get() { Locale::Zh => "现场", Locale::En => "Live" }}</th><td>{move || localize(locale.get(), &visual_live)}</td></tr>
                                </tbody>
                            </table>
                        </section>
                    }
                })}

                {host.visible.then(|| {
                    let title = host.title.clone();
                    let body = host.body.clone();
                    let cta_label = host.cta_label.clone();
                    view! {
                        <section id="create" class="survey-colophon" style=format!("order:{}", host.order)>
                            <h2>{move || localize(locale.get(), &title)}</h2>
                            <p>{move || localize(locale.get(), &body)}</p>
                            <button type="button" class="button button-secondary" on:click=move |_| create_modal.open.set(true)>{move || localize(locale.get(), &cta_label)}</button>
                        </section>
                    }
                })}
            </div>
        </main>
    }
}

fn localize(locale: Locale, value: &LocalizedText) -> String {
    match locale {
        Locale::Zh => value.zh.clone(),
        Locale::En => value.en.clone(),
    }
}
