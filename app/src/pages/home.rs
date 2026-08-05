use instant_domain::{
    site::{HomePageConfig, LocalizedText},
    spaces::SpaceType,
};
use leptos::prelude::*;
use leptos_meta::{Meta, Title};

use crate::{
    components::{
        space_experience_modal::{
            use_space_experience_modal, OpenSpaceLink, SpaceExperienceModalState,
        },
        space_form::{provide_create_space_modal, use_create_space_modal},
    },
    i18n::{use_i18n, Locale},
    pages::space::SpacePanel,
    server::{
        site::get_public_home_config,
        spaces::{
            list_home_featured_spaces, list_home_featured_stories, HomeStoryView, SpaceMarker,
        },
    },
};

/// Public product homepage. The editable hero stays in the site-page
/// configuration. Editorial selections below it come from real Spaces and
/// traces, never fabricated activity counters.
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
    let host = config.host;

    view! {
        <Title text=move || localize(locale.get(), &seo_title) />
        <Meta name="description" content=move || localize(locale.get(), &seo_description) />
        <main id="main-content" class=page_class style=theme_style>
            <div class="inspace-home-modules home-discovery-flow">
                {hero.visible.then(|| {
                    let eyebrow = hero.eyebrow.clone();
                    let title = hero.title.clone();
                    let body = hero.body.clone();
                    let primary_label = hero.primary_label.clone();
                    let secondary_label = hero.secondary_label.clone();
                    let primary_url = hero.primary_url.clone();
                    let secondary_url = hero.secondary_url.clone();
                    view! {
                        <section class="survey-hero home-record-hero" style=format!("order:{}", hero.order) aria-labelledby="inspace-home-title">
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
                            </div>
                            <figure class="home-river-scroll" aria-label=move || match locale.get() { Locale::Zh => "北宋王希孟《千里江山图》长卷", Locale::En => "A Thousand Li of Rivers and Mountains by Wang Ximeng" }>
                                <div class="home-river-scroll-viewport">
                                    <div class="home-river-scroll-track" aria-hidden="true">
                                        <img src="/inspace/vendor/img/culture/thousand-li-rivers-mountains.webp" width="6000" height="1272" loading="eager" decoding="async" fetchpriority="high" alt="" />
                                        <img src="/inspace/vendor/img/culture/thousand-li-rivers-mountains.webp" width="6000" height="1272" loading="eager" decoding="async" alt="" />
                                    </div>
                                </div>
                                <figcaption>{move || match locale.get() { Locale::Zh => "北宋 · 王希孟 · 千里江山图", Locale::En => "Northern Song · Wang Ximeng · A Thousand Li of Rivers and Mountains" }}</figcaption>
                            </figure>
                        </section>
                    }
                })}

                <HomeExampleCarousel />
                <HomeFeaturedSpaces />
                <HomeFeaturedStories />

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
                        <section class="survey-passage home-journey" style=format!("order:{}", journey.order) aria-labelledby="inspace-journey-title">
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

                {host.visible.then(|| {
                    let title = host.title.clone();
                    let body = host.body.clone();
                    let cta_label = host.cta_label.clone();
                    view! {
                        <section id="create" class="survey-colophon home-host-call" style=format!("order:{}", host.order)>
                            <div>
                                <h2>{move || localize(locale.get(), &title)}</h2>
                                <p>{move || localize(locale.get(), &body)}</p>
                                <p class="survey-host-open-call">{move || match locale.get() {
                                    Locale::Zh => "空间主理人招募中。真正熟悉这里的人，最适合替这个地点留下第一份长期记录。",
                                    Locale::En => "Space hosts are wanted. The people who know a place are the right people to keep its first long record.",
                                }}</p>
                            </div>
                            <div class="survey-host-actions">
                                <button type="button" class="button button-primary" on:click=move |_| create_modal.open.set(true)>{move || localize(locale.get(), &cta_label)}</button>
                                <a class="button button-quiet" href="/inspace/about#hosts">{move || match locale.get() { Locale::Zh => "了解主理人计划", Locale::En => "About the host programme" }}</a>
                            </div>
                        </section>
                    }
                })}
            </div>
        </main>
    }
}

/// Four practical examples make the abstract idea of a Space tangible. They
/// are deliberately illustrative, labelled as examples, and never presented as
/// real activity or testimonials.
#[component]
fn HomeExampleCarousel() -> impl IntoView {
    let locale = use_i18n().locale;
    view! {
        <section class="home-examples" style="order:20" data-home-carousel aria-roledescription="carousel" aria-label=move || match locale.get() {
            Locale::Zh => "空间示例",
            Locale::En => "Space examples",
        }>
            <div class="home-examples-head">
                <div>
                    <h2>{move || match locale.get() { Locale::Zh => "一个地点，可以留下些什么？", Locale::En => "What can a place keep?" }}</h2>
                    <p>{move || match locale.get() {
                        Locale::Zh => "不是再多一张介绍页，而是让到过这里的人，把有用的事情留在地点名下。",
                        Locale::En => "Not another profile page. A Space keeps useful things with the place for the next person.",
                    }}</p>
                </div>
                <div class="home-carousel-controls">
                    <button type="button" class="home-carousel-arrow" data-home-carousel-prev aria-label=move || match locale.get() { Locale::Zh => "查看上一个示例", Locale::En => "Previous example" }>{"←"}</button>
                    <button type="button" class="home-carousel-arrow" data-home-carousel-next aria-label=move || match locale.get() { Locale::Zh => "查看下一个示例", Locale::En => "Next example" }>{"→"}</button>
                </div>
            </div>
            <div class="home-carousel-viewport">
                <ExampleSlide
                    id="home-example-0"
                    tab_id="home-example-tab-0"
                    active=true
                    image="/inspace/vendor/img/place-lake-1080.webp"
                    kind_zh="景点空间"
                    kind_en="Landmark Space"
                    title_zh="一座山的路线，不必每次从头问起。"
                    title_en="A mountain route should not start from zero each time."
                    body_zh="日出前从哪条路上，风大时在哪里停，后来的人都能在到达之前看见。"
                    body_en="Which path to take before sunrise and where to stop in wind can be there before the next arrival."
                    action_zh="看景点空间"
                    action_en="Explore landmarks"
                    href="/inspace/explore"
                />
                <ExampleSlide
                    id="home-example-1"
                    tab_id="home-example-tab-1"
                    active=false
                    image="/inspace/vendor/img/place-alley-1080.webp"
                    kind_zh="美食空间"
                    kind_en="Food Space"
                    title_zh="一家小店，留下它真正的营业节奏。"
                    title_en="A small restaurant keeps its real rhythm."
                    body_zh="主理人写清招牌和时间，熟客补上怎么点、怎么吃，以及这家店为什么值得再来。"
                    body_en="The host records the signatures and hours. Regulars add how to order, eat, and return."
                    action_zh="看美食空间"
                    action_en="Explore food spaces"
                    href="/inspace/explore"
                />
                <ExampleSlide
                    id="home-example-2"
                    tab_id="home-example-tab-2"
                    active=false
                    image="/inspace/vendor/img/place-lane-1080.webp"
                    kind_zh="公园空间"
                    kind_en="Park Space"
                    title_zh="每天路过的人，最懂一座公园。"
                    title_en="Daily visitors know a park best."
                    body_zh="哪扇门先开，花什么时候盛，傍晚哪里安静。这些日常经验终于有了留下来的地方。"
                    body_en="Which gate opens first, when flowers peak, where dusk is quiet. Daily knowledge finally has a home."
                    action_zh="看公园空间"
                    action_en="Explore parks"
                    href="/inspace/explore"
                />
                <ExampleSlide
                    id="home-example-3"
                    tab_id="home-example-tab-3"
                    active=false
                    image="/inspace/vendor/img/place-harbour-1080.webp"
                    kind_zh="公司空间"
                    kind_en="Company Space"
                    title_zh="一串地址，也可以成为一段被看见的来路。"
                    title_en="An address can hold the story of how a company arrived here."
                    body_zh="它在做什么，怎么进门，由谁接待，经历过什么，都能从这个地点开始被讲清楚。"
                    body_en="What it does, how to enter, who welcomes visitors, and what it has lived through can begin at this place."
                    action_zh="创建一个空间"
                    action_en="Create a Space"
                    href="/inspace/my-spaces"
                />
            </div>
            <div class="home-carousel-tabs" role="tablist" aria-label=move || match locale.get() { Locale::Zh => "切换空间示例", Locale::En => "Choose a space example" }>
                <button id="home-example-tab-0" type="button" role="tab" aria-controls="home-example-0" aria-selected="true" tabindex="0" data-home-carousel-dot data-home-carousel-index="0">{move || match locale.get() { Locale::Zh => "景点", Locale::En => "Landmark" }}</button>
                <button id="home-example-tab-1" type="button" role="tab" aria-controls="home-example-1" aria-selected="false" tabindex="-1" data-home-carousel-dot data-home-carousel-index="1">{move || match locale.get() { Locale::Zh => "美食", Locale::En => "Food" }}</button>
                <button id="home-example-tab-2" type="button" role="tab" aria-controls="home-example-2" aria-selected="false" tabindex="-1" data-home-carousel-dot data-home-carousel-index="2">{move || match locale.get() { Locale::Zh => "公园", Locale::En => "Park" }}</button>
                <button id="home-example-tab-3" type="button" role="tab" aria-controls="home-example-3" aria-selected="false" tabindex="-1" data-home-carousel-dot data-home-carousel-index="3">{move || match locale.get() { Locale::Zh => "公司", Locale::En => "Company" }}</button>
            </div>
        </section>
    }
}

#[component]
fn ExampleSlide(
    id: &'static str,
    tab_id: &'static str,
    active: bool,
    image: &'static str,
    kind_zh: &'static str,
    kind_en: &'static str,
    title_zh: &'static str,
    title_en: &'static str,
    body_zh: &'static str,
    body_en: &'static str,
    action_zh: &'static str,
    action_en: &'static str,
    href: &'static str,
) -> impl IntoView {
    let locale = use_i18n().locale;
    let class = if active {
        "home-example-slide is-active"
    } else {
        "home-example-slide"
    };
    let aria_hidden = if active { "false" } else { "true" };
    let link_tabindex = if active { "0" } else { "-1" };
    view! {
        <article id=id class=class role="tabpanel" aria-labelledby=tab_id data-home-carousel-slide aria-hidden=aria_hidden>
            <figure>
                <img src=image width="1080" height="720" loading="lazy" decoding="async" alt=move || match locale.get() { Locale::Zh => kind_zh, Locale::En => kind_en } />
            </figure>
            <div class="home-example-copy">
                <p>{move || match locale.get() { Locale::Zh => kind_zh, Locale::En => kind_en }}</p>
                <h3>{move || match locale.get() { Locale::Zh => title_zh, Locale::En => title_en }}</h3>
                <p>{move || match locale.get() { Locale::Zh => body_zh, Locale::En => body_en }}</p>
                <a href=href tabindex=link_tabindex>{move || match locale.get() { Locale::Zh => action_zh, Locale::En => action_en }}</a>
            </div>
        </article>
    }
}

#[component]
fn HomeFeaturedSpaces() -> impl IntoView {
    let locale = use_i18n().locale;
    let modal = use_space_experience_modal().expect("Space modal provider must exist");
    let spaces = Resource::new(
        || (),
        |_| async move { list_home_featured_spaces(6).await.unwrap_or_default() },
    );
    view! {
        <section class="home-featured-spaces" style="order:30" aria-labelledby="home-featured-spaces-title">
            <header class="home-content-head">
                <div>
                    <p class="survey-kicker"><span class="survey-kicker-mark" aria-hidden="true"></span>{move || match locale.get() { Locale::Zh => "正在被看见的地点", Locale::En => "Places in view" }}</p>
                    <h2 id="home-featured-spaces-title">{move || match locale.get() { Locale::Zh => "热门空间", Locale::En => "Featured Spaces" }}</h2>
                </div>
                <a href="/inspace/explore">{move || match locale.get() { Locale::Zh => "查看全部空间", Locale::En => "Browse all Spaces" }}</a>
            </header>
            <Suspense fallback=move || view! { <div class="home-space-skeleton" aria-label="Loading featured spaces"><span></span><span></span><span></span></div> }>
                {move || Suspend::new(async move {
                    let items = spaces.await;
                    view! { <FeaturedSpaceShelf items=items modal=modal /> }
                })}
            </Suspense>
        </section>
    }
}

#[component]
fn FeaturedSpaceShelf(items: Vec<SpaceMarker>, modal: SpaceExperienceModalState) -> impl IntoView {
    let locale = use_i18n().locale;
    if items.is_empty() {
        return view! {
            <div class="home-empty-note">
                <p>{move || match locale.get() {
                    Locale::Zh => "第一批地点正在整理中。你熟悉的地方，也可以从你开始。",
                    Locale::En => "The first places are being prepared. A place you know can begin with you.",
                }}</p>
                <a href="/inspace/my-spaces">{move || match locale.get() { Locale::Zh => "创建空间", Locale::En => "Create a Space" }}</a>
            </div>
        }.into_any();
    }
    let indexed_items = items.into_iter().enumerate().collect::<Vec<_>>();
    let has_attributed_artwork = indexed_items
        .iter()
        .any(|(_, space)| known_space_artwork(space).is_some());

    let total_items = indexed_items.len();
    let dots = (0..total_items)
        .map(|index| {
            let selected = if index == 0 { "true" } else { "false" };
            let tab_index = if index == 0 { "0" } else { "-1" };
            view! {
                <button
                    type="button"
                    data-home-space-dot
                    data-home-space-index=index
                    aria-current=selected
                    tabindex=tab_index
                    aria-label=move || match locale.get() {
                        Locale::Zh => format!("查看第 {} 个热门空间", index + 1),
                        Locale::En => format!("Show featured Space {}", index + 1),
                    }
                ></button>
            }
        })
        .collect_view();

    view! {
        <div
            class="home-space-carousel"
            data-home-space-carousel
            aria-roledescription="carousel"
            aria-label=move || match locale.get() {
                Locale::Zh => "热门空间轮展",
                Locale::En => "Featured Spaces carousel",
            }
        >
            <div
                class="home-space-stage"
                data-home-space-stage
                tabindex="0"
                aria-label=move || match locale.get() {
                    Locale::Zh => "热门空间：自动轮展，也可使用方向键、圆点或滑动切换",
                    Locale::En => "Featured Spaces: rotates automatically; use keyboard, dots, or swipe to browse",
                }
            >
                <For
                    each=move || indexed_items.clone()
                    key=|(index, space)| format!("{}-{}", index, space.id)
                    children=move |(index, space)| {
                        let artwork = space_artwork(&space);
                        let image = artwork.src;
                        let image_style = format!("object-position:{};", artwork.position);
                        let space_id = space.id.clone();
                        let title = space.name_zh.clone();
                        let aria_title = title.clone();
                        let location = space_location(&space);
                        let aria_location = location.clone();
                        let kind = space_type_label(&space.space_type);
                        let class = if index == 0 { "home-space-item is-active" } else { "home-space-item" };
                        view! {
                            <article class=class data-home-space-slide data-home-space-index=index>
                                <OpenSpaceLink space_id=space_id initial_panel=SpacePanel::Wall modal_state=modal require_login=true class="home-space-item-link" aria_label=format!("{} {}", aria_title, aria_location)>
                                    <div class="home-space-image-frame">
                                        <img src=image style=image_style width="1080" height="720" loading="lazy" decoding="async" alt="" />
                                    </div>
                                    <div class="home-space-item-copy">
                                        <p>{move || match locale.get() { Locale::Zh => kind.0, Locale::En => kind.1 }}</p>
                                        <h3>{title}</h3>
                                        <span>{location}</span>
                                    </div>
                                </OpenSpaceLink>
                            </article>
                        }
                    }
                />
            </div>
            <div class="home-space-carousel-nav">
                <div class="home-space-carousel-dots" role="group" aria-label=move || match locale.get() { Locale::Zh => "选择热门空间", Locale::En => "Choose a featured Space" }>
                    {dots}
                </div>
                <p class="home-space-carousel-count" aria-live="polite">
                    <span data-home-space-current>{"01"}</span>
                    <span aria-hidden="true">{" / "}</span>
                    <span>{format!("{:02}", total_items)}</span>
                </p>
            </div>
            <p class="home-space-carousel-hint">{move || match locale.get() {
                Locale::Zh => "无需移动鼠标：空间会自行轮展，也可点击、滑动或按方向键浏览。",
                Locale::En => "No mouse movement required: Spaces rotate automatically and also support click, swipe, and arrow keys.",
            }}</p>
        </div>
        {has_attributed_artwork.then(|| view! { <HomePhotoAttributions /> })}
    }.into_any()
}

#[component]
fn HomePhotoAttributions() -> impl IntoView {
    let locale = use_i18n().locale;
    view! {
        <details class="home-photo-attributions">
            <summary>{move || match locale.get() {
                Locale::Zh => "精选地点图片来源与许可",
                Locale::En => "Featured image credits and licenses",
            }}</summary>
            <ul>
                <li>
                    <a href="https://commons.wikimedia.org/wiki/File:The_Bund_in_Shanghai.jpg" target="_blank" rel="noreferrer">
                        "外滩：GillyBerlin，CC BY 2.0，Wikimedia Commons"
                    </a>
                </li>
                <li>
                    <a href="https://commons.wikimedia.org/wiki/File:Forbidden_city_06.jpg" target="_blank" rel="noreferrer">
                        "故宫：Jacob Ehnmark，CC BY 2.0，Wikimedia Commons"
                    </a>
                </li>
                <li>
                    <a href="https://commons.wikimedia.org/wiki/File:Potala_Palace_-_Lhasa,_Tibet.jpg" target="_blank" rel="noreferrer">
                        "布达拉宫：cattan2011，CC BY 2.0，Wikimedia Commons"
                    </a>
                </li>
                <li>
                    <a href="https://commons.wikimedia.org/wiki/File:China_Hangzhou_Westlake-2.jpg" target="_blank" rel="noreferrer">
                        "西湖：Jacob Ehnmark，CC BY 2.0，Wikimedia Commons"
                    </a>
                </li>
                <li>
                    <a href="https://commons.wikimedia.org/wiki/File:Bayi_Square,_Nanchang.jpg" target="_blank" rel="noreferrer">
                        "八一广场：钉钉，CC BY-SA 4.0，Wikimedia Commons"
                    </a>
                </li>
            </ul>
        </details>
    }
}

#[component]
fn HomeFeaturedStories() -> impl IntoView {
    let locale = use_i18n().locale;
    let modal = use_space_experience_modal().expect("Space modal provider must exist");
    let stories = Resource::new(
        || (),
        |_| async move { list_home_featured_stories(3).await.unwrap_or_default() },
    );
    view! {
        <section class="home-featured-stories" style="order:40" aria-labelledby="home-featured-stories-title">
            <header class="home-content-head">
                <div>
                    <h2 id="home-featured-stories-title">{move || match locale.get() { Locale::Zh => "热门故事", Locale::En => "Featured stories" }}</h2>
                    <p>{move || match locale.get() { Locale::Zh => "只放真实留下的话。没有合适的故事，就不替地点编一个。", Locale::En => "Only real words left at a place appear here. We do not invent stories for it." }}</p>
                </div>
            </header>
            <Suspense fallback=move || view! { <div class="home-story-skeleton"><span></span><span></span></div> }>
                {move || Suspend::new(async move {
                    let items = stories.await;
                    view! { <FeaturedStoryShelf items=items modal=modal /> }
                })}
            </Suspense>
        </section>
    }
}

#[component]
fn FeaturedStoryShelf(
    items: Vec<HomeStoryView>,
    modal: SpaceExperienceModalState,
) -> impl IntoView {
    let locale = use_i18n().locale;
    if items.is_empty() {
        return view! {
            <div class="home-story-empty">
                <p>{move || match locale.get() {
                    Locale::Zh => "这里暂时没有值得推荐的公开故事。第一句，留给真的到过那里的人。",
                    Locale::En => "There is no public story worth featuring yet. The first line belongs to someone who has really been there.",
                }}</p>
                <a href="/inspace/map">{move || match locale.get() { Locale::Zh => "去地图找一个地点", Locale::En => "Find a place on the map" }}</a>
            </div>
        }.into_any();
    }

    view! {
        <div class="home-story-shelf">
            <For
                each=move || items.clone()
                key=|story| story.id.clone()
                children=move |story| {
                    let space_id = story.space_id.clone();
                    let place = story.space_name_zh.clone();
                    let body = home_story_excerpt(&story.body, 82);
                    let author = story.author_name.clone();
                    let date = story.created_at.clone();
                    let city = story.city.clone().unwrap_or_default();
                    let proof = story.proof;
                    view! {
                        <article class="home-story-item">
                            <OpenSpaceLink space_id=space_id initial_panel=SpacePanel::Story modal_state=modal require_login=true class="home-story-item-link">
                                <blockquote>{body}</blockquote>
                                <footer>
                                    <strong>{place}</strong>
                                    <span>{move || format!("{}  {}  {}", city, proof.label(locale.get() == Locale::Zh), date)}</span>
                                    <span>{author}</span>
                                </footer>
                            </OpenSpaceLink>
                        </article>
                    }
                }
            />
        </div>
    }.into_any()
}

fn home_story_excerpt(body: &str, max_chars: usize) -> String {
    let normalized = body.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut chars = normalized.chars();
    let excerpt = chars.by_ref().take(max_chars).collect::<String>();
    if chars.next().is_some() {
        format!("{excerpt}……")
    } else {
        excerpt
    }
}

#[derive(Clone, Copy)]
struct SpaceArtwork {
    src: &'static str,
    position: &'static str,
}

fn known_space_artwork(space: &SpaceMarker) -> Option<SpaceArtwork> {
    match space.name_zh.trim() {
        "外滩" => Some(SpaceArtwork {
            src: "/inspace/vendor/img/featured/bund.webp",
            position: "50% 62%",
        }),
        "故宫" => Some(SpaceArtwork {
            src: "/inspace/vendor/img/featured/forbidden-city.webp",
            position: "50% 61%",
        }),
        "布达拉宫" => Some(SpaceArtwork {
            src: "/inspace/vendor/img/featured/potala.webp",
            position: "50% 65%",
        }),
        "西湖" => Some(SpaceArtwork {
            src: "/inspace/vendor/img/featured/west-lake.webp",
            position: "50% 56%",
        }),
        "八一广场" => Some(SpaceArtwork {
            src: "/inspace/vendor/img/featured/bayi-square.webp",
            position: "50% 59%",
        }),
        _ => None,
    }
}

fn space_artwork(space: &SpaceMarker) -> SpaceArtwork {
    known_space_artwork(space).unwrap_or_else(|| SpaceArtwork {
        src: space_type_image(&space.space_type),
        position: "50% 50%",
    })
}

fn space_type_image(space_type: &SpaceType) -> &'static str {
    match space_type {
        SpaceType::Scenic => "/inspace/vendor/img/place-lake-1080.webp",
        SpaceType::Food => "/inspace/vendor/img/place-alley-1080.webp",
        SpaceType::Park => "/inspace/vendor/img/place-lane-1080.webp",
        SpaceType::Transit => "/inspace/vendor/img/place-harbour-1080.webp",
        SpaceType::Event => "/inspace/vendor/img/place-canal-1080.webp",
        SpaceType::Custom => "/inspace/vendor/img/place-bund-1080.webp",
    }
}

fn space_type_label(space_type: &SpaceType) -> (&'static str, &'static str) {
    match space_type {
        SpaceType::Scenic => ("景点空间", "Landmark Space"),
        SpaceType::Food => ("美食空间", "Food Space"),
        SpaceType::Park => ("公园空间", "Park Space"),
        SpaceType::Transit => ("交通空间", "Transit Space"),
        SpaceType::Event => ("活动空间", "Event Space"),
        SpaceType::Custom => ("地点空间", "Place Space"),
    }
}

fn space_location(space: &SpaceMarker) -> String {
    [
        space.province.as_deref(),
        space.city.as_deref(),
        space.spot_name.as_deref(),
    ]
    .into_iter()
    .flatten()
    .filter(|value| !value.trim().is_empty())
    .collect::<Vec<_>>()
    .join(" / ")
}

fn localize(locale: Locale, value: &LocalizedText) -> String {
    match locale {
        Locale::Zh => value.zh.clone(),
        Locale::En => value.en.clone(),
    }
}
