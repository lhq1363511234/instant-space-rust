use sqlx::{PgPool, Row};
use time::OffsetDateTime;
use uuid::Uuid;

use instant_domain::admin::AdminUser;
use instant_domain::auth::{CurrentUser, UserRole};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatedUser {
    pub id: Uuid,
    pub email: String,
    pub name: Option<String>,
    pub role: UserRole,
}

pub async fn find_user_password_hash(
    pool: &PgPool,
    email: &str,
) -> Result<Option<(Uuid, String, UserRole)>, sqlx::Error> {
    let row = sqlx::query("SELECT id, password_hash, role FROM users WHERE email = $1")
        .bind(email)
        .fetch_optional(pool)
        .await?;

    row.map(|record| {
        Ok((
            record.try_get("id")?,
            record.try_get("password_hash")?,
            user_role_from_db(record.try_get::<String, _>("role")?.as_str()),
        ))
    })
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
        RETURNING id, email, name, role
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
        role: user_role_from_db(row.try_get::<String, _>("role")?.as_str()),
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

    // Expired rows would otherwise accumulate forever and slow this insert down.
    sqlx::query("DELETE FROM sessions WHERE expires_at < now()")
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn delete_session_by_token(pool: &PgPool, token_hash: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM sessions WHERE token_hash = $1")
        .bind(token_hash)
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
        SELECT users.id, users.email, users.name, users.role
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
            role: user_role_from_db(record.try_get::<String, _>("role")?.as_str()),
        })
    })
    .transpose()
}

pub async fn current_user_by_id(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<CurrentUser>, sqlx::Error> {
    let row = sqlx::query("SELECT id, email, name, role FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

    row.map(|record| {
        Ok(CurrentUser {
            id: record.try_get("id")?,
            email: record.try_get("email")?,
            name: record.try_get("name")?,
            role: user_role_from_db(record.try_get::<String, _>("role")?.as_str()),
        })
    })
    .transpose()
}

fn user_role_from_db(value: &str) -> UserRole {
    match value {
        "admin" => UserRole::Admin,
        "super_admin" => UserRole::SuperAdmin,
        _ => UserRole::User,
    }
}

fn user_role_to_db(role: &UserRole) -> &'static str {
    match role {
        UserRole::User => "user",
        UserRole::Admin => "admin",
        UserRole::SuperAdmin => "super_admin",
    }
}

/// Admin-only: list every user with role, newest first.
pub async fn list_users(pool: &PgPool) -> Result<Vec<AdminUser>, sqlx::Error> {
    let rows = sqlx::query("SELECT id, email, name, role FROM users ORDER BY created_at DESC")
        .fetch_all(pool)
        .await?;

    rows.into_iter()
        .map(|record| {
            Ok(AdminUser {
                id: record.try_get("id")?,
                email: record.try_get("email")?,
                name: record.try_get("name")?,
                role: user_role_from_db(record.try_get::<String, _>("role")?.as_str()),
            })
        })
        .collect()
}

/// Admin-only: change a user's role.
pub async fn set_user_role(
    pool: &PgPool,
    user_id: Uuid,
    role: UserRole,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE users SET role = $2, updated_at = now() WHERE id = $1")
        .bind(user_id)
        .bind(user_role_to_db(&role))
        .execute(pool)
        .await?;

    Ok(())
}
