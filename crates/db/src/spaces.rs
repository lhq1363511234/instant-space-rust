use instant_domain::spaces::{SpaceStatus, SpaceSummary, SpaceType};
use sqlx::{PgPool, Row};

#[derive(Debug, Default, Clone)]
pub struct SpaceFilter {
    pub q: Option<String>,
    pub space_type: Option<SpaceType>,
    pub country: Option<String>,
    pub province: Option<String>,
    pub city: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreateSpaceInput {
    pub name_zh: String,
    pub name_en: Option<String>,
    pub country: Option<String>,
    pub province: String,
    pub city: String,
    pub district: Option<String>,
    pub spot_name: Option<String>,
    pub address_line: Option<String>,
    pub lat: f64,
    pub lng: f64,
    pub space_type: SpaceType,
    pub custom_type: Option<String>,
    pub description_zh: Option<String>,
    pub description_en: Option<String>,
    pub tag_zh: Option<String>,
    pub tag_en: Option<String>,
    pub is_public: bool,
    pub duration_hours: i32,
    pub password_hash: String,
    pub host_user_id: uuid::Uuid,
}

#[derive(Debug, Clone)]
pub struct UpdateSpaceInput {
    pub name_zh: String,
    pub name_en: Option<String>,
    pub country: Option<String>,
    pub province: String,
    pub city: String,
    pub district: Option<String>,
    pub spot_name: Option<String>,
    pub address_line: Option<String>,
    pub lat: f64,
    pub lng: f64,
    pub is_public: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreatedSpace {
    pub id: uuid::Uuid,
    pub name_zh: String,
    pub host_user_id: Option<uuid::Uuid>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpaceAccessMeta {
    pub id: uuid::Uuid,
    pub name_zh: String,
    pub name_en: Option<String>,
    pub is_public: bool,
    pub password_version: i32,
}

/// Transition any `active` space whose `expires_at` has passed into `expired`.
/// Returns the number of rows updated. This keeps the map, explore list and
/// space detail consistent without relying on per-request ad-hoc checks.
pub async fn expire_stale_spaces(pool: &PgPool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE spaces
        SET status = 'expired', updated_at = now()
        WHERE status = 'active'
          AND expires_at IS NOT NULL
          AND expires_at < now()
        "#,
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

pub async fn list_home_spaces(
    pool: &PgPool,
    filter: SpaceFilter,
) -> Result<Vec<SpaceSummary>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT id, name_zh, name_en, space_type::text AS space_type, country, province, city, district, spot_name, address_line,
               lat, lng, is_public, status::text AS status, expires_at, online_count
        FROM spaces
        WHERE status IN ('active', 'expired')
          AND ($1::text IS NULL OR name_zh ILIKE '%' || $1 || '%' OR name_en ILIKE '%' || $1 || '%')
          AND ($2::text IS NULL OR space_type::text = $2)
          AND (
            $3::text IS NULL
            OR country ILIKE $3
            OR province ILIKE $3
            OR city ILIKE $3
          )
          AND (
            $4::text IS NULL
            OR province ILIKE $4
            OR city ILIKE $4
            OR district ILIKE $4
          )
          AND (
            $5::text IS NULL
            OR city ILIKE $5
            OR district ILIKE $5
            OR spot_name ILIKE $5
          )
        ORDER BY (CASE WHEN resident THEN 10 ELSE 0 END) DESC, created_at DESC
        "#,
    )
    .bind(filter.q)
    .bind(filter.space_type.map(space_type_to_db))
    .bind(filter.country)
    .bind(filter.province)
    .bind(filter.city)
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(row_to_space_summary).collect()
}

pub async fn list_host_spaces(
    pool: &PgPool,
    host_user_id: uuid::Uuid,
) -> Result<Vec<SpaceSummary>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT id, name_zh, name_en, space_type::text AS space_type, country, province, city, district, spot_name, address_line,
               lat, lng, is_public, status::text AS status, expires_at, online_count
        FROM spaces
        WHERE host_user_id = $1
          AND status <> 'archived'
        ORDER BY created_at DESC
        "#,
    )
    .bind(host_user_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(row_to_space_summary).collect()
}

pub async fn list_manageable_spaces(pool: &PgPool) -> Result<Vec<SpaceSummary>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT id, name_zh, name_en, space_type::text AS space_type, country, province, city, district, spot_name, address_line,
               lat, lng, is_public, status::text AS status, expires_at, online_count
        FROM spaces
        WHERE status <> 'archived'
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(row_to_space_summary).collect()
}

pub async fn get_space_summary(
    pool: &PgPool,
    space_id: uuid::Uuid,
) -> Result<Option<SpaceSummary>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT id, name_zh, name_en, space_type::text AS space_type, country, province, city, district, spot_name, address_line,
               lat, lng, is_public, status::text AS status, expires_at, online_count
        FROM spaces
        WHERE id = $1
          AND status <> 'archived'
        "#,
    )
    .bind(space_id)
    .fetch_optional(pool)
    .await?;

    row.map(row_to_space_summary).transpose()
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

pub async fn space_access_meta(
    pool: &PgPool,
    space_id: uuid::Uuid,
) -> Result<Option<SpaceAccessMeta>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT id, name_zh, name_en, is_public, password_version
        FROM spaces
        WHERE id = $1
        "#,
    )
    .bind(space_id)
    .fetch_optional(pool)
    .await?;

    row.map(|record| {
        Ok(SpaceAccessMeta {
            id: record.try_get("id")?,
            name_zh: record.try_get("name_zh")?,
            name_en: record.try_get("name_en")?,
            is_public: record.try_get("is_public")?,
            password_version: record.try_get("password_version")?,
        })
    })
    .transpose()
}

pub async fn space_host_user_id(
    pool: &PgPool,
    space_id: uuid::Uuid,
) -> Result<Option<uuid::Uuid>, sqlx::Error> {
    let row = sqlx::query("SELECT host_user_id FROM spaces WHERE id = $1")
        .bind(space_id)
        .fetch_optional(pool)
        .await?;

    row.map(|record| record.try_get("host_user_id")).transpose()
}

pub async fn create_host_space(
    pool: &PgPool,
    input: CreateSpaceInput,
) -> Result<CreatedSpace, sqlx::Error> {
    let row = sqlx::query(
        r#"
        INSERT INTO spaces (
            name_zh, name_en, country, province, city, district, spot_name, address_line, lat, lng, space_type,
            custom_type, description_zh, description_en, tag_zh, tag_en,
            is_public, duration_hours, expires_at, password_hash, host_user_id, creator_id, status
        )
        VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11::space_type,
            $12, $13, $14, $15, $16,
            $17, $18, now() + make_interval(hours => $18), $19, $20, $20, 'active'
        )
        RETURNING id, name_zh, host_user_id
        "#,
    )
    .bind(input.name_zh)
    .bind(input.name_en)
    .bind(input.country)
    .bind(input.province)
    .bind(input.city)
    .bind(input.district)
    .bind(input.spot_name)
    .bind(input.address_line)
    .bind(input.lat)
    .bind(input.lng)
    .bind(space_type_to_db(input.space_type))
    .bind(input.custom_type)
    .bind(input.description_zh)
    .bind(input.description_en)
    .bind(input.tag_zh)
    .bind(input.tag_en)
    .bind(input.is_public)
    .bind(input.duration_hours)
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

pub async fn update_host_space(
    pool: &PgPool,
    space_id: uuid::Uuid,
    input: UpdateSpaceInput,
) -> Result<SpaceSummary, sqlx::Error> {
    let row = sqlx::query(
        r#"
        UPDATE spaces
        SET name_zh = $2,
            name_en = $3,
            country = $4,
            province = $5,
            city = $6,
            district = $7,
            spot_name = $8,
            address_line = $9,
            lat = $10,
            lng = $11,
            is_public = $12,
            updated_at = now()
        WHERE id = $1
        RETURNING id, name_zh, name_en, space_type::text AS space_type, country, province, city, district, spot_name, address_line,
                  lat, lng, is_public, status::text AS status, expires_at, online_count
        "#,
    )
    .bind(space_id)
    .bind(input.name_zh)
    .bind(input.name_en)
    .bind(input.country)
    .bind(input.province)
    .bind(input.city)
    .bind(input.district)
    .bind(input.spot_name)
    .bind(input.address_line)
    .bind(input.lat)
    .bind(input.lng)
    .bind(input.is_public)
    .fetch_one(pool)
    .await?;

    row_to_space_summary(row)
}

pub async fn set_space_status(
    pool: &PgPool,
    space_id: uuid::Uuid,
    status: SpaceStatus,
) -> Result<SpaceSummary, sqlx::Error> {
    let row = sqlx::query(
        r#"
        UPDATE spaces
        SET status = $2::space_status,
            closed_at = CASE WHEN $2 = 'closed' THEN now() ELSE closed_at END,
            updated_at = now()
        WHERE id = $1
        RETURNING id, name_zh, name_en, space_type::text AS space_type, country, province, city, district, spot_name, address_line,
                  lat, lng, is_public, status::text AS status, expires_at, online_count
        "#,
    )
    .bind(space_id)
    .bind(status_to_db(status))
    .fetch_one(pool)
    .await?;

    row_to_space_summary(row)
}

pub async fn rotate_space_password(
    pool: &PgPool,
    space_id: uuid::Uuid,
    password_hash: String,
) -> Result<i32, sqlx::Error> {
    let row = sqlx::query(
        r#"
        UPDATE spaces
        SET password_hash = $2,
            password_version = password_version + 1,
            updated_at = now()
        WHERE id = $1
        RETURNING password_version
        "#,
    )
    .bind(space_id)
    .bind(password_hash)
    .fetch_one(pool)
    .await?;

    row.try_get("password_version")
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
        SET resident = true,
            resident_days = $2,
            expires_at = now() + make_interval(days => $2),
            updated_at = now()
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
        country: row.try_get("country")?,
        province: row.try_get("province")?,
        city: row.try_get("city")?,
        district: row.try_get("district")?,
        spot_name: row.try_get("spot_name")?,
        address_line: row.try_get("address_line")?,
        lat: row.try_get("lat")?,
        lng: row.try_get("lng")?,
        is_public: row.try_get("is_public")?,
        status: status_from_db(row.try_get::<String, _>("status")?.as_str()),
        expires_at: row.try_get("expires_at")?,
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

fn status_to_db(status: SpaceStatus) -> &'static str {
    match status {
        SpaceStatus::Active => "active",
        SpaceStatus::Expired => "expired",
        SpaceStatus::Closed => "closed",
        SpaceStatus::Archived => "archived",
        SpaceStatus::Template => "template",
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
                country: Some("中国".to_string()),
                province: "上海市".to_string(),
                city: "上海市".to_string(),
                district: Some("黄浦区".to_string()),
                spot_name: Some("外滩".to_string()),
                address_line: Some("中山东一路".to_string()),
                lat: 31.23,
                lng: 121.47,
                space_type: SpaceType::Scenic,
                custom_type: None,
                description_zh: None,
                description_en: None,
                tag_zh: None,
                tag_en: None,
                is_public: true,
                duration_hours: 24,
                password_hash: "hash".to_string(),
                host_user_id: user.id,
            },
        )
        .await
        .expect("space");

        assert_eq!(space.host_user_id, Some(user.id));

        sqlx::query("DELETE FROM spaces WHERE id = $1")
            .bind(space.id)
            .execute(&pool)
            .await
            .expect("cleanup space");
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user.id)
            .execute(&pool)
            .await
            .expect("cleanup user");
    }
}
