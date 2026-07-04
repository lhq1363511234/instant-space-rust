use instant_domain::chat::ChatMessage;
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub async fn list_messages(pool: &PgPool, space_id: Uuid) -> Result<Vec<ChatMessage>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT id, space_id, sender, body, created_at FROM chat_messages WHERE space_id = $1 ORDER BY created_at ASC",
    )
    .bind(space_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(ChatMessage {
                id: row.try_get("id")?,
                space_id: row.try_get("space_id")?,
                sender: row.try_get("sender")?,
                body: row.try_get("body")?,
                created_at: row.try_get("created_at")?,
            })
        })
        .collect()
}
