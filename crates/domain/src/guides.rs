use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    pub province: String,
    pub city: String,
    pub district: Option<String>,
    pub spot_name: Option<String>,
    pub status: GuideStatus,
    pub featured: bool,
}
