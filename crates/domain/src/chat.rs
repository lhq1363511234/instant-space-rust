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
    fn password_version_mismatch_invalidates_chat_access() {
        assert_eq!(
            check_password_version(2, 1),
            ChatAccess::PasswordVersionMismatch
        );
        assert_eq!(check_password_version(2, 2), ChatAccess::Allowed);
    }
}
