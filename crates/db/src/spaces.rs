use instant_domain::admin::ResidentApplication;
use instant_domain::spaces::{SpaceMember, SpaceStatus, SpaceSummary, SpaceType};
use sqlx::{PgPool, Row};

#[derive(Debug, Default, Clone)]
pub struct SpaceFilter {
    pub q: Option<String>,
    pub space_type: Option<SpaceType>,
    pub country: Option<String>,
    pub province: Option<String>,
    pub city: Option<String>,
    pub district: Option<String>,
    pub spot_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PaginatedSpaces {
    pub items: Vec<SpaceSummary>,
    pub total: i64,
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
    pub custom_type: Option<String>,
    pub description_zh: Option<String>,
    pub description_en: Option<String>,
    pub tag_zh: Option<String>,
    pub tag_en: Option<String>,
    pub host_bio_zh: Option<String>,
    pub host_bio_en: Option<String>,
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
               lat, lng, is_public, status::text AS status, expires_at, online_count, home_weight
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
        ORDER BY home_weight DESC, (CASE WHEN resident THEN 10 ELSE 0 END) DESC,
                 online_count DESC, created_at DESC
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

pub async fn list_home_spaces_page(
    pool: &PgPool,
    filter: SpaceFilter,
    limit: i64,
    offset: i64,
) -> Result<PaginatedSpaces, sqlx::Error> {
    let tokens = filter.q.as_ref().map(|value| {
        value
            .split_whitespace()
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>()
    });
    let tokens = tokens.filter(|items| !items.is_empty());
    let space_type = filter.space_type.map(space_type_to_db);
    let country = filter.country;
    let province = filter.province;
    let city = filter.city;
    let district = filter.district;
    let spot_name = filter.spot_name;

    const FILTER: &str = r#"
        WHERE status IN ('active', 'expired')
          AND (
            $1::text[] IS NULL
            OR NOT EXISTS (
              SELECT 1 FROM unnest($1::text[]) tok
              WHERE concat_ws(
                ' ', name_zh, name_en, country, province, city, district,
                spot_name, address_line, custom_type, description_zh, description_en,
                tag_zh, tag_en
              ) NOT ILIKE '%' || tok || '%'
            )
          )
          AND ($2::text IS NULL OR space_type::text = $2)
          AND ($3::text IS NULL OR country = $3)
          AND ($4::text IS NULL OR province = $4)
          AND ($5::text IS NULL OR city = $5)
          AND ($6::text IS NULL OR district = $6)
          AND ($7::text IS NULL OR spot_name = $7)
    "#;

    let total: i64 = sqlx::query_scalar(&format!("SELECT count(*)::bigint FROM spaces {FILTER}"))
        .bind(tokens.clone())
        .bind(space_type.clone())
        .bind(country.clone())
        .bind(province.clone())
        .bind(city.clone())
        .bind(district.clone())
        .bind(spot_name.clone())
        .fetch_one(pool)
        .await?;

    let rows = sqlx::query(&format!(
        r#"
        SELECT id, name_zh, name_en, space_type::text AS space_type, country, province, city, district, spot_name, address_line,
               lat, lng, is_public, status::text AS status, expires_at, online_count, home_weight
        FROM spaces
        {FILTER}
        ORDER BY home_weight DESC, (CASE WHEN resident THEN 10 ELSE 0 END) DESC,
                 online_count DESC, created_at DESC, id DESC
        LIMIT $8 OFFSET $9
        "#
    ))
    .bind(tokens)
    .bind(space_type)
    .bind(country)
    .bind(province)
    .bind(city)
    .bind(district)
    .bind(spot_name)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(PaginatedSpaces {
        items: rows
            .into_iter()
            .map(row_to_space_summary)
            .collect::<Result<Vec<_>, _>>()?,
        total,
    })
}

/// Filter options are derived from currently discoverable Spaces. This avoids
/// offering countries or cities that lead to an empty directory.
pub async fn discoverable_space_countries(pool: &PgPool) -> Result<Vec<String>, sqlx::Error> {
    distinct_space_values(
        pool,
        "SELECT DISTINCT country AS value FROM spaces WHERE status IN ('active','expired') AND country IS NOT NULL AND country <> '' ORDER BY country",
        None,
        None,
    )
    .await
}

pub async fn discoverable_space_provinces(
    pool: &PgPool,
    country: Option<String>,
) -> Result<Vec<String>, sqlx::Error> {
    distinct_space_values(
        pool,
        "SELECT DISTINCT province AS value FROM spaces WHERE status IN ('active','expired') AND ($1::text IS NULL OR country = $1) AND province IS NOT NULL AND province <> '' ORDER BY province",
        country,
        None,
    )
    .await
}

pub async fn discoverable_space_cities(
    pool: &PgPool,
    country: Option<String>,
    province: Option<String>,
) -> Result<Vec<String>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT DISTINCT city AS value FROM spaces WHERE status IN ('active','expired') AND ($1::text IS NULL OR country = $1) AND ($2::text IS NULL OR province = $2) AND city IS NOT NULL AND city <> '' ORDER BY city",
    )
    .bind(country)
    .bind(province)
    .fetch_all(pool)
    .await?;
    rows.into_iter().map(|row| row.try_get("value")).collect()
}

pub async fn discoverable_space_districts(
    pool: &PgPool,
    country: Option<String>,
    province: Option<String>,
    city: Option<String>,
) -> Result<Vec<String>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT DISTINCT district AS value FROM spaces WHERE status IN ('active','expired') AND ($1::text IS NULL OR country = $1) AND ($2::text IS NULL OR province = $2) AND ($3::text IS NULL OR city = $3) AND district IS NOT NULL AND district <> '' ORDER BY district",
    )
    .bind(country)
    .bind(province)
    .bind(city)
    .fetch_all(pool)
    .await?;
    rows.into_iter().map(|row| row.try_get("value")).collect()
}

pub async fn discoverable_space_spots(
    pool: &PgPool,
    country: Option<String>,
    province: Option<String>,
    city: Option<String>,
    district: Option<String>,
) -> Result<Vec<String>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT DISTINCT spot_name AS value FROM spaces WHERE status IN ('active','expired') AND ($1::text IS NULL OR country = $1) AND ($2::text IS NULL OR province = $2) AND ($3::text IS NULL OR city = $3) AND ($4::text IS NULL OR district = $4) AND spot_name IS NOT NULL AND spot_name <> '' ORDER BY spot_name",
    )
    .bind(country)
    .bind(province)
    .bind(city)
    .bind(district)
    .fetch_all(pool)
    .await?;
    rows.into_iter().map(|row| row.try_get("value")).collect()
}

async fn distinct_space_values(
    pool: &PgPool,
    sql: &str,
    first: Option<String>,
    second: Option<String>,
) -> Result<Vec<String>, sqlx::Error> {
    let mut query = sqlx::query(sql);
    if sql.contains("$1") {
        query = query.bind(first);
    }
    if sql.contains("$2") {
        query = query.bind(second);
    }
    let rows = query.fetch_all(pool).await?;
    rows.into_iter().map(|row| row.try_get("value")).collect()
}

pub async fn list_host_spaces(
    pool: &PgPool,
    host_user_id: uuid::Uuid,
) -> Result<Vec<SpaceSummary>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT id, name_zh, name_en, space_type::text AS space_type, country, province, city, district, spot_name, address_line,
               lat, lng, is_public, status::text AS status, expires_at, online_count, home_weight
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
               lat, lng, is_public, status::text AS status, expires_at, online_count, home_weight
        FROM spaces
        WHERE status <> 'archived'
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(row_to_space_summary).collect()
}

/// Admin-only: search and paginate spaces without loading the full catalog.
/// `status = "managed"` means every operational space except soft-deleted rows.
pub async fn list_admin_spaces_page(
    pool: &PgPool,
    q: Option<String>,
    status: Option<String>,
    space_type: Option<SpaceType>,
    limit: i64,
    offset: i64,
) -> Result<PaginatedSpaces, sqlx::Error> {
    let space_type = space_type.map(space_type_to_db);
    let total: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::bigint
        FROM spaces
        WHERE (
            $1::text IS NULL
            OR name_zh ILIKE '%' || $1 || '%'
            OR name_en ILIKE '%' || $1 || '%'
            OR country ILIKE '%' || $1 || '%'
            OR province ILIKE '%' || $1 || '%'
            OR city ILIKE '%' || $1 || '%'
            OR district ILIKE '%' || $1 || '%'
            OR spot_name ILIKE '%' || $1 || '%'
            OR address_line ILIKE '%' || $1 || '%'
          )
          AND (
            $2::text IS NULL
            OR ($2 = 'managed' AND status <> 'archived')
            OR ($2 <> 'managed' AND status::text = $2)
          )
          AND ($3::text IS NULL OR space_type::text = $3)
        "#,
    )
    .bind(q.clone())
    .bind(status.clone())
    .bind(space_type.clone())
    .fetch_one(pool)
    .await?;

    let rows = sqlx::query(
        r#"
        SELECT id, name_zh, name_en, space_type::text AS space_type, country, province, city, district, spot_name, address_line,
               lat, lng, is_public, status::text AS status, expires_at, online_count, home_weight
        FROM spaces
        WHERE (
            $1::text IS NULL
            OR name_zh ILIKE '%' || $1 || '%'
            OR name_en ILIKE '%' || $1 || '%'
            OR country ILIKE '%' || $1 || '%'
            OR province ILIKE '%' || $1 || '%'
            OR city ILIKE '%' || $1 || '%'
            OR district ILIKE '%' || $1 || '%'
            OR spot_name ILIKE '%' || $1 || '%'
            OR address_line ILIKE '%' || $1 || '%'
          )
          AND (
            $2::text IS NULL
            OR ($2 = 'managed' AND status <> 'archived')
            OR ($2 <> 'managed' AND status::text = $2)
          )
          AND ($3::text IS NULL OR space_type::text = $3)
        ORDER BY updated_at DESC, created_at DESC, id DESC
        LIMIT $4 OFFSET $5
        "#,
    )
    .bind(q)
    .bind(status)
    .bind(space_type)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(PaginatedSpaces {
        items: rows
            .into_iter()
            .map(row_to_space_summary)
            .collect::<Result<Vec<_>, _>>()?,
        total,
    })
}

/// Admin-only: list every space regardless of status (includes archived),
/// so operators can audit and restore soft-deleted spaces.
pub async fn list_all_spaces_admin(pool: &PgPool) -> Result<Vec<SpaceSummary>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT id, name_zh, name_en, space_type::text AS space_type, country, province, city, district, spot_name, address_line,
               lat, lng, is_public, status::text AS status, expires_at, online_count, home_weight
        FROM spaces
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
               lat, lng, is_public, status::text AS status, expires_at, online_count, home_weight
        FROM spaces
        WHERE id = $1
          AND status <> 'archived'
        "#,
    )
    .bind(space_id)
    .fetch_optional(pool)
    .await?;

    row.map(|row| row_to_space_summary_ref(&row)).transpose()
}

pub async fn get_space_detail(
    pool: &PgPool,
    space_id: uuid::Uuid,
) -> Result<Option<instant_domain::spaces::SpaceDetail>, sqlx::Error> {
    // One query for the whole detail view: the summary columns plus the extra
    // fields the card wall needs (description, tags, community, custom type),
    // and the host's display name via a left join so unclaimed Spaces still work.
    let row = sqlx::query(
        r#"
        SELECT s.id, s.name_zh, s.name_en, s.space_type::text AS space_type, s.country, s.province, s.city, s.district,
               s.spot_name, s.address_line, s.lat, s.lng, s.is_public, s.status::text AS status, s.expires_at, s.online_count, s.home_weight,
               s.description_zh, s.description_en, s.tag_zh, s.tag_en, s.discord_group, s.qq_group, s.password_version,
               s.custom_type, s.host_bio_zh, s.host_bio_en, s.created_at, u.name AS host_name
        FROM spaces s
        LEFT JOIN users u ON u.id = s.host_user_id
        WHERE s.id = $1
          AND s.status <> 'archived'
        "#,
    )
    .bind(space_id)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let summary = row_to_space_summary_ref(&row)?;
    let created_at: Option<time::OffsetDateTime> = row.try_get("created_at")?;
    let created_at = created_at.and_then(|dt| {
        dt.format(&time::format_description::well_known::Rfc3339)
            .ok()
    });

    Ok(Some(instant_domain::spaces::SpaceDetail {
        summary,
        description_zh: row.try_get("description_zh")?,
        description_en: row.try_get("description_en")?,
        tag_zh: row.try_get("tag_zh")?,
        tag_en: row.try_get("tag_en")?,
        discord_group: row.try_get("discord_group")?,
        qq_group: row.try_get("qq_group")?,
        password_version: row.try_get("password_version")?,
        custom_type: row.try_get("custom_type")?,
        host_bio_zh: row.try_get("host_bio_zh")?,
        host_bio_en: row.try_get("host_bio_en")?,
        host_name: row.try_get("host_name")?,
        created_at,
    }))
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
            custom_type = $13,
            description_zh = $14,
            description_en = $15,
            tag_zh = $16,
            tag_en = $17,
            host_bio_zh = $18,
            host_bio_en = $19,
            updated_at = now()
        WHERE id = $1
        RETURNING id, name_zh, name_en, space_type::text AS space_type, country, province, city, district, spot_name, address_line,
                  lat, lng, is_public, status::text AS status, expires_at, online_count, home_weight
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
    .bind(input.custom_type)
    .bind(input.description_zh)
    .bind(input.description_en)
    .bind(input.tag_zh)
    .bind(input.tag_en)
    .bind(input.host_bio_zh)
    .bind(input.host_bio_en)
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
                  lat, lng, is_public, status::text AS status, expires_at, online_count, home_weight
        "#,
    )
    .bind(space_id)
    .bind(status_to_db(status))
    .fetch_one(pool)
    .await?;

    row_to_space_summary(row)
}

/// An operator-controlled editorial value used to decide which real Spaces
/// appear on the homepage. It is intentionally separate from views or likes:
/// the product must not pretend popularity it has not measured.
pub async fn set_home_weight(
    pool: &PgPool,
    space_id: uuid::Uuid,
    home_weight: i32,
) -> Result<SpaceSummary, sqlx::Error> {
    let row = sqlx::query(
        r#"
        UPDATE spaces
        SET home_weight = $2,
            updated_at = now()
        WHERE id = $1
        RETURNING id, name_zh, name_en, space_type::text AS space_type, country, province, city, district, spot_name, address_line,
                  lat, lng, is_public, status::text AS status, expires_at, online_count, home_weight
        "#,
    )
    .bind(space_id)
    .bind(home_weight)
    .fetch_one(pool)
    .await?;

    row_to_space_summary(row)
}

/// A deliberately small homepage selection. If no operator has assigned a
/// weight yet, the secondary ordering still supplies a useful first view while
/// exposing no fake "hotness" numbers to visitors.
pub async fn list_featured_home_spaces(
    pool: &PgPool,
    limit: i64,
) -> Result<Vec<SpaceSummary>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT id, name_zh, name_en, space_type::text AS space_type, country, province, city, district, spot_name, address_line,
               lat, lng, is_public, status::text AS status, expires_at, online_count, home_weight
        FROM spaces
        WHERE status IN ('active', 'expired')
        ORDER BY home_weight DESC, (CASE WHEN resident THEN 10 ELSE 0 END) DESC,
                 online_count DESC, created_at DESC, id DESC
        LIMIT $1
        "#,
    )
    .bind(limit.clamp(1, 12))
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(row_to_space_summary).collect()
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

/// Admin-only: list pending resident applications (applied but not yet approved),
/// joined with the space name and host contact for review.
pub async fn list_resident_applications(
    pool: &PgPool,
) -> Result<Vec<ResidentApplication>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT s.id AS space_id, s.name_zh, s.name_en, s.host_user_id,
               u.email AS host_email, s.resident_days
        FROM spaces s
        LEFT JOIN users u ON u.id = s.host_user_id
        WHERE s.resident_apply_at IS NOT NULL AND s.resident = false
        ORDER BY s.resident_apply_at ASC
        "#,
    )
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(ResidentApplication {
                space_id: row.try_get("space_id")?,
                name_zh: row.try_get("name_zh")?,
                name_en: row.try_get("name_en")?,
                host_user_id: row.try_get("host_user_id")?,
                host_email: row.try_get("host_email")?,
                resident_days: row.try_get("resident_days")?,
            })
        })
        .collect()
}

/// Admin-only: reject a resident application by clearing the applied marker,
/// leaving the space as a normal (non-resident) space.
pub async fn reject_resident_application(
    pool: &PgPool,
    space_id: uuid::Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE spaces
        SET resident_apply_at = NULL, updated_at = now()
        WHERE id = $1
        "#,
    )
    .bind(space_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// A logged-in user applies to become the host of an unclaimed Space. Re-applying
/// after a rejection resets the row to pending; a duplicate pending apply is a
/// no-op. Returns true when a claim now sits pending.
pub async fn apply_host_claim(
    pool: &PgPool,
    space_id: uuid::Uuid,
    user_id: uuid::Uuid,
    message: Option<String>,
) -> Result<bool, sqlx::Error> {
    // Guard: only Spaces that are still unclaimed accept applications.
    let taken: Option<uuid::Uuid> = sqlx::query("SELECT host_user_id FROM spaces WHERE id = $1")
        .bind(space_id)
        .fetch_optional(pool)
        .await?
        .and_then(|row| row.try_get("host_user_id").ok().flatten());
    if taken.is_some() {
        return Ok(false);
    }

    let result = sqlx::query(
        r#"
        INSERT INTO space_host_claims (space_id, user_id, message, status)
        VALUES ($1, $2, $3, 'pending')
        ON CONFLICT (space_id, user_id)
        DO UPDATE SET status = 'pending',
                      message = EXCLUDED.message,
                      created_at = now(),
                      decided_at = NULL
        "#,
    )
    .bind(space_id)
    .bind(user_id)
    .bind(message)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Whether this user already has a pending claim on this Space — used to show
/// the "application submitted" state instead of the apply button.
pub async fn host_claim_status(
    pool: &PgPool,
    space_id: uuid::Uuid,
    user_id: uuid::Uuid,
) -> Result<Option<String>, sqlx::Error> {
    let row =
        sqlx::query("SELECT status FROM space_host_claims WHERE space_id = $1 AND user_id = $2")
            .bind(space_id)
            .bind(user_id)
            .fetch_optional(pool)
            .await?;

    row.map(|row| row.try_get("status")).transpose()
}

/// Admin-only: the review queue of pending host claims, oldest first, joined
/// with the Space name and the applicant's contact.
pub async fn list_host_claims(
    pool: &PgPool,
) -> Result<Vec<instant_domain::admin::HostClaimApplication>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT c.id AS claim_id, c.space_id, s.name_zh, s.name_en,
               c.user_id AS applicant_id, u.email AS applicant_email, u.name AS applicant_name,
               c.message, c.created_at
        FROM space_host_claims c
        JOIN spaces s ON s.id = c.space_id
        LEFT JOIN users u ON u.id = c.user_id
        WHERE c.status = 'pending'
        ORDER BY c.created_at ASC
        "#,
    )
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            let created_at: time::OffsetDateTime = row.try_get("created_at")?;
            let created_at = created_at
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default();
            Ok(instant_domain::admin::HostClaimApplication {
                claim_id: row.try_get("claim_id")?,
                space_id: row.try_get("space_id")?,
                name_zh: row.try_get("name_zh")?,
                name_en: row.try_get("name_en")?,
                applicant_id: row.try_get("applicant_id")?,
                applicant_email: row.try_get("applicant_email")?,
                applicant_name: row.try_get("applicant_name")?,
                message: row.try_get("message")?,
                created_at,
            })
        })
        .collect()
}

/// Admin-only: approve a claim. Assigns the Space's host, marks this claim
/// approved and every other pending claim on the same Space rejected. Returns
/// the applicant id that became host, or None if the claim was already decided.
pub async fn approve_host_claim(
    pool: &PgPool,
    claim_id: uuid::Uuid,
) -> Result<Option<(uuid::Uuid, uuid::Uuid)>, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let row = sqlx::query(
        "SELECT space_id, user_id FROM space_host_claims WHERE id = $1 AND status = 'pending' FOR UPDATE",
    )
    .bind(claim_id)
    .fetch_optional(&mut *tx)
    .await?;

    let Some(row) = row else {
        tx.rollback().await?;
        return Ok(None);
    };
    let space_id: uuid::Uuid = row.try_get("space_id")?;
    let user_id: uuid::Uuid = row.try_get("user_id")?;

    sqlx::query("UPDATE spaces SET host_user_id = $2, updated_at = now() WHERE id = $1")
        .bind(space_id)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query(
        "UPDATE space_host_claims SET status = 'approved', decided_at = now() WHERE id = $1",
    )
    .bind(claim_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "UPDATE space_host_claims SET status = 'rejected', decided_at = now() WHERE space_id = $1 AND id <> $2 AND status = 'pending'",
    )
    .bind(space_id)
    .bind(claim_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(Some((space_id, user_id)))
}

/// Admin-only: reject a single pending claim.
pub async fn reject_host_claim(pool: &PgPool, claim_id: uuid::Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE space_host_claims SET status = 'rejected', decided_at = now() WHERE id = $1 AND status = 'pending'",
    )
    .bind(claim_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

fn row_to_space_summary(row: sqlx::postgres::PgRow) -> Result<SpaceSummary, sqlx::Error> {
    row_to_space_summary_ref(&row)
}

fn row_to_space_summary_ref(row: &sqlx::postgres::PgRow) -> Result<SpaceSummary, sqlx::Error> {
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
        home_weight: row.try_get("home_weight")?,
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

/// Permanently delete a Space and everything attached to it. Related rows
/// (guides, chat messages, traces, capsules, host claims, resident
/// applications) are removed by their ON DELETE CASCADE / SET NULL foreign
/// keys; the agent API is the current caller.
pub async fn delete_space(pool: &PgPool, space_id: uuid::Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM spaces WHERE id = $1")
        .bind(space_id)
        .execute(pool)
        .await?;
    Ok(())
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

/// A person admitted to a Space with an explicit role. Roles are a coarse
/// trust ladder: `member` (participant) and `host` (can manage members).


/// Everyone admitted to a Space, newest first. Joining through a session
/// records `member`; the Space manager can raise or remove roles.
pub async fn list_space_members(
    pool: &PgPool,
    space_id: uuid::Uuid,
) -> Result<Vec<SpaceMember>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT m.id, m.space_id, m.user_id, m.role, m.created_at,
               u.email, u.name AS display_name
        FROM space_members m
        JOIN users u ON u.id = m.user_id
        WHERE m.space_id = $1
        ORDER BY m.created_at DESC
        "#,
    )
    .bind(space_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(row_to_space_member).collect()
}

/// Admit a user to a Space or change their role. Returns the updated member.
pub async fn set_space_member(
    pool: &PgPool,
    space_id: uuid::Uuid,
    user_id: uuid::Uuid,
    role: &str,
) -> Result<SpaceMember, sqlx::Error> {
    let role = if role == "host" { "host" } else { "member" };
    let row = sqlx::query(
        r#"
        WITH upsert AS (
          INSERT INTO space_members (space_id, user_id, role)
          VALUES ($1, $2, $3)
          ON CONFLICT (space_id, user_id)
          DO UPDATE SET role = EXCLUDED.role
          RETURNING id, space_id, user_id, role, created_at
        )
        SELECT m.id, m.space_id, m.user_id, m.role, m.created_at,
               u.email, u.name AS display_name
        FROM upsert m
        JOIN users u ON u.id = m.user_id
        "#,
    )
    .bind(space_id)
    .bind(user_id)
    .bind(role)
    .fetch_one(pool)
    .await?;

    row_to_space_member(row)
}

fn row_to_space_member(row: sqlx::postgres::PgRow) -> Result<SpaceMember, sqlx::Error> {
    Ok(SpaceMember {
        id: row.try_get("id")?,
        space_id: row.try_get("space_id")?,
        user_id: row.try_get("user_id")?,
        role: row.try_get("role")?,
        created_at: row
            .try_get::<time::OffsetDateTime, _>("created_at")?
            .to_string(),
        email: row.try_get("email")?,
        display_name: row.try_get("display_name")?,
    })
}

/// Remove a member from a Space. Returns true when a row was actually deleted.
pub async fn remove_space_member(
    pool: &PgPool,
    space_id: uuid::Uuid,
    user_id: uuid::Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM space_members WHERE space_id = $1 AND user_id = $2")
        .bind(space_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Resolve a user by exact email; used by the manager UI to admit people.
pub async fn find_user_id_by_email(
    pool: &PgPool,
    email: &str,
) -> Result<Option<uuid::Uuid>, sqlx::Error> {
    sqlx::query_scalar("SELECT id FROM users WHERE lower(email) = lower($1)")
        .bind(email)
        .fetch_optional(pool)
        .await
}
