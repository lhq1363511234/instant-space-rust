use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AgentApiKeyRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_email: String,
    pub key_hash: String,
    pub scopes: Vec<String>,
    pub rate_limit_per_minute: i32,
}

pub async fn find_active_key_by_prefix(
    pool: &PgPool,
    prefix: &str,
) -> Result<Option<AgentApiKeyRecord>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT k.id, k.user_id, u.email AS user_email, k.key_hash,
               k.scopes, k.rate_limit_per_minute
        FROM agent_api_keys k
        JOIN users u ON u.id = k.user_id
        WHERE k.key_prefix = $1
          AND k.revoked_at IS NULL
        "#,
    )
    .bind(prefix)
    .fetch_optional(pool)
    .await?;

    row.map(|row| {
        Ok(AgentApiKeyRecord {
            id: row.try_get("id")?,
            user_id: row.try_get("user_id")?,
            user_email: row.try_get("user_email")?,
            key_hash: row.try_get("key_hash")?,
            scopes: row.try_get("scopes")?,
            rate_limit_per_minute: row.try_get("rate_limit_per_minute")?,
        })
    })
    .transpose()
}

pub async fn recent_request_count(pool: &PgPool, key_id: Uuid) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT count(*)::bigint
        FROM agent_api_audit_log
        WHERE key_id = $1
          AND created_at >= now() - interval '1 minute'
        "#,
    )
    .bind(key_id)
    .fetch_one(pool)
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn record_request(
    pool: &PgPool,
    key_id: Uuid,
    user_id: Uuid,
    method: &str,
    path: &str,
    status_code: u16,
    target_type: Option<&str>,
    target_id: Option<&str>,
    remote_addr: Option<&str>,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query(
        r#"
        INSERT INTO agent_api_audit_log
          (key_id, user_id, method, path, status_code, target_type, target_id, remote_addr)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind(key_id)
    .bind(user_id)
    .bind(method)
    .bind(path)
    .bind(i32::from(status_code))
    .bind(target_type)
    .bind(target_id)
    .bind(remote_addr)
    .execute(&mut *tx)
    .await?;

    sqlx::query("UPDATE agent_api_keys SET last_used_at = now() WHERE id = $1")
        .bind(key_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await
}

pub async fn user_manages_space(
    pool: &PgPool,
    user_id: Uuid,
    space_id: Uuid,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT EXISTS(
          SELECT 1 FROM spaces
          WHERE id = $1
            AND (host_user_id = $2 OR creator_id = $2)
            AND status <> 'archived'
        )
        "#,
    )
    .bind(space_id)
    .bind(user_id)
    .fetch_one(pool)
    .await
}

pub async fn user_manages_guide(
    pool: &PgPool,
    user_id: Uuid,
    guide_id: Uuid,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT EXISTS(
          SELECT 1
          FROM guides g
          LEFT JOIN spaces s ON s.id = g.space_id
          WHERE g.id = $1
            AND (g.author_id = $2 OR s.host_user_id = $2 OR s.creator_id = $2)
        )
        "#,
    )
    .bind(guide_id)
    .bind(user_id)
    .fetch_one(pool)
    .await
}
