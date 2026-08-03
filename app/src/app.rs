use leptos::prelude::*;
#[cfg(feature = "ssr")]
use leptos_meta::MetaTags;
use leptos_meta::{provide_meta_context, Html, Link, Meta, Script, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

use crate::components::{
    header::Header,
    map_workspace::MapWorkspace,
    space_form::{provide_create_space_modal, CreateSpaceModalHost},
};
use crate::feedback::{provide_feedback, FeedbackToasts};
use crate::pages::{
    about::AboutPage,
    admin::AdminRoutes,
    admin_claims::AdminClaimsPage,
    admin_guides::AdminGuidesPage,
    admin_home::AdminHomePage,
    admin_residents::AdminResidentsPage,
    admin_spaces::AdminSpacesPage,
    admin_users::AdminUsersPage,
    auth::LoginPage,
    explore::ExplorePage,
    guides::{GuideDetailPage, GuideEditorPage, GuidesPage},
    home::HomePage,
    host::HostRoutes,
    lives::{CloudHomePage, LivesPage, MemorialPage},
    space::{SpaceChatPage, SpacePage},
};

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    let i18n = crate::i18n::provide_i18n();
    let locale = i18n.locale;
    crate::app_state::provide_app_refresh_state();
    provide_create_space_modal();
    #[allow(unused_variables)]
    let feedback = provide_feedback();

    view! {
        <Html {..} lang=move || locale.get().code() attr:data-locale=move || locale.get().code() />
        <Title text="inspace｜走到导航的尽头，体验才开始" />
        <Meta name="description" content="地图带你到达，inspace 让你真正进入真实地点：看攻略、了解现场、分享二维码，并与在场的人连接。" />
        <Meta name="keywords" content="inspace,旅行攻略,空间地图,真实地点,介观空间,Travel guide,travel map,destination guide" />
        <Meta name="robots" content="index,follow,max-image-preview:large,max-snippet:-1,max-video-preview:-1" />
        <Meta name="googlebot" content="index,follow" />
        <Meta name="author" content="inspace" />
        <Meta name="theme-color" content="#0f172a" />
        <Meta property="og:type" content="website" />
        <Meta property="og:site_name" content="inspace" />
        <Meta property="og:title" content="inspace | Beyond the map" />
        <Meta property="og:description" content="在地图上发现真实旅行地点，进入空间看攻略、社群入口和二维码分享。Discover travel guide spaces for real places on a world map." />
        <Meta property="og:url" content="https://opctoai.com/inspace" />
        <Meta property="og:locale" content="zh_CN" />
        <Meta property="og:locale:alternate" content="en_US" />
        <Meta name="twitter:card" content="summary_large_image" />
        <Meta name="twitter:title" content="inspace | Beyond the map" />
        <Meta name="twitter:description" content="地图发现旅行空间 · 攻略 · 社群入口 · 二维码分享" />
        <Meta name="geo.region" content="CN" />
        <Meta name="geo.placename" content="Global travel destinations" />
        <Meta name="ICBM" content="31.2397, 121.4998" />
        <Link rel="canonical" href="https://opctoai.com/inspace" />
        <Link rel="alternate" href="https://opctoai.com/inspace" hreflang="zh-CN" />
        <Link rel="alternate" href="https://opctoai.com/inspace" hreflang="en" />
        <Link rel="alternate" href="https://opctoai.com/inspace" hreflang="x-default" />
        <Script type_="application/ld+json">
            r#"{
  "@context": "https://schema.org",
  "@graph": [
    {
      "@type": "WebSite",
      "@id": "https://opctoai.com/inspace#website",
      "url": "https://opctoai.com/inspace",
      "name": "inspace",
      "alternateName": ["inspace", "即时空间", "InSpaceOS"],
      "description": "Global travel guide spaces on a live map. Discover destinations, enter shared spaces, read guides, and chat in real time.",
      "inLanguage": ["zh-CN", "en"],
      "potentialAction": {
        "@type": "SearchAction",
        "target": "https://opctoai.com/inspace?q={search_term_string}",
        "query-input": "required name=search_term_string"
      }
    },
    {
      "@type": "Organization",
      "@id": "https://opctoai.com/inspace#organization",
      "name": "inspace",
      "url": "https://opctoai.com/inspace",
      "description": "Travel guide space platform for real-world destinations."
    },
    {
      "@type": "WebApplication",
      "name": "inspace",
      "url": "https://opctoai.com/inspace",
      "applicationCategory": "TravelApplication",
      "operatingSystem": "Web",
      "offers": { "@type": "Offer", "price": "0", "priceCurrency": "USD" },
      "description": "Map-first travel guide spaces: discover places, enter public or private rooms, read guides, and share QR/link entry."
    }
  ]
}"#
        </Script>
        <Stylesheet id="fonts-css" href="/style/fonts.css?v=20260729-craft-v17" />
        <Stylesheet id="main-css" href="/style/main.css?v=20260729-craft-v17" />
        <Stylesheet id="ui-system-css" href="/style/ui-system.css?v=20260729-craft-v17" />
        <Stylesheet id="app-shell-css" href="/style/app-shell.css?v=20260731-shell-feedback-v5" />
        <Stylesheet id="workspace-css" href="/style/workspace.css?v=20260731-members-v18" />
        <Stylesheet id="backoffice-css" href="/style/backoffice.css?v=20260727-editor-v5" />
        <Stylesheet id="world-css" href="/style/inspace-world.css?v=20260728-hyperframes-taste-v3" />
        <Stylesheet id="song-css" href="/style/song-system.css?v=20260730-song-colour-v13" />
        <Stylesheet id="about-css" href="/style/about.css?v=20260729-about-v5" />
        <Stylesheet id="directory-css" href="/style/directory-system.css?v=20260730-directory-v4" />
        <Stylesheet id="space-experience-css" href="/style/space-experience.css?v=20260729-space-experience-v2" />
        <Stylesheet id="home-reframe-css" href="/style/home-reframe.css?v=20260729-home-restore-record-v1" />
        <Stylesheet id="home-discovery-css" href="/style/home-discovery.css?v=20260730-home-discovery-v10" />
        <Stylesheet id="home-song-css" href="/style/home-song.css?v=20260803-song-kv-v1" />
        <Stylesheet id="admin-operations-css" href="/style/admin-operations.css?v=20260729-home-featured-v1" />
        <Stylesheet id="lives-css" href="/style/lives.css?v=20260731-lives-v2" />
        <Router>
            <Header />
            <div class="app-main">
            <Routes fallback=|| view! { <main id="main-content" class="page"><h1>"Not found"</h1></main> }>
                <Route path=path!("/") view=HomePage />
                <Route path=path!("/inspace") view=HomePage />
                <Route path=path!("/about") view=AboutPage />
                <Route path=path!("/inspace/about") view=AboutPage />
                <Route path=path!("/explore") view=ExplorePage />
                <Route path=path!("/inspace/explore") view=ExplorePage />
                <Route path=path!("/map") view=MapWorkspace />
                <Route path=path!("/inspace/map") view=MapWorkspace />
                <Route path=path!("/login") view=LoginPage />
                <Route path=path!("/inspace/login") view=LoginPage />
                <Route path=path!("/my-spaces") view=HostRoutes />
                <Route path=path!("/inspace/my-spaces") view=HostRoutes />
                <Route path=path!("/spaces/:space_id") view=SpacePage />
                <Route path=path!("/inspace/spaces/:space_id") view=SpacePage />
                <Route path=path!("/spaces/:space_id/chat") view=SpaceChatPage />
                <Route path=path!("/inspace/spaces/:space_id/chat") view=SpaceChatPage />
                <Route path=path!("/guides") view=GuidesPage />
                <Route path=path!("/inspace/guides") view=GuidesPage />
                <Route path=path!("/guides/new") view=GuideEditorPage />
                <Route path=path!("/inspace/guides/new") view=GuideEditorPage />
                <Route path=path!("/admin/guides/new") view=GuideEditorPage />
                <Route path=path!("/inspace/admin/guides/new") view=GuideEditorPage />
                <Route path=path!("/guides/:guide_id/edit") view=GuideEditorPage />
                <Route path=path!("/inspace/guides/:guide_id/edit") view=GuideEditorPage />
                <Route path=path!("/admin/guides/:guide_id/edit") view=GuideEditorPage />
                <Route path=path!("/inspace/admin/guides/:guide_id/edit") view=GuideEditorPage />
                <Route path=path!("/guides/:guide_id") view=GuideDetailPage />
                <Route path=path!("/inspace/guides/:guide_id") view=GuideDetailPage />
                <Route path=path!("/lives") view=LivesPage />
                <Route path=path!("/inspace/lives") view=LivesPage />
                <Route path=path!("/homes/:home_id") view=CloudHomePage />
                <Route path=path!("/inspace/homes/:home_id") view=CloudHomePage />
                <Route path=path!("/lives/:life_id") view=MemorialPage />
                <Route path=path!("/inspace/lives/:life_id") view=MemorialPage />
                <Route path=path!("/admin") view=AdminRoutes />
                <Route path=path!("/inspace/admin") view=AdminRoutes />
                <Route path=path!("/admin/home") view=AdminHomePage />
                <Route path=path!("/inspace/admin/home") view=AdminHomePage />
                <Route path=path!("/admin/spaces") view=AdminSpacesPage />
                <Route path=path!("/inspace/admin/spaces") view=AdminSpacesPage />
                <Route path=path!("/admin/guides") view=AdminGuidesPage />
                <Route path=path!("/inspace/admin/guides") view=AdminGuidesPage />
                <Route path=path!("/admin/resident-applications") view=AdminResidentsPage />
                <Route path=path!("/inspace/admin/resident-applications") view=AdminResidentsPage />
                <Route path=path!("/admin/host-claims") view=AdminClaimsPage />
                <Route path=path!("/inspace/admin/host-claims") view=AdminClaimsPage />
                <Route path=path!("/admin/users") view=AdminUsersPage />
                <Route path=path!("/inspace/admin/users") view=AdminUsersPage />
            </Routes>
                        <FeedbackToasts />
</div>
            <CreateSpaceModalHost />
        </Router>
    }
}

#[cfg(feature = "ssr")]
pub fn shell(options: LeptosOptions) -> impl IntoView {
    // MapLibre is loaded only by the dedicated /map workspace.
    let map_boot = include_str!("map_boot.js");
    let capitals_boot = include_str!("geo_capitals_boot.js");
    let chat_realtime = include_str!("chat_realtime.js");
    let home_hyperframes = include_str!("home_hyperframes.js");

    view! {
        <!DOCTYPE html>
        <html data-instant-ssr="leptos">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <script>{capitals_boot}</script>
                <script>{map_boot}</script>
                <AutoReload options=options.clone() />
                <HydrationScripts options=options.clone() />
                <script>{chat_realtime}</script>
                <script>{home_hyperframes}</script>
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}
