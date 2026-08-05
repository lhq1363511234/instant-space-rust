use leptos::prelude::*;
use leptos_meta::{Meta, Title};

use crate::i18n::{t, use_i18n, Locale};

#[component]
pub fn AboutPage() -> impl IntoView {
    let locale = use_i18n().locale;

    view! {
        <Title text=move || t(locale.get(), "关于 inspace｜把人带回真实现场", "About inspace | Bringing people back to the place itself") />
        <Meta
            name="description"
            content=move || t(
                locale.get(),
                "inspace 想把人从手机屏幕带回真实地点：到了现场再打开空间，读攻略、问在场的人、留下记忆。了解我们为什么这么做，以及如何成为正在招募的空间主理人。",
                "inspace wants to bring people off the screen and back to real places: arrive first, then open the Space to read its guide, ask people there, and leave a memory. Learn why, and how to become a Space host we are recruiting.",
            )
        />
        <main id="main-content" class="page about-inspace">
            <header class="about-hero">
                <p class="survey-kicker">{move || t(locale.get(), "关于 inspace", "About inspace")}</p>
                <h1>{move || t(locale.get(), "别让世界，\n只剩屏幕上的一个点。", "Don't let the world shrink\ninto a dot on your screen.")}</h1>
                <p>{move || t(
                    locale.get(),
                    "inspace 希望你先抵达，再打开地点的空间：看实地记录，遇见在场的人，也留下一点属于这里的东西。",
                    "Arrive first, then open the place: read field notes, meet people who are there, and leave something that belongs to it.",
                )}</p>
                <div class="about-hero-actions">
                    <a class="button button-primary" href="/inspace/explore">{move || t(locale.get(), "看看真实地点的空间", "Explore Spaces at real places")}</a>
                    <a class="button button-quiet" href="/inspace/my-spaces">{move || t(locale.get(), "为熟悉的地点建空间", "Create a Space for a place you know")}</a>
                </div>
            </header>

            <figure class="about-place-window">
                <img src="/inspace/vendor/img/place-lane-1080.webp" width="1080" height="720" loading="eager" decoding="async" alt=move || t(locale.get(), "真实街巷与远处城市天际线", "A real lane looking toward the city skyline") />
                <figcaption>{move || t(locale.get(), "地图负责到达。真正的地点，从到场以后开始。", "The map gets you there. The place begins once you arrive.")}</figcaption>
            </figure>

            <section class="about-definition" aria-labelledby="about-definition-title">
                <div>
                    <p class="about-section-kicker">{move || t(locale.get(), "为什么做", "Why it exists")}</p>
                    <h2 id="about-definition-title">{move || t(locale.get(), "到达以后，才是地点真正发生的地方。", "A place begins after arrival.")}</h2>
                </div>
                <div class="about-prose">
                    <p>{move || t(
                        locale.get(),
                        "地图解决到达，inspace 接住到达之后的体验。攻略、故事、讨论和胶囊，都围绕一个真实地点慢慢生长。",
                        "Maps solve arrival. inspace holds what comes next: field notes, stories, discussion, and capsules that grow around a real place.",
                    )}</p>
                    <dl>
                        <div><dt>{move || t(locale.get(), "抬头", "Look up")}</dt><dd>{move || t(locale.get(), "先到现场，别停在屏幕里", "Arrive first, don't stay in the screen")}</dd></div>
                        <div><dt>{move || t(locale.get(), "进入", "Enter")}</dt><dd>{move || t(locale.get(), "在场再打开：攻略、现场、讨论", "Present, then open: guide, presence, discussion")}</dd></div>
                        <div><dt>{move || t(locale.get(), "留下", "Leave")}</dt><dd>{move || t(locale.get(), "故事、留痕、时间胶囊", "Stories, traces, capsules")}</dd></div>
                    </dl>
                </div>
            </section>

            <section id="hosts" class="about-host-call" aria-labelledby="about-host-title">
                <div class="about-host-heading">
                    <p class="about-section-kicker">{move || t(locale.get(), "空间主理人", "Space hosts")}</p>
                    <p class="about-open-label">{move || t(locale.get(), "正在招募", "Now recruiting")}</p>
                    <h2 id="about-host-title">{move || t(locale.get(), "空间需要主理人，\n不是更多管理员。", "Spaces need local hosts,\nnot more administrators.")}</h2>
                </div>
                <div class="about-host-body">
                    <p class="about-host-lede">{move || t(
                        locale.get(),
                        "空间真正可信，靠的是熟悉这里、愿意长期维护它的人。",
                        "A Space becomes trustworthy through someone who knows the place and keeps it over time.",
                    )}</p>
                    <ul class="about-host-duties">
                        <li><div><strong>{move || t(locale.get(), "整理真实攻略", "Keep a field-tested guide")}</strong><p>{move || t(locale.get(), "路线、时段与避坑。", "Routes, timing, and pitfalls.")}</p></div></li>
                        <li><div><strong>{move || t(locale.get(), "回答现场问题", "Answer on-site questions")}</strong><p>{move || t(locale.get(), "告诉别人今天的变化。", "Share what changed today.")}</p></div></li>
                        <li><div><strong>{move || t(locale.get(), "保留地方故事", "Keep the place's stories")}</strong><p>{move || t(locale.get(), "留下值得保存的记忆。", "Keep memories worth saving.")}</p></div></li>
                    </ul>
                    <div class="about-actions">
                        <a class="button button-primary" href="/inspace/my-spaces">{move || t(locale.get(), "成为空间主理人", "Become a Space host")}</a>
                        <a class="button button-secondary" href="/inspace/explore">{move || t(locale.get(), "看看已经点亮的空间", "See Spaces already lit")}</a>
                    </div>
                    <p class="about-host-note">{move || t(
                        locale.get(),
                        "你可以先为熟悉的地点创建空间；预先点亮的空间会逐步开放认领。",
                        "Create a Space for a place you know. Seeded Spaces will open for claims over time.",
                    )}</p>
                </div>
            </section>

            <section class="about-founder" aria-labelledby="about-founder-title">
                <div>
                    <p class="about-section-kicker">{move || t(locale.get(), "创始人寄语", "Founder note")}</p>
                    <h2 id="about-founder-title">{move || t(locale.get(), "为什么还要为地点做一层空间", "Why places need another layer")}</h2>
                </div>
                <blockquote>
                    <p>{move || match locale.get() {
                        Locale::Zh => "我们不想让人留在屏幕里。inspace 想让人走进现实，在一个地方亲眼看见、真的遇见，也留下值得后来人读到的东西。",
                        Locale::En => "inspace is made to move people into the real world: to see a place, meet there, and leave something worth finding later.",
                    }}</p>
                    <footer><strong>"inspace"</strong><span>"Be IN the space, beyond the map."</span></footer>
                </blockquote>
            </section>
        </main>
    }
}
