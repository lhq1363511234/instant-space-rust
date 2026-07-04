use sqlx::{PgPool, Row};
use time::OffsetDateTime;
use uuid::Uuid;

use instant_domain::auth::CurrentUser;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatedUser {
    pub id: Uuid,
    pub email: String,
    pub name: Option<String>,
}

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

pub async fn create_user(
    pool: &PgPool,
    email: &str,
    name: Option<&str>,
    password_hash: &str,
) -> Result<CreatedUser, sqlx::Error> {
    let row = sqlx::query(
        r#"
        INSERT INTO users (email, name, password_hash)
        VALUES ($1, $2, $3)
        RETURNING id, email, name
        "#,
    )
    .bind(email)
    .bind(name)
    .bind(password_hash)
    .fetch_one(pool)
    .await?;

    Ok(CreatedUser {
        id: row.try_get("id")?,
        email: row.try_get("email")?,
        name: row.try_get("name")?,
    })
}

pub async fn create_session(
    pool: &PgPool,
    user_id: Uuid,
    token_hash: &str,
    expires_at: OffsetDateTime,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO sessions (user_id, token_hash, expires_at)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(user_id)
    .bind(token_hash)
    .bind(expires_at)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn current_user_by_token(
    pool: &PgPool,
    token_hash: &str,
) -> Result<Option<CurrentUser>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT users.id, users.email, users.name
        FROM sessions
        JOIN users ON users.id = sessions.user_id
        WHERE sessions.token_hash = $1
          AND sessions.expires_at > now()
        "#,
    )
    .bind(token_hash)
    .fetch_optional(pool)
    .await?;

    row.map(|record| {
        Ok(CurrentUser {
            id: record.try_get("id")?,
            email: record.try_get("email")?,
            name: record.try_get("name")?,
        })
    })
    .transpose()
}

pub async fn current_user_by_id(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<CurrentUser>, sqlx::Error> {
    let row = sqlx::query("SELECT id, email, name FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

    row.map(|record| {
        Ok(CurrentUser {
            id: record.try_get("id")?,
            email: record.try_get("email")?,
            name: record.try_get("name")?,
        })
    })
    .transpose()
}
