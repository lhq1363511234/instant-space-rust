use instant_domain::spaces::{SpaceStatus, SpaceSummary, SpaceType};
use sqlx::{PgPool, Row};

#[derive(Debug, Default, Clone)]
pub struct SpaceFilter {
    pub q: Option<String>,
    pub space_type: Option<SpaceType>,
}

#[derive(Debug, Clone)]
pub struct CreateSpaceInput {
    pub name_zh: String,
    pub name_en: Option<String>,
    pub province: String,
    pub city: String,
    pub district: Option<String>,
    pub lat: f64,
    pub lng: f64,
    pub space_type: SpaceType,
    pub is_public: bool,
    pub password_hash: String,
    pub host_user_id: uuid::Uuid,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreatedSpace {
    pub id: uuid::Uuid,
    pub name_zh: String,
    pub host_user_id: Option<uuid::Uuid>,
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

pub async fn create_host_space(
    pool: &PgPool,
    input: CreateSpaceInput,
) -> Result<CreatedSpace, sqlx::Error> {
    let row = sqlx::query(
        r#"
        INSERT INTO spaces (
            name_zh, name_en, province, city, district, lat, lng, space_type,
            is_public, password_hash, host_user_id, creator_id, status
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8::space_type, $9, $10, $11, $11, 'active')
        RETURNING id, name_zh, host_user_id
        "#,
    )
    .bind(input.name_zh)
    .bind(input.name_en)
    .bind(input.province)
    .bind(input.city)
    .bind(input.district)
    .bind(input.lat)
    .bind(input.lng)
    .bind(space_type_to_db(input.space_type))
    .bind(input.is_public)
    .bind(input.password_hash)
    .bind(input.host_user_id)
    .fetch_one(pool)
    .await?;

    Ok(CreatedSpace {
        id: row.try_get("id")?,
        name_zh: row.try_get("name_zh")?,
        host_user_id: row.try_get("host_user_id")?,
    })
}

pub async fn archive_template(
    pool: &PgPool,
    space_id: uuid::Uuid,
) -> Result<CreatedSpace, sqlx::Error> {
    let row = sqlx::query(
        r#"
        UPDATE spaces
        SET status = 'template', updated_at = now()
        WHERE id = $1
        RETURNING id, name_zh, host_user_id
        "#,
    )
    .bind(space_id)
    .fetch_one(pool)
    .await?;

    Ok(CreatedSpace {
        id: row.try_get("id")?,
        name_zh: row.try_get("name_zh")?,
        host_user_id: row.try_get("host_user_id")?,
    })
}

pub async fn apply_resident(pool: &PgPool, space_id: uuid::Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE spaces
        SET resident_apply_at = now(), updated_at = now()
        WHERE id = $1
        "#,
    )
    .bind(space_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn approve_resident_application(
    pool: &PgPool,
    space_id: uuid::Uuid,
    resident_days: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE spaces
        SET resident = true, resident_days = $2, updated_at = now()
        WHERE id = $1
        "#,
    )
    .bind(space_id)
    .bind(resident_days)
    .execute(pool)
    .await?;

    Ok(())
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

    #[tokio::test]
    async fn create_user_and_space_are_linked() {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
        let pool = sqlx::PgPool::connect(&database_url).await.expect("pool");
        let email = format!("host-{}@example.com", uuid::Uuid::new_v4());
        let user = crate::users::create_user(&pool, &email, Some("Host"), "hash")
            .await
            .expect("user");

        let space = create_host_space(
            &pool,
            CreateSpaceInput {
                name_zh: "测试空间".to_string(),
                name_en: Some("Test Space".to_string()),
                province: "上海市".to_string(),
                city: "上海市".to_string(),
                district: Some("黄浦区".to_string()),
                lat: 31.23,
                lng: 121.47,
                space_type: SpaceType::Scenic,
                is_public: true,
                password_hash: "hash".to_string(),
                host_user_id: user.id,
            },
        )
        .await
        .expect("space");

        assert_eq!(space.host_user_id, Some(user.id));
    }
}
