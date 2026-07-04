use sqlx::{PgPool, Row};
use uuid::Uuid;

pub async fn find_user_password_hash(
    pool: &PgPool,
    email: &str,
) -> Result<Option<(Uuid, String)>, sqlx::Error> {
    let row = sqlx::query("SELECT id, password_hash FROM users WHERE email = $1")
        .bind(email)
        .fetch_optional(pool)
        .await?;

    row.map(|record| Ok((record.try_get("id")?, record.try_get("password_hash")?)))
        .transpose()
}
