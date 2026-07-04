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
