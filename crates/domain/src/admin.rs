use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdminStats {
    pub spaces_count: i64,
    pub guides_count: i64,
    pub users_count: i64,
    pub pending_resident_applications: i64,
}
