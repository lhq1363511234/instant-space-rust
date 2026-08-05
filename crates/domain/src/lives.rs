//! Digital lives — the pets (and, one day, the people) that live with us.
//!
//! Brand line: 「万物有灵，皆有归处」. Sections: 在侧 (accompanying) /
//! 追远 (in memory, from 慎终追远). All copy follows Song-style restraint:
//! 白描、短句、留白、时节与地点意象、哀而不伤 (see the life-distill skill).

use serde::{Deserialize, Serialize};
use time::Date;
use uuid::Uuid;

/// Where a companion is right now.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompanionState {
    /// 在侧 — the owner has been active recently, so the companion travels
    /// with them and records footprints wherever the owner proves presence.
    Following,
    /// 在家 — the owner is idle, so the companion waits at the cloud home.
    AtHome,
    /// 追远 — the companion has died; the memorial space (digital_lives) rules.
    Memorial,
}

impl CompanionState {
    pub fn as_db(self) -> &'static str {
        match self {
            Self::Following => "following",
            Self::AtHome => "at_home",
            Self::Memorial => "memorial",
        }
    }

    pub fn from_db(value: &str) -> Self {
        match value {
            "following" => Self::Following,
            "memorial" => Self::Memorial,
            _ => Self::AtHome,
        }
    }

    pub fn label(self, zh: bool) -> &'static str {
        match (self, zh) {
            (Self::Following, true) => "在侧",
            (Self::AtHome, true) => "在家",
            (Self::Memorial, true) => "追远",
            (Self::Following, false) => "Following",
            (Self::AtHome, false) => "At home",
            (Self::Memorial, false) => "In memory",
        }
    }
}

/// What a visitor leaves at a memorial.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrayerKind {
    /// 一炷香
    Incense,
    /// 一枝花
    Flower,
    /// 一盏灯
    Lantern,
    /// 留字
    Word,
}

impl PrayerKind {
    pub fn as_db(self) -> &'static str {
        match self {
            Self::Incense => "incense",
            Self::Flower => "flower",
            Self::Lantern => "lantern",
            Self::Word => "word",
        }
    }

    pub fn from_db(value: &str) -> Self {
        match value {
            "flower" => Self::Flower,
            "lantern" => Self::Lantern,
            "word" => Self::Word,
            _ => Self::Incense,
        }
    }

    pub fn label(self, zh: bool) -> &'static str {
        match (self, zh) {
            (Self::Incense, true) => "一炷香",
            (Self::Flower, true) => "一枝花",
            (Self::Lantern, true) => "一盏灯",
            (Self::Word, true) => "留字",
            (Self::Incense, false) => "Incense",
            (Self::Flower, false) => "Flower",
            (Self::Lantern, false) => "Lantern",
            (Self::Word, false) => "Words",
        }
    }
}

/// The cloud home every user owns. Friends may visit, but entering requires
/// the home passphrase — a door key, not a Wi-Fi join.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudHome {
    pub id: Uuid,
    /// The canonical private Space represented by this home.
    pub space_id: Uuid,
    pub owner_id: Uuid,
    pub owner_name: Option<String>,
    pub name: String,
    pub motto: Option<String>,
    pub has_passphrase: bool,
    pub door_note: Option<String>,
    pub created_at: time::OffsetDateTime,
}

/// A family member: pet today, human tomorrow. The model is deliberately
/// neutral so the same rows can be upgraded via subject_type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Companion {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub home_id: Uuid,
    pub subject_type: String,
    pub name: String,
    pub species: Option<String>,
    pub breed: Option<String>,
    pub gender: Option<String>,
    pub birth_at: Option<Date>,
    pub death_at: Option<Date>,
    pub state: CompanionState,
    pub avatar_url: Option<String>,
    pub trail_count: i64,
    pub created_at: time::OffsetDateTime,
}

/// One footprint: the owner proved presence somewhere, so the companion was
/// "there too". Optional handwritten snippets feed the later distillation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanionTrail {
    pub id: Uuid,
    pub companion_id: Uuid,
    pub space_id: Option<Uuid>,
    pub space_name: Option<String>,
    pub place_name: Option<String>,
    pub proof: String,
    pub noted_at: time::OffsetDateTime,
    pub snippet: Option<String>,
    pub season_hint: Option<String>,
}

/// A free chapter of the 小传.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiographyChapter {
    pub title: String,
    pub body: String,
}

/// One line of the 生命地图: place × season × one deed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifeMapEntry {
    pub place: String,
    pub season: Option<String>,
    pub deed: String,
}

/// The distilled memorial, created only after death. Versioned so a rewrite
/// of the distillation can always be traced back.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigitalLife {
    pub id: Uuid,
    pub companion_id: Uuid,
    pub owner_id: Uuid,
    pub subject_type: String,
    pub name: String,
    pub epitaph: String,
    pub biography: Vec<BiographyChapter>,
    pub inscription: String,
    pub life_map: Vec<LifeMapEntry>,
    pub memorial_date: Option<Date>,
    pub incense_count: i64,
    pub visitor_count: i64,
    pub distill_version: i32,
    pub content_version: i32,
    pub published: bool,
    pub created_at: time::OffsetDateTime,
}

/// What a visitor left at a memorial: incense, flower, lantern, or words.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifePrayer {
    pub id: Uuid,
    pub life_id: Uuid,
    pub visitor_name: String,
    pub kind: PrayerKind,
    pub message: Option<String>,
    pub created_at: time::OffsetDateTime,
}

/// Paginated directory of memorials (追远).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedLives {
    pub items: Vec<DigitalLife>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}
