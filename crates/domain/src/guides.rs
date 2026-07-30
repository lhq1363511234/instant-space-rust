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
