//! Phase 9 world foundation.
//!
//! A [`Space`] remains the durable identity of a real place. A [`Scene`] is
//! the visual environment people enter, while [`SceneObject`] turns content
//! into buildings, people and objects. Portals preserve exploratory travel;
//! teleport entry preserves direct arrival from maps, search, QR and links.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

macro_rules! string_enum {
    ($name:ident { $($variant:ident => $value:literal),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub enum $name { $($variant),+ }

        impl $name {
            pub fn as_db(self) -> &'static str {
                match self { $(Self::$variant => $value),+ }
            }

            pub fn from_db(value: &str) -> Self {
                match value { $($value => Self::$variant,)+ _ => Self::default() }
            }
        }
    };
}

string_enum!(SpaceRole {
    Hub => "hub",
    Place => "place",
    Micro => "micro",
    Home => "home",
    Memorial => "memorial",
});
impl Default for SpaceRole {
    fn default() -> Self {
        Self::Place
    }
}

string_enum!(SceneKind {
    Hub => "hub",
    Place => "place",
    Home => "home",
    Memorial => "memorial",
    Interior => "interior",
});
impl Default for SceneKind {
    fn default() -> Self {
        Self::Place
    }
}

string_enum!(SceneStatus {
    Draft => "draft",
    Published => "published",
    Archived => "archived",
});
impl Default for SceneStatus {
    fn default() -> Self {
        Self::Draft
    }
}

string_enum!(SceneObjectKind {
    TouristCenter => "tourist_center",
    AiGuide => "ai_guide",
    MessageWall => "message_wall",
    NoticeBoard => "notice_board",
    Host => "host",
    Portal => "portal",
    Capsule => "capsule",
    Display => "display",
    Building => "building",
    Decoration => "decoration",
});
impl Default for SceneObjectKind {
    fn default() -> Self {
        Self::Display
    }
}

string_enum!(RelationKind {
    Parent => "parent",
    Child => "child",
    Related => "related",
    Portal => "portal",
    HomeOf => "home_of",
    MemorialOf => "memorial_of",
});
impl Default for RelationKind {
    fn default() -> Self {
        Self::Related
    }
}

string_enum!(EntryMethod {
    Direct => "direct",
    Search => "search",
    Map => "map",
    Link => "link",
    Qr => "qr",
    Nfc => "nfc",
    Wifi => "wifi",
    Ai => "ai",
    Portal => "portal",
    Capsule => "capsule",
    History => "history",
    Home => "home",
});
impl Default for EntryMethod {
    fn default() -> Self {
        Self::Direct
    }
}

string_enum!(HostTenureRole {
    Primary => "primary",
    CoHost => "co_host",
    Steward => "steward",
});
impl Default for HostTenureRole {
    fn default() -> Self {
        Self::Primary
    }
}

string_enum!(HostTenureStatus {
    Active => "active",
    Ended => "ended",
    Revoked => "revoked",
});
impl Default for HostTenureStatus {
    fn default() -> Self {
        Self::Active
    }
}

string_enum!(PresenceSubjectKind {
    User => "user",
    Companion => "companion",
});
impl Default for PresenceSubjectKind {
    fn default() -> Self {
        Self::User
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scene {
    pub id: Uuid,
    pub space_id: Uuid,
    pub slug: String,
    pub kind: SceneKind,
    pub name_zh: String,
    pub name_en: Option<String>,
    pub description_zh: Option<String>,
    pub description_en: Option<String>,
    pub layout: Value,
    pub is_default: bool,
    pub status: SceneStatus,
    pub version: i32,
    pub created_by: Option<Uuid>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneObject {
    pub id: Uuid,
    pub scene_id: Uuid,
    pub kind: SceneObjectKind,
    pub name_zh: String,
    pub name_en: Option<String>,
    /// Percentage coordinates in the authored scene (0–100).
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub z_index: i32,
    pub interaction_radius: f64,
    pub content_kind: Option<String>,
    pub content_id: Option<Uuid>,
    pub target_space_id: Option<Uuid>,
    pub target_scene_id: Option<Uuid>,
    pub target_spawn_key: Option<String>,
    pub config: Value,
    pub status: SceneStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneSpawnPoint {
    pub id: Uuid,
    pub scene_id: Uuid,
    pub key: String,
    pub name_zh: String,
    pub name_en: Option<String>,
    pub x: f64,
    pub y: f64,
    pub facing: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneBundle {
    pub space_id: Uuid,
    pub space_name_zh: String,
    pub space_name_en: Option<String>,
    pub space_role: SpaceRole,
    pub is_public: bool,
    pub scene: Scene,
    pub objects: Vec<SceneObject>,
    pub spawn_points: Vec<SceneSpawnPoint>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpaceRelation {
    pub id: Uuid,
    pub source_space_id: Uuid,
    pub target_space_id: Uuid,
    pub kind: RelationKind,
    pub label_zh: Option<String>,
    pub label_en: Option<String>,
    pub metadata: Value,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpaceHostTenure {
    pub id: Uuid,
    pub space_id: Uuid,
    pub user_id: Uuid,
    pub role: HostTenureRole,
    pub status: HostTenureStatus,
    pub started_at: OffsetDateTime,
    pub ended_at: Option<OffsetDateTime>,
    pub granted_by: Option<Uuid>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostGovernanceState {
    Hosted,
    Recruiting,
    SystemCare,
}

impl HostGovernanceState {
    pub fn as_db(self) -> &'static str {
        match self {
            Self::Hosted => "hosted",
            Self::Recruiting => "recruiting",
            Self::SystemCare => "system_care",
        }
    }

    pub fn from_db(value: &str) -> Self {
        match value {
            "recruiting" => Self::Recruiting,
            "system_care" => Self::SystemCare,
            _ => Self::Hosted,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpaceHostIdentity {
    pub tenure_id: Uuid,
    pub user_id: Uuid,
    pub display_name: String,
    pub email: Option<String>,
    pub role: HostTenureRole,
    pub status: HostTenureStatus,
    pub started_at: OffsetDateTime,
    pub ended_at: Option<OffsetDateTime>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpaceGovernanceEvent {
    pub id: Uuid,
    pub action: String,
    pub actor_name: Option<String>,
    pub from_name: Option<String>,
    pub to_name: Option<String>,
    pub note: Option<String>,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpaceGovernanceSnapshot {
    pub space_id: Uuid,
    pub state: HostGovernanceState,
    pub recruitment_note: Option<String>,
    pub current_user_role: Option<HostTenureRole>,
    pub can_manage_content: bool,
    pub can_manage_governance: bool,
    pub active_hosts: Vec<SpaceHostIdentity>,
    pub past_hosts: Vec<SpaceHostIdentity>,
    pub events: Vec<SpaceGovernanceEvent>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldPresence {
    pub id: Uuid,
    pub subject_kind: PresenceSubjectKind,
    pub subject_id: Uuid,
    pub owner_user_id: Uuid,
    pub space_id: Uuid,
    pub scene_id: Uuid,
    pub spawn_point_id: Option<Uuid>,
    pub x: f64,
    pub y: f64,
    pub entry_method: EntryMethod,
    pub entered_at: OffsetDateTime,
    pub last_seen_at: OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnterSpaceOutcome {
    pub bundle: SceneBundle,
    pub spawn: SceneSpawnPoint,
    pub presence: WorldPresence,
    pub companions_moved: i64,
}
