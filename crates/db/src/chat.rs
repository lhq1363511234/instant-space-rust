use instant_domain::chat::ChatMessage;
use sqlx::{PgPool, Row};
use time::OffsetDateTime;
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

pub async fn create_access_session(
    pool: &PgPool,
    space_id: Uuid,
    token: &str,
    password_version: i32,
    expires_at: OffsetDateTime,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO access_sessions (space_id, token_hash, password_version, expires_at)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(space_id)
    .bind(token)
    .bind(password_version)
    .bind(expires_at)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn has_valid_access_session(
    pool: &PgPool,
    space_id: Uuid,
    token: &str,
) -> Result<Option<i32>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT access_sessions.password_version
        FROM access_sessions
        WHERE access_sessions.space_id = $1
          AND access_sessions.token_hash = $2
          AND access_sessions.expires_at > now()
        LIMIT 1
        "#,
    )
    .bind(space_id)
    .bind(token)
    .fetch_optional(pool)
    .await?;

    row.map(|record| record.try_get("password_version"))
        .transpose()
}

pub async fn insert_message(
    pool: &PgPool,
    space_id: Uuid,
    sender: String,
    body: String,
    password_version: i32,
) -> Result<ChatMessage, sqlx::Error> {
    let row = sqlx::query(
        r#"
        INSERT INTO chat_messages (space_id, sender, body, password_version)
        VALUES ($1, $2, $3, $4)
        RETURNING id, space_id, sender, body, created_at
        "#,
    )
    .bind(space_id)
    .bind(sender)
    .bind(body)
    .bind(password_version)
    .fetch_one(pool)
    .await?;

    Ok(ChatMessage {
        id: row.try_get("id")?,
        space_id: row.try_get("space_id")?,
        sender: row.try_get("sender")?,
        body: row.try_get("body")?,
        created_at: row.try_get("created_at")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spaces::{create_host_space, rotate_space_password, CreateSpaceInput};
    use instant_domain::spaces::SpaceType;
    use time::Duration;

    #[tokio::test]
    async fn rotated_space_password_keeps_session_but_marks_version_stale() {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
        let pool = sqlx::PgPool::connect(&database_url).await.expect("pool");
        let email = format!("chat-host-{}@example.com", uuid::Uuid::new_v4());
        let user = crate::users::create_user(&pool, &email, Some("Chat Host"), "hash")
            .await
            .expect("user");
        let space = create_host_space(
            &pool,
            CreateSpaceInput {
                name_zh: "私密测试空间".to_string(),
                name_en: Some("Private Test Space".to_string()),
                country: Some("China".to_string()),
                province: "上海市".to_string(),
                city: "上海市".to_string(),
                district: Some("黄浦区".to_string()),
                spot_name: None,
                address_line: None,
                lat: 31.23,
                lng: 121.47,
                space_type: SpaceType::Custom,
                custom_type: None,
                description_zh: None,
                description_en: None,
                tag_zh: None,
                tag_en: None,
                is_public: false,
                duration_hours: 24,
                password_hash: "hash".to_string(),
                host_user_id: user.id,
            },
        )
        .await
        .expect("space");

        let token = uuid::Uuid::new_v4().to_string();
        create_access_session(
            &pool,
            space.id,
            &token,
            1,
            OffsetDateTime::now_utc() + Duration::hours(1),
        )
        .await
        .expect("access session");
        assert_eq!(
            has_valid_access_session(&pool, space.id, &token)
                .await
                .expect("valid"),
            Some(1)
        );

        rotate_space_password(&pool, space.id, "new-hash".to_string())
            .await
            .expect("rotate");
        assert_eq!(
            has_valid_access_session(&pool, space.id, &token)
                .await
                .expect("still readable"),
            Some(1)
        );

        sqlx::query("DELETE FROM spaces WHERE id = $1")
            .bind(space.id)
            .execute(&pool)
            .await
            .expect("cleanup space");
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user.id)
            .execute(&pool)
            .await
            .expect("cleanup user");
    }
}
