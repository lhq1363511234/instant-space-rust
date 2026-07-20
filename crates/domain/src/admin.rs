use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::UserRole;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdminStats {
    pub spaces_count: i64,
    pub guides_count: i64,
    pub users_count: i64,
    pub pending_resident_applications: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdminUser {
    pub id: Uuid,
    pub email: String,
    pub name: Option<String>,
    pub role: UserRole,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResidentApplication {
    pub space_id: Uuid,
    pub name_zh: String,
    pub name_en: Option<String>,
    pub host_user_id: Option<Uuid>,
    pub host_email: Option<String>,
    pub resident_days: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: Uuid,
    pub actor_email: Option<String>,
    pub action: String,
    pub target_type: String,
    pub target_id: Option<String>,
    pub detail: Option<String>,
    pub created_at: String,
}
