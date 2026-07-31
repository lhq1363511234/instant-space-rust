use instant_domain::admin::{AdminStats, AuditLogEntry};
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub async fn stats(pool: &PgPool) -> Result<AdminStats, sqlx::Error> {
    let spaces_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM spaces")
        .fetch_one(pool)
        .await?;
    let guides_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM guides")
        .fetch_one(pool)
        .await?;
    let users_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;
    let pending_resident_applications = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM spaces WHERE resident_apply_at IS NOT NULL AND resident = false",
    )
    .fetch_one(pool)
    .await?;

    Ok(AdminStats {
        spaces_count,
        guides_count,
        users_count,
        pending_resident_applications,
    })
}

/// Append an entry to the admin audit log. Best-effort accountability record.
pub async fn record_audit(
    pool: &PgPool,
    actor_id: Option<Uuid>,
    actor_email: Option<&str>,
    action: &str,
    target_type: &str,
    target_id: Option<&str>,
    detail: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO admin_audit_log (actor_id, actor_email, action, target_type, target_id, detail)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(actor_id)
    .bind(actor_email)
    .bind(action)
    .bind(target_type)
    .bind(target_id)
    .bind(detail)
    .execute(pool)
    .await?;

    Ok(())
}

/// Admin-only: list recent audit entries, newest first.
pub async fn list_audit_log(pool: &PgPool, limit: i64) -> Result<Vec<AuditLogEntry>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT id, actor_email, action, target_type, target_id, detail,
               to_char(created_at, 'YYYY-MM-DD"T"HH24:MI:SSOF') AS created_at
        FROM admin_audit_log
        ORDER BY created_at DESC
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(AuditLogEntry {
                id: row.try_get("id")?,
                actor_email: row.try_get("actor_email")?,
                action: row.try_get("action")?,
                target_type: row.try_get("target_type")?,
                target_id: row.try_get("target_id")?,
                detail: row.try_get("detail")?,
                created_at: row.try_get("created_at")?,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn admin_stats_include_seeded_rows() {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
        let pool = sqlx::PgPool::connect(&database_url).await.expect("pool");
        let stats = stats(&pool).await.expect("stats");

        assert!(stats.spaces_count >= 2);
        assert!(stats.guides_count >= 1);
        assert!(stats.users_count >= 1);
    }
}

/// Full-table CSV export for operators (Phase 5). Admin console downloads these
/// for offline review, backups of content metadata, and spreadsheet analysis.
/// Rows are streamed through one query; the CSV is built with proper escaping.
pub async fn export_spaces_csv(pool: &PgPool) -> Result<String, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT id, name_zh, name_en, space_type, country, province, city, district,
               spot_name, lat, lng, status::text AS status, is_public, host_user_id,
               created_at
        FROM spaces
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await?;

    let mut out = String::from("id,name_zh,name_en,space_type,country,province,city,district,spot_name,lat,lng,status,is_public,host_user_id,created_at\n");
    for row in rows {
        let created_at: Option<time::OffsetDateTime> = row.try_get("created_at")?;
        out.push_str(&csv_join(&[
            csv_escape(&row.try_get::<String, _>("id")?),
            csv_escape(&row.try_get::<String, _>("name_zh")?),
            csv_escape(&row.try_get::<Option<String>, _>("name_en")?.unwrap_or_default()),
            csv_escape(&row.try_get::<String, _>("space_type")?),
            csv_escape(&row.try_get::<Option<String>, _>("country")?.unwrap_or_default()),
            csv_escape(&row.try_get::<String, _>("province")?),
            csv_escape(&row.try_get::<String, _>("city")?),
            csv_escape(&row.try_get::<Option<String>, _>("district")?.unwrap_or_default()),
            csv_escape(&row.try_get::<Option<String>, _>("spot_name")?.unwrap_or_default()),
            row.try_get::<f64, _>("lat")?.to_string(),
            row.try_get::<f64, _>("lng")?.to_string(),
            csv_escape(&row.try_get::<String, _>("status")?),
            row.try_get::<bool, _>("is_public")?.to_string(),
            row.try_get::<Option<Uuid>, _>("host_user_id")?.map(|id| id.to_string()).unwrap_or_default(),
            csv_escape(&created_at.map(|value| value.to_string()).unwrap_or_default()),
        ]));
    }
    Ok(out)
}

pub async fn export_guides_csv(pool: &PgPool) -> Result<String, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT id, title_zh, title_en, country, province, city, district, spot_name,
               guide_type, category, status::text AS status, featured, author_id,
               space_id, created_at, updated_at
        FROM guides
        ORDER BY updated_at DESC
        "#,
    )
    .fetch_all(pool)
    .await?;

    let mut out = String::from("id,title_zh,title_en,country,province,city,district,spot_name,guide_type,category,status,featured,author_id,space_id,created_at,updated_at\n");
    for row in rows {
        out.push_str(&csv_join(&[
            csv_escape(&row.try_get::<String, _>("id")?),
            csv_escape(&row.try_get::<String, _>("title_zh")?),
            csv_escape(&row.try_get::<Option<String>, _>("title_en")?.unwrap_or_default()),
            csv_escape(&row.try_get::<Option<String>, _>("country")?.unwrap_or_default()),
            csv_escape(&row.try_get::<String, _>("province")?),
            csv_escape(&row.try_get::<String, _>("city")?),
            csv_escape(&row.try_get::<Option<String>, _>("district")?.unwrap_or_default()),
            csv_escape(&row.try_get::<Option<String>, _>("spot_name")?.unwrap_or_default()),
            csv_escape(&row.try_get::<String, _>("guide_type")?),
            csv_escape(&row.try_get::<Option<String>, _>("category")?.unwrap_or_default()),
            csv_escape(&row.try_get::<String, _>("status")?),
            row.try_get::<bool, _>("featured")?.to_string(),
            row.try_get::<Option<Uuid>, _>("author_id")?.map(|id| id.to_string()).unwrap_or_default(),
            row.try_get::<Option<Uuid>, _>("space_id")?.map(|id| id.to_string()).unwrap_or_default(),
            csv_escape(&row.try_get::<time::OffsetDateTime, _>("created_at")?.to_string()),
            csv_escape(&row.try_get::<time::OffsetDateTime, _>("updated_at")?.to_string()),
        ]));
    }
    Ok(out)
}

fn csv_escape(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') || field.contains('\r') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

fn csv_join(fields: &[String]) -> String {
    let mut line = fields.join(",");
    line.push('\n');
    line
}
