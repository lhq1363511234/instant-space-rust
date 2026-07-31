use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatMessageKind {
    #[default]
    Text,
    System,
    Help,
    HelpResolved,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: Uuid,
    pub space_id: Uuid,
    pub sender: String,
    pub body: String,
    #[serde(default)]
    pub kind: ChatMessageKind,
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

/// A help request raised inside a Space room (Phase 6 minimal closed loop).
/// Guests ask, the host or anyone on site resolves; resolved requests leave
/// the active list but stay in history.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpaceHelp {
    pub id: Uuid,
    pub space_id: Uuid,
    pub body: String,
    pub requester_name: Option<String>,
    pub resolved_at: Option<String>,
    pub created_at: String,
}
