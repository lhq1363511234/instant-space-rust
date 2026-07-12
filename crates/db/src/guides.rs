use instant_domain::guides::{GuideDetail, GuideSection, GuideStatus, GuideSummary};
use serde_json::Value;
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CreateGuideDraftInput {
    pub title_zh: String,
    pub title_en: Option<String>,
    pub summary_zh: Option<String>,
    pub summary_en: Option<String>,
    pub content_zh: Option<String>,
    pub content_en: Option<String>,
    pub guide_type: String,
    pub category: Option<String>,
    pub province: String,
    pub city: String,
    pub district: Option<String>,
    pub spot_name: Option<String>,
    pub author_id: Uuid,
    pub author_name: Option<String>,
    pub space_id: Option<Uuid>,
    pub cover_image_url: Option<String>,
    pub images: Vec<String>,
    pub sections: Vec<GuideSection>,
    pub status: GuideStatus,
    pub featured: bool,
}

#[derive(Debug, Clone)]
pub struct UpdateGuideInput {
    pub title_zh: String,
    pub title_en: Option<String>,
    pub summary_zh: Option<String>,
    pub summary_en: Option<String>,
    pub content_zh: Option<String>,
    pub content_en: Option<String>,
    pub guide_type: String,
    pub category: Option<String>,
    pub province: String,
    pub city: String,
    pub district: Option<String>,
    pub spot_name: Option<String>,
    pub space_id: Option<Uuid>,
    pub cover_image_url: Option<String>,
    pub images: Vec<String>,
    pub sections: Vec<GuideSection>,
    pub status: GuideStatus,
    pub featured: bool,
}

pub async fn list_published_guides(
    pool: &PgPool,
    province: Option<String>,
    city: Option<String>,
    district: Option<String>,
    spot_name: Option<String>,
) -> Result<Vec<GuideSummary>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT id, title_zh, title_en, province, city, district, spot_name,
               status::text AS status, featured, author_id, space_id
        FROM guides
        WHERE status = 'published'
          AND ($1::text IS NULL OR province = $1)
          AND ($2::text IS NULL OR city = $2)
          AND ($3::text IS NULL OR district = $3)
          AND ($4::text IS NULL OR spot_name = $4)
        ORDER BY featured DESC, updated_at DESC, created_at DESC
        "#,
    )
    .bind(province)
    .bind(city)
    .bind(district)
    .bind(spot_name)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| row_to_guide_summary(row))
        .collect()
}

pub async fn get_published_guide(
    pool: &PgPool,
    guide_id: Uuid,
) -> Result<Option<GuideDetail>, sqlx::Error> {
    let Some(row) = sqlx::query(
        r#"
        SELECT id, title_zh, title_en, summary_zh, summary_en, content_zh, content_en,
               guide_type, category, province, city, district, spot_name,
               status::text AS status, featured, author_id, author_name, space_id, cover_image_url,
               images, sections, created_at, updated_at
        FROM guides
        WHERE id = $1
          AND status = 'published'
        "#,
    )
    .bind(guide_id)
    .fetch_optional(pool)
    .await?
    else {
        return Ok(None);
    };

    let status = guide_status_from_db(row.try_get::<String, _>("status")?.as_str());
    let images = parse_images(row.try_get("images")?);
    let sections = parse_sections(row.try_get("sections")?);

    Ok(Some(GuideDetail {
        id: row.try_get("id")?,
        title_zh: row.try_get("title_zh")?,
        title_en: row.try_get("title_en")?,
        summary_zh: row.try_get("summary_zh")?,
        summary_en: row.try_get("summary_en")?,
        content_zh: row.try_get("content_zh")?,
        content_en: row.try_get("content_en")?,
        guide_type: row.try_get("guide_type")?,
        category: row.try_get("category")?,
        province: row.try_get("province")?,
        city: row.try_get("city")?,
        district: row.try_get("district")?,
        spot_name: row.try_get("spot_name")?,
        status,
        featured: row.try_get("featured")?,
        author_id: row.try_get("author_id")?,
        author_name: row.try_get("author_name")?,
        space_id: row.try_get("space_id")?,
        can_edit: false,
        cover_image_url: row.try_get("cover_image_url")?,
        images,
        sections,
        created_at: row
            .try_get::<time::OffsetDateTime, _>("created_at")?
            .to_string(),
        updated_at: row
            .try_get::<time::OffsetDateTime, _>("updated_at")?
            .to_string(),
    }))
}

pub async fn get_guide(pool: &PgPool, guide_id: Uuid) -> Result<Option<GuideDetail>, sqlx::Error> {
    let Some(row) = sqlx::query(
        r#"
        SELECT id, title_zh, title_en, summary_zh, summary_en, content_zh, content_en,
               guide_type, category, province, city, district, spot_name,
               status::text AS status, featured, author_id, author_name, space_id, cover_image_url,
               images, sections, created_at, updated_at
        FROM guides
        WHERE id = $1
        "#,
    )
    .bind(guide_id)
    .fetch_optional(pool)
    .await?
    else {
        return Ok(None);
    };

    row_to_guide_detail(row).map(Some)
}

pub async fn list_published_guides_by_space(
    pool: &PgPool,
    space_id: Uuid,
) -> Result<Vec<GuideSummary>, sqlx::Error> {
    list_guides_by_space(pool, space_id, false).await
}

/// List guides bound to a space.
/// - public mode (`include_unpublished=false`): only published
/// - manage mode (`include_unpublished=true`): draft + published + archived
pub async fn list_guides_by_space(
    pool: &PgPool,
    space_id: Uuid,
    include_unpublished: bool,
) -> Result<Vec<GuideSummary>, sqlx::Error> {
    let rows = if include_unpublished {
        sqlx::query(
            r#"
            SELECT id, title_zh, title_en, province, city, district, spot_name,
                   status::text AS status, featured, author_id, space_id
            FROM guides
            WHERE space_id = $1
              AND status IN ('draft', 'published', 'archived')
            ORDER BY
              CASE status
                WHEN 'published' THEN 0
                WHEN 'draft' THEN 1
                ELSE 2
              END,
              featured DESC,
              updated_at DESC,
              created_at DESC
            "#,
        )
        .bind(space_id)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query(
            r#"
            SELECT id, title_zh, title_en, province, city, district, spot_name,
                   status::text AS status, featured, author_id, space_id
            FROM guides
            WHERE space_id = $1
              AND status = 'published'
            ORDER BY featured DESC, updated_at DESC, created_at DESC
            "#,
        )
        .bind(space_id)
        .fetch_all(pool)
        .await?
    };

    rows.into_iter()
        .map(|row| row_to_guide_summary(row))
        .collect()
}

fn row_to_guide_summary(row: sqlx::postgres::PgRow) -> Result<GuideSummary, sqlx::Error> {
    Ok(GuideSummary {
        id: row.try_get("id")?,
        title_zh: row.try_get("title_zh")?,
        title_en: row.try_get("title_en")?,
        province: row.try_get("province")?,
        city: row.try_get("city")?,
        district: row.try_get("district")?,
        spot_name: row.try_get("spot_name")?,
        status: guide_status_from_db(row.try_get::<String, _>("status")?.as_str()),
        featured: row.try_get("featured")?,
        author_id: row.try_get("author_id")?,
        space_id: row.try_get("space_id")?,
        can_edit: false,
    })
}

fn row_to_guide_detail(row: sqlx::postgres::PgRow) -> Result<GuideDetail, sqlx::Error> {
    let status = guide_status_from_db(row.try_get::<String, _>("status")?.as_str());
    let images = parse_images(row.try_get("images")?);
    let sections = parse_sections(row.try_get("sections")?);

    Ok(GuideDetail {
        id: row.try_get("id")?,
        title_zh: row.try_get("title_zh")?,
        title_en: row.try_get("title_en")?,
        summary_zh: row.try_get("summary_zh")?,
        summary_en: row.try_get("summary_en")?,
        content_zh: row.try_get("content_zh")?,
        content_en: row.try_get("content_en")?,
        guide_type: row.try_get("guide_type")?,
        category: row.try_get("category")?,
        province: row.try_get("province")?,
        city: row.try_get("city")?,
        district: row.try_get("district")?,
        spot_name: row.try_get("spot_name")?,
        status,
        featured: row.try_get("featured")?,
        author_id: row.try_get("author_id")?,
        author_name: row.try_get("author_name")?,
        space_id: row.try_get("space_id")?,
        can_edit: false,
        cover_image_url: row.try_get("cover_image_url")?,
        images,
        sections,
        created_at: row
            .try_get::<time::OffsetDateTime, _>("created_at")?
            .to_string(),
        updated_at: row
            .try_get::<time::OffsetDateTime, _>("updated_at")?
            .to_string(),
    })
}

pub async fn create_guide_draft(
    pool: &PgPool,
    input: CreateGuideDraftInput,
) -> Result<GuideSummary, sqlx::Error> {
    let images_json = serde_json::to_value(input.images).unwrap_or(Value::Array(Vec::new()));
    let sections_json = serde_json::to_value(input.sections).unwrap_or(Value::Array(Vec::new()));
    let status = guide_status_to_db(&input.status);
    let row = sqlx::query(
        r#"
        INSERT INTO guides (
            title_zh, title_en, summary_zh, summary_en, content_zh, content_en,
            guide_type, category, province, city, district, spot_name,
            status, featured, author_id, author_name, space_id, cover_image_url, images, sections
        )
        VALUES (
            $1, $2, $3, $4, $5, $6,
            $7, $8, $9, $10, $11, $12,
            $13::guide_status, $14, $15, $16, $17, $18, $19, $20
        )
        RETURNING id, title_zh, title_en, province, city, district, spot_name,
                  status::text AS status, featured, author_id, space_id
        "#,
    )
    .bind(input.title_zh)
    .bind(input.title_en)
    .bind(input.summary_zh)
    .bind(input.summary_en)
    .bind(input.content_zh)
    .bind(input.content_en)
    .bind(input.guide_type)
    .bind(input.category)
    .bind(input.province)
    .bind(input.city)
    .bind(input.district)
    .bind(input.spot_name)
    .bind(status)
    .bind(input.featured)
    .bind(input.author_id)
    .bind(input.author_name)
    .bind(input.space_id)
    .bind(input.cover_image_url)
    .bind(images_json)
    .bind(sections_json)
    .fetch_one(pool)
    .await?;

    row_to_guide_summary(row)
}

pub async fn update_guide(
    pool: &PgPool,
    guide_id: Uuid,
    input: UpdateGuideInput,
) -> Result<GuideSummary, sqlx::Error> {
    let images_json = serde_json::to_value(input.images).unwrap_or(Value::Array(Vec::new()));
    let sections_json = serde_json::to_value(input.sections).unwrap_or(Value::Array(Vec::new()));
    let status = guide_status_to_db(&input.status);
    let row = sqlx::query(
        r#"
        UPDATE guides
        SET title_zh = $2,
            title_en = $3,
            summary_zh = $4,
            summary_en = $5,
            content_zh = $6,
            content_en = $7,
            guide_type = $8,
            category = $9,
            province = $10,
            city = $11,
            district = $12,
            spot_name = $13,
            space_id = $14,
            cover_image_url = $15,
            images = $16,
            sections = $17,
            status = $18::guide_status,
            featured = $19,
            updated_at = now()
        WHERE id = $1
        RETURNING id, title_zh, title_en, province, city, district, spot_name,
                  status::text AS status, featured, author_id, space_id
        "#,
    )
    .bind(guide_id)
    .bind(input.title_zh)
    .bind(input.title_en)
    .bind(input.summary_zh)
    .bind(input.summary_en)
    .bind(input.content_zh)
    .bind(input.content_en)
    .bind(input.guide_type)
    .bind(input.category)
    .bind(input.province)
    .bind(input.city)
    .bind(input.district)
    .bind(input.spot_name)
    .bind(input.space_id)
    .bind(input.cover_image_url)
    .bind(images_json)
    .bind(sections_json)
    .bind(status)
    .bind(input.featured)
    .fetch_one(pool)
    .await?;

    row_to_guide_summary(row)
}

pub async fn archive_guide(pool: &PgPool, guide_id: Uuid) -> Result<GuideSummary, sqlx::Error> {
    let row = sqlx::query(
        r#"
        UPDATE guides
        SET status = 'archived',
            updated_at = now()
        WHERE id = $1
        RETURNING id, title_zh, title_en, province, city, district, spot_name,
                  status::text AS status, featured, author_id, space_id
        "#,
    )
    .bind(guide_id)
    .fetch_one(pool)
    .await?;

    row_to_guide_summary(row)
}

fn guide_status_from_db(value: &str) -> GuideStatus {
    match value {
        "draft" => GuideStatus::Draft,
        "archived" => GuideStatus::Archived,
        _ => GuideStatus::Published,
    }
}

fn guide_status_to_db(value: &GuideStatus) -> &'static str {
    match value {
        GuideStatus::Draft => "draft",
        GuideStatus::Published => "published",
        GuideStatus::Archived => "archived",
    }
}

fn parse_images(value: Value) -> Vec<String> {
    match value {
        Value::Array(items) => items
            .into_iter()
            .filter_map(|item| match item {
                Value::String(url) => Some(url),
                Value::Object(object) => object
                    .get("url")
                    .or_else(|| object.get("src"))
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
                _ => None,
            })
            .filter(|url| !url.trim().is_empty())
            .collect(),
        _ => Vec::new(),
    }
}

fn parse_sections(value: Value) -> Vec<GuideSection> {
    match value {
        Value::Array(items) => items
            .into_iter()
            .enumerate()
            .filter_map(|(index, item)| {
                let Value::Object(object) = item else {
                    return None;
                };
                let title_zh = object
                    .get("heading")
                    .or_else(|| object.get("title"))
                    .or_else(|| object.get("title_zh"))
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .trim()
                    .to_string();
                let content_zh = object
                    .get("body")
                    .or_else(|| object.get("content"))
                    .or_else(|| object.get("content_zh"))
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .trim()
                    .to_string();
                if title_zh.is_empty() && content_zh.is_empty() {
                    None
                } else {
                    Some(GuideSection {
                        id: object
                            .get("id")
                            .and_then(Value::as_str)
                            .map(ToOwned::to_owned)
                            .unwrap_or_else(|| format!("section_{index}")),
                        section_type: object
                            .get("type")
                            .or_else(|| object.get("section_type"))
                            .and_then(Value::as_str)
                            .unwrap_or("text")
                            .to_string(),
                        title_zh,
                        title_en: object
                            .get("title_en")
                            .and_then(Value::as_str)
                            .map(ToOwned::to_owned),
                        content_zh,
                        content_en: object
                            .get("content_en")
                            .and_then(Value::as_str)
                            .map(ToOwned::to_owned),
                        images: object
                            .get("images")
                            .cloned()
                            .map(parse_images)
                            .unwrap_or_default(),
                    })
                }
            })
            .collect(),
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn create_guide_draft_stores_structured_sections() {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
        let pool = sqlx::PgPool::connect(&database_url).await.expect("pool");
        let email = format!("guide-author-{}@example.com", uuid::Uuid::new_v4());
        let user = crate::users::create_user(&pool, &email, Some("Guide Author"), "hash")
            .await
            .expect("user");

        let guide = create_guide_draft(
            &pool,
            CreateGuideDraftInput {
                title_zh: "结构化攻略草稿".to_string(),
                title_en: Some("Structured guide draft".to_string()),
                summary_zh: Some("摘要".to_string()),
                summary_en: Some("Summary".to_string()),
                content_zh: Some("正文".to_string()),
                content_en: Some("Content".to_string()),
                guide_type: "attraction".to_string(),
                category: Some("walk".to_string()),
                province: "上海市".to_string(),
                city: "上海市".to_string(),
                district: Some("黄浦区".to_string()),
                spot_name: Some("外滩".to_string()),
                author_id: user.id,
                author_name: user.name.clone(),
                space_id: None,
                cover_image_url: None,
                images: vec!["https://example.com/guide.jpg".to_string()],
                sections: vec![GuideSection {
                    id: "sec_test".to_string(),
                    section_type: "transport".to_string(),
                    title_zh: "到达".to_string(),
                    title_en: Some("Arrival".to_string()),
                    content_zh: "从地铁站步行到达。".to_string(),
                    content_en: Some("Walk from metro.".to_string()),
                    images: vec!["https://example.com/section.jpg".to_string()],
                }],
                status: GuideStatus::Draft,
                featured: false,
            },
        )
        .await
        .expect("guide");

        assert!(matches!(guide.status, GuideStatus::Draft));
        assert_eq!(guide.title_zh, "结构化攻略草稿");

        let sections: Value = sqlx::query_scalar("SELECT sections FROM guides WHERE id = $1")
            .bind(guide.id)
            .fetch_one(&pool)
            .await
            .expect("sections");
        assert_eq!(sections[0]["title_zh"], "到达");
        assert_eq!(sections[0]["type"], "transport");

        sqlx::query("DELETE FROM guides WHERE id = $1")
            .bind(guide.id)
            .execute(&pool)
            .await
            .expect("cleanup guide");
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user.id)
            .execute(&pool)
            .await
            .expect("cleanup user");
    }
}
