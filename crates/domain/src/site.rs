use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct LocalizedText {
    pub zh: String,
    pub en: String,
}

impl LocalizedText {
    pub fn new(zh: impl Into<String>, en: impl Into<String>) -> Self {
        Self {
            zh: zh.into(),
            en: en.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct HomeThemeConfig {
    pub preset: String,
    pub primary: String,
    pub deep: String,
    pub background: String,
    pub density: String,
    pub hero_layout: String,
}

impl Default for HomeThemeConfig {
    fn default() -> Self {
        Self {
            preset: "sky-ocean".to_string(),
            primary: "#238EE8".to_string(),
            deep: "#061A2B".to_string(),
            background: "#EDF6FC".to_string(),
            density: "comfortable".to_string(),
            hero_layout: "split".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct HomeNavConfig {
    pub home: LocalizedText,
    pub map: LocalizedText,
    pub guides: LocalizedText,
    pub my_spaces: LocalizedText,
}

impl Default for HomeNavConfig {
    fn default() -> Self {
        Self {
            home: LocalizedText::new("首页", "Home"),
            map: LocalizedText::new("空间地图", "Space map"),
            guides: LocalizedText::new("空间攻略", "Guides"),
            my_spaces: LocalizedText::new("我的空间", "My spaces"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct HomeSeoConfig {
    pub title: LocalizedText,
    pub description: LocalizedText,
}

impl Default for HomeSeoConfig {
    fn default() -> Self {
        Self {
            title: LocalizedText::new(
                "inspace｜到达之后，进入真实地点的攻略与在线空间",
                "inspace | Guides and live rooms for real places",
            ),
            description: LocalizedText::new(
                "地图带你到达，空间攻略帮助你了解这里，在线空间让来到这里的人彼此帮助。",
                "Maps get you there. Guides explain the place. Live rooms help people there support one another.",
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct HomeHeroConfig {
    pub visible: bool,
    pub order: i32,
    pub eyebrow: LocalizedText,
    pub title: LocalizedText,
    pub body: LocalizedText,
    pub note: LocalizedText,
    pub sample_location: LocalizedText,
    pub sample_title: LocalizedText,
    pub sample_body: LocalizedText,
    pub sample_guide_label: LocalizedText,
    pub sample_question: LocalizedText,
    pub sample_presence: LocalizedText,
    pub primary_label: LocalizedText,
    pub primary_url: String,
    pub secondary_label: LocalizedText,
    pub secondary_url: String,
}

impl Default for HomeHeroConfig {
    fn default() -> Self {
        Self {
            visible: true,
            order: 10,
            eyebrow: LocalizedText::new(
                "关于每一个真实地点",
                "About every real place",
            ),
            title: LocalizedText::new(
                "我们不生产故事，我们只是地点故事的搬运工。",
                "We do not make the stories. We just carry a place's stories to you.",
            ),
            body: LocalizedText::new(
                "每一个真实的地方，都有人来过、走过、留下过。我们把这些收下来，等你到了，推门进去，交给你。",
                "Every real place has people who came, walked it, and left something behind. We keep it here, so that when you arrive and step in, it is waiting for you.",
            ),
            note: LocalizedText::new("Be IN the space, beyond the map.", "Be IN the space, beyond the map."),
            sample_location: LocalizedText::new("上海 · 黄浦区 · 外滩", "Shanghai · Huangpu · The Bund"),
            sample_title: LocalizedText::new("今晚去外滩，走哪条路人少？", "The Bund tonight, which way is less crowded?"),
            sample_body: LocalizedText::new("这一条被空间里的人反复修订过：路线、上桥口、退场时间。", "Revised again and again by people who were there: the route, the ramp, when to leave."),
            sample_guide_label: LocalizedText::new("外滩夜景与人流", "Night views and crowds"),
            sample_question: LocalizedText::new("南京东路站哪个出口离江边最近？", "Which exit at East Nanjing Rd is closest to the river?"),
            sample_presence: LocalizedText::new("12 人在场 · 3 条现场更新", "12 people here · 3 live updates"),
            primary_label: LocalizedText::new("找一个地方", "Find a place"),
            primary_url: "/inspace/explore".to_string(),
            secondary_label: LocalizedText::new("翻阅攻略", "Read the guides"),
            secondary_url: "/inspace/guides".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct HomeJourneyConfig {
    pub visible: bool,
    pub order: i32,
    pub eyebrow: LocalizedText,
    pub title: LocalizedText,
    pub body: LocalizedText,
    pub arrive_title: LocalizedText,
    pub arrive_body: LocalizedText,
    pub guide_title: LocalizedText,
    pub guide_body: LocalizedText,
    pub help_title: LocalizedText,
    pub help_body: LocalizedText,
}

impl Default for HomeJourneyConfig {
    fn default() -> Self {
        Self {
            visible: true,
            order: 20,
            eyebrow: LocalizedText::new("从到达到互助", "From arrival to mutual help"),
            title: LocalizedText::new(
                "到了以后，真正有用的只有三件事",
                "Once you arrive, only three things matter",
            ),
            body: LocalizedText::new(
                "不是介绍，不是评分，是有人替你先走过一遍，并且还在这里。",
                "Not descriptions, not ratings, someone walked it first, and is still here.",
            ),
            arrive_title: LocalizedText::new("到达", "Arrive"),
            arrive_body: LocalizedText::new("地图的任务到此结束。", "This is where the map stops."),
            guide_title: LocalizedText::new("看懂", "Understand"),
            guide_body: LocalizedText::new(
                "路线、时段、避坑，写成这个地方的一份长期档案。",
                "Routes, timing, pitfalls, kept as one long-running record of the place.",
            ),
            help_title: LocalizedText::new("问人", "Ask"),
            help_body: LocalizedText::new(
                "档案没写的，问此刻站在那里的人。",
                "What the record misses, ask whoever is standing there.",
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct HomeGuideConfig {
    pub visible: bool,
    pub order: i32,
    pub eyebrow: LocalizedText,
    pub title: LocalizedText,
    pub body: LocalizedText,
    pub visual_route: LocalizedText,
    pub visual_warning: LocalizedText,
    pub visual_live: LocalizedText,
    pub cta_label: LocalizedText,
    pub cta_url: String,
}

impl Default for HomeGuideConfig {
    fn default() -> Self {
        Self {
            visible: true,
            order: 30,
            eyebrow: LocalizedText::new("一个地方的档案", "The record of a place"),
            title: LocalizedText::new(
                "经验写下来，才不会每个人都重踩一遍。",
                "Written down once, so nobody has to learn it the hard way again.",
            ),
            body: LocalizedText::new(
                "每个空间有一份持续修订的攻略。讨论里有价值的答案会被收回攻略，下一个来的人一进门就能读到。",
                "Every space keeps one guide that is continuously revised. Useful answers from the room are folded back in, so the next person reads them on arrival.",
            ),
            visual_route: LocalizedText::new("南京东路站 2 号口出，沿滇池路步行 8 分钟", "Exit 2 at East Nanjing Rd, 8 minutes along Dianchi Rd"),
            visual_warning: LocalizedText::new("周五、周六 19:30 后观景平台限流", "Platform access is capped after 19:30 on Fri and Sat"),
            visual_live: LocalizedText::new("今晚南段围挡施工，改走北侧台阶", "Barriers on the south stretch tonight, use the north steps"),
            cta_label: LocalizedText::new("翻阅空间攻略", "Read space guides"),
            cta_url: "/inspace/guides".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct HomeHostConfig {
    pub visible: bool,
    pub order: i32,
    pub title: LocalizedText,
    pub body: LocalizedText,
    pub cta_label: LocalizedText,
}

impl Default for HomeHostConfig {
    fn default() -> Self {
        Self {
            visible: true,
            order: 40,
            title: LocalizedText::new(
                "为熟悉的地点，建一个空间。",
                "Create a Space for a place you know.",
            ),
            body: LocalizedText::new(
                "我们正在招募空间主理人，让真实地点有人长期维护。",
                "We are recruiting Space hosts to keep real places maintained over time.",
            ),
            cta_label: LocalizedText::new("成为空间主理人", "Become a Space host"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct HomePageConfig {
    pub theme: HomeThemeConfig,
    pub nav: HomeNavConfig,
    pub seo: HomeSeoConfig,
    pub hero: HomeHeroConfig,
    pub journey: HomeJourneyConfig,
    pub guide: HomeGuideConfig,
    pub host: HomeHostConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HomePageAdminState {
    pub draft: HomePageConfig,
    pub published: HomePageConfig,
    pub published_version: i32,
    pub updated_at: Option<String>,
}

impl Default for HomePageAdminState {
    fn default() -> Self {
        let config = HomePageConfig::default();
        Self {
            draft: config.clone(),
            published: config,
            published_version: 0,
            updated_at: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SitePageVersion {
    pub id: Uuid,
    pub version: i32,
    pub actor_email: Option<String>,
    pub created_at: String,
}
