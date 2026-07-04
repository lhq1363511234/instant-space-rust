use instant_domain::spaces::{SpaceStatus, SpaceSummary, SpaceType};
use sqlx::{PgPool, Row};

#[derive(Debug, Default, Clone)]
pub struct SpaceFilter {
    pub q: Option<String>,
    pub space_type: Option<SpaceType>,
}

pub async fn list_home_spaces(
    pool: &PgPool,
    filter: SpaceFilter,
) -> Result<Vec<SpaceSummary>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT id, name_zh, name_en, space_type::text AS space_type, province, city, district,
               lat, lng, is_public, status::text AS status, online_count
        FROM spaces
        WHERE status IN ('active', 'expired')
          AND ($1::text IS NULL OR name_zh ILIKE '%' || $1 || '%' OR name_en ILIKE '%' || $1 || '%')
          AND ($2::text IS NULL OR space_type::text = $2)
        ORDER BY created_at DESC
        "#,
    )
    .bind(filter.q)
    .bind(filter.space_type.map(space_type_to_db))
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(row_to_space_summary).collect()
}

pub async fn space_password_hash(
    pool: &PgPool,
    space_id: uuid::Uuid,
) -> Result<Option<(String, i32)>, sqlx::Error> {
    let row = sqlx::query("SELECT password_hash, password_version FROM spaces WHERE id = $1")
        .bind(space_id)
        .fetch_optional(pool)
        .await?;

    row.map(|record| {
        Ok((
            record.try_get("password_hash")?,
            record.try_get("password_version")?,
        ))
    })
    .transpose()
}

fn row_to_space_summary(row: sqlx::postgres::PgRow) -> Result<SpaceSummary, sqlx::Error> {
    Ok(SpaceSummary {
        id: row.try_get("id")?,
        name_zh: row.try_get("name_zh")?,
        name_en: row.try_get("name_en")?,
        space_type: space_type_from_db(row.try_get::<String, _>("space_type")?.as_str()),
        province: row.try_get("province")?,
        city: row.try_get("city")?,
        district: row.try_get("district")?,
        lat: row.try_get("lat")?,
        lng: row.try_get("lng")?,
        is_public: row.try_get("is_public")?,
        status: status_from_db(row.try_get::<String, _>("status")?.as_str()),
        online_count: row.try_get("online_count")?,
    })
}

fn space_type_to_db(space_type: SpaceType) -> String {
    match space_type {
        SpaceType::Scenic => "scenic",
        SpaceType::Food => "food",
        SpaceType::Park => "park",
        SpaceType::Transit => "transit",
        SpaceType::Event => "event",
        SpaceType::Custom => "custom",
    }
    .to_string()
}

fn space_type_from_db(value: &str) -> SpaceType {
    match value {
        "food" => SpaceType::Food,
        "park" => SpaceType::Park,
        "transit" => SpaceType::Transit,
        "event" => SpaceType::Event,
        "custom" => SpaceType::Custom,
        _ => SpaceType::Scenic,
    }
}

fn status_from_db(value: &str) -> SpaceStatus {
    match value {
        "expired" => SpaceStatus::Expired,
        "closed" => SpaceStatus::Closed,
        "archived" => SpaceStatus::Archived,
        "template" => SpaceStatus::Template,
        _ => SpaceStatus::Active,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn list_home_spaces_returns_seeded_public_and_private_rows() {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
        let pool = sqlx::PgPool::connect(&database_url).await.expect("pool");
        let rows = list_home_spaces(&pool, SpaceFilter::default())
            .await
            .expect("rows");
        assert!(rows.iter().any(|space| space.name_zh == "外滩"));
        assert!(rows.iter().any(|space| space.name_zh == "私密茶室"));
    }
}
