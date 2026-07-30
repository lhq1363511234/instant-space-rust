use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: Uuid,
    pub space_id: Uuid,
    pub sender: String,
    pub body: String,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatAccessState {
    pub space_id: Uuid,
    /// Chinese display name; the canonical name that always exists.
    pub space_name: String,
    /// English display name when present, so the client can localize the title.
    pub space_name_en: Option<String>,
    pub is_public: bool,
    pub allowed: bool,
    pub password_version: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccessGrant {
    pub password_version: i32,
    pub expires_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatAccess {
    Allowed,
    PasswordVersionMismatch,
}

pub fn check_password_version(expected: i32, actual: i32) -> ChatAccess {
    if expected == actual {
        ChatAccess::Allowed
    } else {
        ChatAccess::PasswordVersionMismatch
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_version_mismatch_requires_reverification_before_send() {
        assert_eq!(
            check_password_version(2, 1),
            ChatAccess::PasswordVersionMismatch
        );
        assert_eq!(check_password_version(2, 2), ChatAccess::Allowed);
    }
}
