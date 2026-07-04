use instant_domain::guides::{GuideStatus, GuideSummary};
use sqlx::{PgPool, Row};

pub async fn list_published_guides(
    pool: &PgPool,
    province: Option<String>,
) -> Result<Vec<GuideSummary>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT id, title_zh, title_en, province, city, district, spot_name,
               status::text AS status, featured
        FROM guides
        WHERE status = 'published'
          AND ($1::text IS NULL OR province = $1)
        ORDER BY featured DESC, created_at DESC
        "#,
    )
    .bind(province)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(GuideSummary {
                id: row.try_get("id")?,
                title_zh: row.try_get("title_zh")?,
                title_en: row.try_get("title_en")?,
                province: row.try_get("province")?,
                city: row.try_get("city")?,
                district: row.try_get("district")?,
                spot_name: row.try_get("spot_name")?,
                status: match row.try_get::<String, _>("status")?.as_str() {
                    "draft" => GuideStatus::Draft,
                    "archived" => GuideStatus::Archived,
                    _ => GuideStatus::Published,
                },
                featured: row.try_get("featured")?,
            })
        })
        .collect()
}
