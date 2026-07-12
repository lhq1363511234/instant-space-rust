use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpaceType {
    Scenic,
    Food,
    Park,
    Transit,
    Event,
    Custom,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpaceStatus {
    Active,
    Expired,
    Closed,
    Archived,
    Template,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpaceLifecycleAction {
    RegeneratePassword,
    Close,
    Reactivate,
    ArchiveTemplate,
    ApplyResident,
}

impl SpaceLifecycleAction {
    pub fn next_status(self, current: SpaceStatus) -> SpaceStatus {
        match self {
            SpaceLifecycleAction::Close => SpaceStatus::Closed,
            SpaceLifecycleAction::Reactivate => SpaceStatus::Active,
            SpaceLifecycleAction::ArchiveTemplate => SpaceStatus::Template,
            SpaceLifecycleAction::RegeneratePassword | SpaceLifecycleAction::ApplyResident => {
                current
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpaceSummary {
    pub id: Uuid,
    pub name_zh: String,
    pub name_en: Option<String>,
    pub space_type: SpaceType,
    pub country: Option<String>,
    pub province: Option<String>,
    pub city: Option<String>,
    pub district: Option<String>,
    pub spot_name: Option<String>,
    pub address_line: Option<String>,
    pub lat: f64,
    pub lng: f64,
    pub is_public: bool,
    pub status: SpaceStatus,
    pub expires_at: Option<OffsetDateTime>,
    pub online_count: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpaceDetail {
    pub summary: SpaceSummary,
    pub description_zh: Option<String>,
    pub description_en: Option<String>,
    pub tag_zh: Option<String>,
    pub tag_en: Option<String>,
    pub discord_group: Option<String>,
    pub qq_group: Option<String>,
    pub password_version: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DomainError {
    #[error("password code must be exactly six digits")]
    InvalidPasswordCode,
}

impl SpaceSummary {
    pub fn is_visible_on_home_map(&self) -> bool {
        matches!(self.status, SpaceStatus::Active | SpaceStatus::Expired)
    }
}

pub fn hotspot_name(password_code: &str) -> Result<String, DomainError> {
    let is_six_digits =
        password_code.len() == 6 && password_code.chars().all(|c| c.is_ascii_digit());
    if is_six_digits {
        Ok(format!("InstantSpace_{password_code}"))
    } else {
        Err(DomainError::InvalidPasswordCode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_hotspot_name_uses_six_digit_password() {
        assert_eq!(
            hotspot_name("123456"),
            Ok("InstantSpace_123456".to_string())
        );
    }

    #[test]
    fn generated_hotspot_name_rejects_non_six_digit_password() {
        assert_eq!(
            hotspot_name("12345").unwrap_err(),
            DomainError::InvalidPasswordCode
        );
        assert_eq!(
            hotspot_name("abcdef").unwrap_err(),
            DomainError::InvalidPasswordCode
        );
    }

    #[test]
    fn active_space_is_publicly_browsable() {
        let summary = SpaceSummary {
            id: Uuid::nil(),
            name_zh: "外滩".to_string(),
            name_en: Some("The Bund".to_string()),
            space_type: SpaceType::Scenic,
            country: Some("中国".to_string()),
            province: Some("上海市".to_string()),
            city: Some("上海市".to_string()),
            district: None,
            spot_name: Some("外滩".to_string()),
            address_line: None,
            lat: 31.2397,
            lng: 121.4998,
            is_public: true,
            status: SpaceStatus::Active,
            expires_at: None,
            online_count: 0,
        };
        assert!(summary.is_visible_on_home_map());
    }

    #[test]
    fn resident_application_sets_review_state() {
        let next = SpaceLifecycleAction::ApplyResident.next_status(SpaceStatus::Active);
        assert_eq!(next, SpaceStatus::Active);
    }

    #[test]
    fn archived_template_status_is_explicit() {
        let next = SpaceLifecycleAction::ArchiveTemplate.next_status(SpaceStatus::Active);
        assert_eq!(next, SpaceStatus::Template);
    }
}
