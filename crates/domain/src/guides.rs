use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuideStatus {
    Draft,
    Published,
    Archived,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GuideSummary {
    pub id: Uuid,
    pub title_zh: String,
    pub title_en: Option<String>,
    #[serde(default)]
    pub country: Option<String>,
    pub province: String,
    pub city: String,
    pub district: Option<String>,
    pub spot_name: Option<String>,
    pub status: GuideStatus,
    pub featured: bool,
    pub author_id: Option<Uuid>,
    pub space_id: Option<Uuid>,
    #[serde(default)]
    pub can_edit: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GuideSection {
    #[serde(default)]
    pub id: String,
    #[serde(rename = "type", default = "default_section_type")]
    pub section_type: String,
    #[serde(default)]
    pub title_zh: String,
    pub title_en: Option<String>,
    #[serde(default)]
    pub content_zh: String,
    pub content_en: Option<String>,
    #[serde(default)]
    pub images: Vec<String>,
}

fn default_section_type() -> String {
    "text".to_string()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GuideDetail {
    pub id: Uuid,
    pub title_zh: String,
    pub title_en: Option<String>,
    pub summary_zh: Option<String>,
    pub summary_en: Option<String>,
    pub content_zh: Option<String>,
    pub content_en: Option<String>,
    pub guide_type: String,
    pub category: Option<String>,
    #[serde(default)]
    pub country: Option<String>,
    pub province: String,
    pub city: String,
    pub district: Option<String>,
    pub spot_name: Option<String>,
    pub status: GuideStatus,
    pub featured: bool,
    pub author_id: Option<Uuid>,
    pub author_name: Option<String>,
    pub space_id: Option<Uuid>,
    #[serde(default)]
    pub can_edit: bool,
    pub cover_image_url: Option<String>,
    pub images: Vec<String>,
    pub sections: Vec<GuideSection>,
    pub created_at: String,
    pub updated_at: String,
}

/// A frozen snapshot of one guide at one point in time (Phase 4 content
/// versioning). Hosts and admins can review the history and restore any
/// version, which rewrites the guide row from the snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GuideVersion {
    pub id: Uuid,
    pub guide_id: Uuid,
    pub version_no: i32,
    pub title_zh: String,
    pub title_en: Option<String>,
    pub summary_zh: Option<String>,
    pub summary_en: Option<String>,
    pub content_zh: Option<String>,
    pub content_en: Option<String>,
    #[serde(default)]
    pub sections: Vec<GuideSection>,
    #[serde(default)]
    pub images: Vec<String>,
    pub cover_image_url: Option<String>,
    pub edited_by: Option<Uuid>,
    pub edited_by_name: Option<String>,
    pub created_at: String,
}

/// Paged guide result shared by the directory and the admin console.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct PaginatedGuides {
    pub items: Vec<GuideSummary>,
    pub total: i64,
}

/// Aggregate status counts for the admin console stat cards.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct GuideStatusCounts {
    pub total: i64,
    pub published: i64,
    pub drafts: i64,
    pub archived: i64,
}
