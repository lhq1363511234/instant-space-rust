use instant_domain::admin::AdminStats;
use sqlx::PgPool;

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
