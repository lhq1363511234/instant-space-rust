use instant_domain::site::{HomePageAdminState, HomePageConfig, SitePageVersion};
use serde_json::Value;
use sqlx::{PgPool, Row};
use uuid::Uuid;

const HOME_PAGE_KEY: &str = "home";

fn decode_config(value: Value) -> Result<HomePageConfig, sqlx::Error> {
    serde_json::from_value(value).map_err(|err| sqlx::Error::Decode(Box::new(err)))
}

pub async fn get_public_home_config(pool: &PgPool) -> Result<Option<HomePageConfig>, sqlx::Error> {
    let value = sqlx::query_scalar::<_, Value>(
        "SELECT published_config FROM site_page_configs WHERE page_key = $1",
    )
    .bind(HOME_PAGE_KEY)
    .fetch_optional(pool)
    .await?;

    value.map(decode_config).transpose()
}

pub async fn get_home_admin_state(
    pool: &PgPool,
) -> Result<Option<HomePageAdminState>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT draft_config, published_config, published_version,
               to_char(updated_at, 'YYYY-MM-DD"T"HH24:MI:SSOF') AS updated_at
        FROM site_page_configs
        WHERE page_key = $1
        "#,
    )
    .bind(HOME_PAGE_KEY)
    .fetch_optional(pool)
    .await?;

    row.map(|row| {
        Ok(HomePageAdminState {
            draft: decode_config(row.try_get("draft_config")?)?,
            published: decode_config(row.try_get("published_config")?)?,
            published_version: row.try_get("published_version")?,
            updated_at: row.try_get("updated_at")?,
        })
    })
    .transpose()
}

pub async fn save_home_draft(
    pool: &PgPool,
    config: &HomePageConfig,
    actor_id: Uuid,
) -> Result<HomePageAdminState, sqlx::Error> {
    let value = serde_json::to_value(config).map_err(|err| sqlx::Error::Decode(Box::new(err)))?;
    sqlx::query(
        r#"
        INSERT INTO site_page_configs
          (page_key, draft_config, published_config, published_version, updated_by)
        VALUES ($1, $2, $2, 0, $3)
        ON CONFLICT (page_key) DO UPDATE
        SET draft_config = EXCLUDED.draft_config,
            updated_by = EXCLUDED.updated_by,
            updated_at = now()
        "#,
    )
    .bind(HOME_PAGE_KEY)
    .bind(value)
    .bind(actor_id)
    .execute(pool)
    .await?;

    Ok(get_home_admin_state(pool).await?.unwrap_or_default())
}

pub async fn publish_home_config(
    pool: &PgPool,
    config: &HomePageConfig,
    actor_id: Uuid,
    actor_email: &str,
) -> Result<HomePageAdminState, sqlx::Error> {
    let value = serde_json::to_value(config).map_err(|err| sqlx::Error::Decode(Box::new(err)))?;
    let mut tx = pool.begin().await?;
    let current = sqlx::query_scalar::<_, i32>(
        "SELECT published_version FROM site_page_configs WHERE page_key = $1 FOR UPDATE",
    )
    .bind(HOME_PAGE_KEY)
    .fetch_optional(&mut *tx)
    .await?
    .unwrap_or(0);
    let next = current + 1;

    sqlx::query(
        r#"
        INSERT INTO site_page_configs
          (page_key, draft_config, published_config, published_version, updated_by)
        VALUES ($1, $2, $2, $3, $4)
        ON CONFLICT (page_key) DO UPDATE
        SET draft_config = EXCLUDED.draft_config,
            published_config = EXCLUDED.published_config,
            published_version = EXCLUDED.published_version,
            updated_by = EXCLUDED.updated_by,
            updated_at = now()
        "#,
    )
    .bind(HOME_PAGE_KEY)
    .bind(value.clone())
    .bind(next)
    .bind(actor_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO site_page_versions
          (page_key, version, config, actor_id, actor_email)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(HOME_PAGE_KEY)
    .bind(next)
    .bind(value)
    .bind(actor_id)
    .bind(actor_email)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(get_home_admin_state(pool).await?.unwrap_or_default())
}

pub async fn list_home_versions(pool: &PgPool) -> Result<Vec<SitePageVersion>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT id, version, actor_email,
               to_char(created_at, 'YYYY-MM-DD"T"HH24:MI:SSOF') AS created_at
        FROM site_page_versions
        WHERE page_key = $1
        ORDER BY version DESC
        LIMIT 50
        "#,
    )
    .bind(HOME_PAGE_KEY)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(SitePageVersion {
                id: row.try_get("id")?,
                version: row.try_get("version")?,
                actor_email: row.try_get("actor_email")?,
                created_at: row.try_get("created_at")?,
            })
        })
        .collect()
}

pub async fn restore_home_version_to_draft(
    pool: &PgPool,
    version_id: Uuid,
    actor_id: Uuid,
) -> Result<HomePageAdminState, sqlx::Error> {
    let value = sqlx::query_scalar::<_, Value>(
        "SELECT config FROM site_page_versions WHERE id = $1 AND page_key = $2",
    )
    .bind(version_id)
    .bind(HOME_PAGE_KEY)
    .fetch_one(pool)
    .await?;

    sqlx::query(
        r#"
        UPDATE site_page_configs
        SET draft_config = $2, updated_by = $3, updated_at = now()
        WHERE page_key = $1
        "#,
    )
    .bind(HOME_PAGE_KEY)
    .bind(value)
    .bind(actor_id)
    .execute(pool)
    .await?;

    Ok(get_home_admin_state(pool).await?.unwrap_or_default())
}
