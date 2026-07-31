#![cfg_attr(not(feature = "ssr"), allow(dead_code))]

use instant_domain::guides::{GuideDetail, GuideSection, GuideStatus, GuideSummary};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GuidePageResult {
    pub items: Vec<GuideSummary>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
    pub total_pages: i32,
}

#[server(ListGuidePage, "/inspace/api")]
pub async fn list_guide_page(
    q: Option<String>,
    country: Option<String>,
    province: Option<String>,
    city: Option<String>,
    district: Option<String>,
    spot_name: Option<String>,
    page: i32,
    page_size: i32,
) -> Result<GuidePageResult, ServerFnError> {
    let page_size = page_size.clamp(10, 50);
    let requested_page = page.max(1);
    let pool = crate::server::db_pool().await?;
    let user = crate::server::auth::current_session().await.ok().flatten();

    let clean = |value: Option<String>| {
        value
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    };
    let q = clean(q);
    let country = clean(country);
    let province = clean(province);
    let city = clean(city);
    let district = clean(district);
    let spot_name = clean(spot_name);

    let fetch = |page: i32| {
        let pool = pool.clone();
        let (q, country, province, city, district, spot_name) = (
            q.clone(),
            country.clone(),
            province.clone(),
            city.clone(),
            district.clone(),
            spot_name.clone(),
        );
        async move {
            instant_db::guides::list_published_guides_page(
                &pool,
                q,
                country,
                province,
                city,
                district,
                spot_name,
                i64::from(page_size),
                i64::from((page - 1) * page_size),
            )
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))
        }
    };

    let mut result = fetch(requested_page).await?;
    let total_pages = if result.total == 0 {
        1
    } else {
        ((result.total + i64::from(page_size) - 1) / i64::from(page_size)) as i32
    };
    // A stale deep link past the last page should land on the last real page
    // rather than an empty list.
    let page = requested_page.min(total_pages);
    if result.items.is_empty() && result.total > 0 {
        result = fetch(page).await?;
    }

    let mut items = result.items;
    mark_summaries_editable(&pool, &mut items, user.as_ref()).await?;

    Ok(GuidePageResult {
        items,
        total: result.total,
        page,
        page_size,
        total_pages,
    })
}

#[server(ListGuides, "/inspace/api")]
pub async fn list_guides(
    province: Option<String>,
    city: Option<String>,
    district: Option<String>,
    spot_name: Option<String>,
) -> Result<Vec<GuideSummary>, ServerFnError> {
    let pool = crate::server::db_pool().await?;
    let user = crate::server::auth::current_session().await.ok().flatten();

    let mut guides =
        instant_db::guides::list_published_guides(&pool, province, city, district, spot_name)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?;
    mark_summaries_editable(&pool, &mut guides, user.as_ref()).await?;

    Ok(guides)
}

#[cfg(feature = "ssr")]
async fn mark_summaries_editable(
    pool: &sqlx::PgPool,
    guides: &mut [GuideSummary],
    user: Option<&instant_domain::auth::CurrentUser>,
) -> Result<(), ServerFnError> {
    let Some(user) = user else {
        return Ok(());
    };
    for guide in guides {
        guide.can_edit = can_edit_summary(pool, guide, user).await?;
    }
    Ok(())
}

#[cfg(not(feature = "ssr"))]
async fn mark_summaries_editable(
    _pool: &(),
    _guides: &mut [GuideSummary],
    _user: Option<&instant_domain::auth::CurrentUser>,
) -> Result<(), ServerFnError> {
    Ok(())
}

#[cfg(feature = "ssr")]
async fn can_edit_summary(
    pool: &sqlx::PgPool,
    guide: &GuideSummary,
    user: &instant_domain::auth::CurrentUser,
) -> Result<bool, ServerFnError> {
    if user.role.is_admin() || guide.author_id == Some(user.id) {
        return Ok(true);
    }
    if guide.author_id.is_none() {
        if let Some(space_id) = guide.space_id {
            let owner = instant_db::spaces::space_host_user_id(pool, space_id)
                .await
                .map_err(|err| ServerFnError::new(err.to_string()))?;
            return Ok(owner == Some(user.id));
        }
    }
    Ok(false)
}

#[cfg(not(feature = "ssr"))]
async fn can_edit_summary(
    _pool: &(),
    _guide: &GuideSummary,
    _user: &instant_domain::auth::CurrentUser,
) -> Result<bool, ServerFnError> {
    Ok(false)
}

#[cfg(feature = "ssr")]
async fn mark_detail_editable(
    pool: &sqlx::PgPool,
    guide: &mut GuideDetail,
    user: Option<&instant_domain::auth::CurrentUser>,
) -> Result<(), ServerFnError> {
    let Some(user) = user else {
        return Ok(());
    };
    guide.can_edit = if user.role.is_admin() || guide.author_id == Some(user.id) {
        true
    } else if guide.author_id.is_none() {
        if let Some(space_id) = guide.space_id {
            let owner = instant_db::spaces::space_host_user_id(pool, space_id)
                .await
                .map_err(|err| ServerFnError::new(err.to_string()))?;
            owner == Some(user.id)
        } else {
            false
        }
    } else {
        false
    };
    Ok(())
}

#[cfg(not(feature = "ssr"))]
async fn mark_detail_editable(
    _pool: &(),
    _guide: &mut GuideDetail,
    _user: Option<&instant_domain::auth::CurrentUser>,
) -> Result<(), ServerFnError> {
    Ok(())
}

#[server(GetGuideDetail, "/inspace/api")]
pub async fn get_guide_detail(guide_id: String) -> Result<Option<GuideDetail>, ServerFnError> {
    let pool = crate::server::db_pool().await?;
    let user = crate::server::auth::current_session().await.ok().flatten();
    let guide_id =
        uuid::Uuid::parse_str(&guide_id).map_err(|_| ServerFnError::new("invalid guide id"))?;

    let mut guide = instant_db::guides::get_published_guide(&pool, guide_id)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;
    if let Some(guide) = guide.as_mut() {
        mark_detail_editable(&pool, guide, user.as_ref()).await?;
    }
    Ok(guide)
}

#[server(GetGuideForEdit, "/inspace/api")]
pub async fn get_guide_for_edit(guide_id: String) -> Result<Option<GuideDetail>, ServerFnError> {
    let Some(user) = crate::server::auth::current_session().await? else {
        return Err(ServerFnError::new("login required"));
    };
    let pool = crate::server::db_pool().await?;
    let guide_id =
        uuid::Uuid::parse_str(&guide_id).map_err(|_| ServerFnError::new("invalid guide id"))?;
    let Some(guide) = instant_db::guides::get_guide(&pool, guide_id)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?
    else {
        return Ok(None);
    };
    ensure_guide_editor(&pool, &guide, &user).await?;

    Ok(Some(guide))
}

#[server(ListBindableGuides, "/inspace/api")]
pub async fn list_bindable_guides() -> Result<Vec<GuideSummary>, ServerFnError> {
    let Some(user) = crate::server::auth::current_session().await? else {
        return Err(ServerFnError::new("login required"));
    };
    let pool = crate::server::db_pool().await?;
    let mut guides = if user.role.is_admin() {
        instant_db::guides::list_all_guides_admin(&pool).await
    } else {
        instant_db::guides::list_guides_by_author(&pool, user.id).await
    }
    .map_err(|err| ServerFnError::new(err.to_string()))?;
    for guide in guides.iter_mut() {
        guide.can_edit = true;
    }
    Ok(guides)
}

#[server(BindGuideToSpace, "/inspace/api")]
pub async fn bind_guide_to_space(
    guide_id: String,
    space_id: String,
) -> Result<GuideSummary, ServerFnError> {
    let Some(user) = crate::server::auth::current_session().await? else {
        return Err(ServerFnError::new("login required"));
    };
    let guide_uuid =
        uuid::Uuid::parse_str(&guide_id).map_err(|_| ServerFnError::new("invalid guide id"))?;
    let space_uuid =
        uuid::Uuid::parse_str(&space_id).map_err(|_| ServerFnError::new("invalid space id"))?;
    let pool = crate::server::db_pool().await?;
    let Some(existing) = instant_db::guides::get_guide(&pool, guide_uuid)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?
    else {
        return Err(ServerFnError::new("guide not found"));
    };
    ensure_guide_editor(&pool, &existing, &user).await?;
    if !user.role.is_admin() {
        let owner = instant_db::spaces::space_host_user_id(&pool, space_uuid)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?;
        if owner != Some(user.id) {
            return Err(ServerFnError::new("space owner or admin required"));
        }
    }
    instant_db::spaces::get_space_summary(&pool, space_uuid)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?
        .ok_or_else(|| ServerFnError::new("space not found"))?;
    instant_db::guides::bind_guide_to_space(&pool, guide_uuid, space_uuid)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))
}

#[server(ListSpaceGuides, "/inspace/api")]
pub async fn list_space_guides(space_id: String) -> Result<Vec<GuideSummary>, ServerFnError> {
    let pool = crate::server::db_pool().await?;
    let space_id =
        uuid::Uuid::parse_str(&space_id).map_err(|_| ServerFnError::new("invalid space id"))?;

    let mut guides = instant_db::guides::list_published_guides_by_space(&pool, space_id)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;
    let user = crate::server::auth::current_session().await.ok().flatten();
    mark_summaries_editable(&pool, &mut guides, user.as_ref()).await?;

    Ok(guides)
}

/// Host/admin view: published + draft + archived guides for one space.
#[server(ListManageableSpaceGuides, "/inspace/api")]
pub async fn list_manageable_space_guides(
    space_id: String,
) -> Result<Vec<GuideSummary>, ServerFnError> {
    let Some(user) = crate::server::auth::current_session().await? else {
        return Err(ServerFnError::new("login required"));
    };
    let pool = crate::server::db_pool().await?;
    let space_uuid =
        uuid::Uuid::parse_str(&space_id).map_err(|_| ServerFnError::new("invalid space id"))?;

    if !user.role.is_admin() {
        let owner = instant_db::spaces::space_host_user_id(&pool, space_uuid)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?;
        if owner != Some(user.id) {
            return Err(ServerFnError::new("space owner or admin required"));
        }
    }

    let mut guides = instant_db::guides::list_guides_by_space(&pool, space_uuid, true)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;
    mark_summaries_editable(&pool, &mut guides, Some(&user)).await?;
    Ok(guides)
}

#[allow(clippy::too_many_arguments)]
#[server(CreateGuideDraft, "/inspace/api")]
pub async fn create_guide_draft(
    title_zh: String,
    title_en: Option<String>,
    summary_zh: Option<String>,
    summary_en: Option<String>,
    content_zh: Option<String>,
    content_en: Option<String>,
    guide_type: String,
    category: Option<String>,
    province: String,
    city: String,
    district: Option<String>,
    spot_name: Option<String>,
    space_id: Option<String>,
    cover_image_url: Option<String>,
    images: Vec<String>,
    sections: Vec<GuideSection>,
    status: GuideStatus,
    featured: bool,
) -> Result<GuideSummary, ServerFnError> {
    let Some(user) = crate::server::auth::current_session().await? else {
        return Err(ServerFnError::new("login required"));
    };
    let title_zh = title_zh.trim().to_string();
    let province = province.trim().to_string();
    let city = city.trim().to_string();
    if title_zh.is_empty() || province.is_empty() || city.is_empty() {
        return Err(ServerFnError::new("title, province and city are required"));
    }
    let guide_type = clean_optional(Some(guide_type)).unwrap_or_else(|| "attraction".to_string());
    let space_id = parse_optional_uuid(space_id, "invalid space id")?;
    let featured = if featured {
        #[cfg(feature = "ssr")]
        {
            user.role.is_admin()
        }
        #[cfg(not(feature = "ssr"))]
        {
            false
        }
    } else {
        false
    };
    let sections = clean_sections(sections);
    let images = clean_images(images);
    let pool = crate::server::db_pool().await?;

    let snapshot_name = user.name.clone().or(Some(user.email.clone()));
    let summary = instant_db::guides::create_guide_draft(
        &pool,
        instant_db::guides::CreateGuideDraftInput {
            title_zh,
            title_en: clean_optional(title_en),
            summary_zh: clean_optional(summary_zh),
            summary_en: clean_optional(summary_en),
            content_zh: clean_optional(content_zh),
            content_en: clean_optional(content_en),
            guide_type,
            category: clean_optional(category),
            province,
            city,
            district: clean_optional(district),
            spot_name: clean_optional(spot_name),
            author_id: user.id,
            author_name: snapshot_name.clone(),
            space_id,
            cover_image_url: clean_optional(cover_image_url),
            images,
            sections,
            status,
            featured,
        },
    )
    .await
    .map_err(|err| ServerFnError::new(err.to_string()))?;

    // Phase 4: open the version history with snapshot v1.
    let _ = instant_db::guides::snapshot_guide_version(
        &pool,
        summary.id,
        Some(user.id),
        snapshot_name.as_deref(),
    )
    .await;

    Ok(summary)
}

#[allow(clippy::too_many_arguments)]
#[server(UpdateGuide, "/inspace/api")]
pub async fn update_guide(
    guide_id: String,
    title_zh: String,
    title_en: Option<String>,
    summary_zh: Option<String>,
    summary_en: Option<String>,
    content_zh: Option<String>,
    content_en: Option<String>,
    guide_type: String,
    category: Option<String>,
    province: String,
    city: String,
    district: Option<String>,
    spot_name: Option<String>,
    space_id: Option<String>,
    cover_image_url: Option<String>,
    images: Vec<String>,
    sections: Vec<GuideSection>,
    status: GuideStatus,
    featured: bool,
) -> Result<GuideSummary, ServerFnError> {
    let Some(user) = crate::server::auth::current_session().await? else {
        return Err(ServerFnError::new("login required"));
    };
    let guide_uuid =
        uuid::Uuid::parse_str(&guide_id).map_err(|_| ServerFnError::new("invalid guide id"))?;
    let pool = crate::server::db_pool().await?;
    let Some(existing) = instant_db::guides::get_guide(&pool, guide_uuid)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?
    else {
        return Err(ServerFnError::new("guide not found"));
    };
    ensure_guide_editor(&pool, &existing, &user).await?;

    let title_zh = title_zh.trim().to_string();
    let province = province.trim().to_string();
    let city = city.trim().to_string();
    if title_zh.is_empty() || province.is_empty() || city.is_empty() {
        return Err(ServerFnError::new("title, province and city are required"));
    }
    let guide_type = clean_optional(Some(guide_type)).unwrap_or_else(|| "attraction".to_string());
    let space_id = parse_optional_uuid(space_id, "invalid space id")?;
    let featured = if featured && user.role.is_admin() {
        true
    } else if user.role.is_admin() {
        false
    } else {
        existing.featured
    };
    let sections = clean_sections(sections);
    let images = clean_images(images);

    let snapshot_name = user.name.clone().or(Some(user.email.clone()));
    let summary = instant_db::guides::update_guide(
        &pool,
        guide_uuid,
        instant_db::guides::UpdateGuideInput {
            title_zh,
            title_en: clean_optional(title_en),
            summary_zh: clean_optional(summary_zh),
            summary_en: clean_optional(summary_en),
            content_zh: clean_optional(content_zh),
            content_en: clean_optional(content_en),
            guide_type,
            category: clean_optional(category),
            province,
            city,
            district: clean_optional(district),
            spot_name: clean_optional(spot_name),
            space_id,
            cover_image_url: clean_optional(cover_image_url),
            images,
            sections,
            status,
            featured,
        },
    )
    .await
    .map_err(|err| ServerFnError::new(err.to_string()))?;

    // Phase 4: every edit freezes the previous state into the version history.
    let _ = instant_db::guides::snapshot_guide_version(
        &pool,
        guide_uuid,
        Some(user.id),
        snapshot_name.as_deref(),
    )
    .await;

    Ok(summary)
}

#[server(DeleteGuide, "/inspace/api")]
pub async fn delete_guide(guide_id: String) -> Result<GuideSummary, ServerFnError> {
    let Some(user) = crate::server::auth::current_session().await? else {
        return Err(ServerFnError::new("login required"));
    };
    let guide_uuid =
        uuid::Uuid::parse_str(&guide_id).map_err(|_| ServerFnError::new("invalid guide id"))?;
    let pool = crate::server::db_pool().await?;
    let Some(existing) = instant_db::guides::get_guide(&pool, guide_uuid)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?
    else {
        return Err(ServerFnError::new("guide not found"));
    };
    ensure_guide_editor(&pool, &existing, &user).await?;

    instant_db::guides::delete_guide_row(&pool, guide_uuid)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))
}

#[server(ListAdminGuides, "/inspace/api")]
pub async fn list_admin_guides() -> Result<Vec<GuideSummary>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::server::auth::require_admin_user().await?;
        let pool = crate::server::db_pool().await?;
        let mut guides = instant_db::guides::list_all_guides_admin(&pool)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?;
        for guide in guides.iter_mut() {
            guide.can_edit = true;
        }
        Ok(guides)
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("server only"))
    }
}

#[server(SetGuideStatusAdmin, "/inspace/api")]
pub async fn set_guide_status_admin(
    guide_id: String,
    status: GuideStatus,
) -> Result<GuideSummary, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let actor = crate::server::auth::require_admin_user().await?;
        let guide_uuid =
            uuid::Uuid::parse_str(&guide_id).map_err(|_| ServerFnError::new("invalid guide id"))?;
        let pool = crate::server::db_pool().await?;
        let updated = instant_db::guides::set_guide_status(&pool, guide_uuid, status)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?;
        let _ = instant_db::admin::record_audit(
            &pool,
            Some(actor.id),
            Some(&actor.email),
            "set_guide_status",
            "guide",
            Some(&guide_id),
            Some(match status {
                GuideStatus::Draft => "draft",
                GuideStatus::Published => "published",
                GuideStatus::Archived => "archived",
            }),
        )
        .await;
        Ok(updated)
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (guide_id, status);
        Err(ServerFnError::new("server only"))
    }
}

#[cfg(feature = "ssr")]
async fn ensure_guide_editor(
    pool: &sqlx::PgPool,
    guide: &GuideDetail,
    user: &instant_domain::auth::CurrentUser,
) -> Result<(), ServerFnError> {
    if user.role.is_admin() || guide.author_id == Some(user.id) {
        return Ok(());
    }

    if guide.author_id.is_none() {
        if let Some(space_id) = guide.space_id {
            let owner = instant_db::spaces::space_host_user_id(pool, space_id)
                .await
                .map_err(|err| ServerFnError::new(err.to_string()))?;
            if owner == Some(user.id) {
                return Ok(());
            }
        }
    }

    Err(ServerFnError::new("guide editor permission required"))
}

fn parse_optional_uuid(
    value: Option<String>,
    err: &str,
) -> Result<Option<uuid::Uuid>, ServerFnError> {
    value
        .and_then(|value| {
            let value = value.trim().to_string();
            (!value.is_empty()).then_some(value)
        })
        .map(|value| uuid::Uuid::parse_str(&value).map_err(|_| ServerFnError::new(err)))
        .transpose()
}

fn clean_sections(sections: Vec<GuideSection>) -> Vec<GuideSection> {
    sections
        .into_iter()
        .map(|section| GuideSection {
            id: clean_optional(Some(section.id))
                .unwrap_or_else(|| format!("sec_{}", uuid::Uuid::new_v4())),
            section_type: clean_optional(Some(section.section_type))
                .unwrap_or_else(|| "text".to_string()),
            title_zh: section.title_zh.trim().to_string(),
            title_en: clean_optional(section.title_en),
            content_zh: section.content_zh.trim().to_string(),
            content_en: clean_optional(section.content_en),
            images: clean_images(section.images),
        })
        .filter(|section| !section.title_zh.is_empty() || !section.content_zh.is_empty())
        .collect()
}

fn clean_images(images: Vec<String>) -> Vec<String> {
    images
        .into_iter()
        .map(|url| url.trim().to_string())
        .filter(|url| !url.is_empty())
        .collect()
}

fn clean_optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[server(ListGuideCountries, "/inspace/api")]
pub async fn list_guide_countries() -> Result<Vec<String>, ServerFnError> {
    let pool = crate::server::db_pool().await?;

    instant_db::guides::published_guide_countries(&pool)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))
}

#[server(ListProvinces, "/inspace/api")]
pub async fn list_provinces(country: Option<String>) -> Result<Vec<String>, ServerFnError> {
    let pool = crate::server::db_pool().await?;
    instant_db::guides::published_guide_provinces(&pool, clean_optional(country))
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))
}

#[server(ListCities, "/inspace/api")]
pub async fn list_cities(
    country: Option<String>,
    province: Option<String>,
) -> Result<Vec<String>, ServerFnError> {
    let pool = crate::server::db_pool().await?;
    instant_db::guides::published_guide_cities(
        &pool,
        clean_optional(country),
        clean_optional(province),
    )
    .await
    .map_err(|err| ServerFnError::new(err.to_string()))
}

#[server(ListDistricts, "/inspace/api")]
pub async fn list_districts(
    country: Option<String>,
    province: Option<String>,
    city: Option<String>,
) -> Result<Vec<String>, ServerFnError> {
    let pool = crate::server::db_pool().await?;
    instant_db::guides::published_guide_districts(
        &pool,
        clean_optional(country),
        clean_optional(province),
        clean_optional(city),
    )
    .await
    .map_err(|err| ServerFnError::new(err.to_string()))
}

#[server(ListSpots, "/inspace/api")]
pub async fn list_spots(
    country: Option<String>,
    province: Option<String>,
    city: Option<String>,
    district: Option<String>,
) -> Result<Vec<String>, ServerFnError> {
    let pool = crate::server::db_pool().await?;
    instant_db::guides::published_guide_spots(
        &pool,
        clean_optional(country),
        clean_optional(province),
        clean_optional(city),
        clean_optional(district),
    )
    .await
    .map_err(|err| ServerFnError::new(err.to_string()))
}

/// Phase 4 content versioning: review a guide's edit history.
#[server(ListGuideVersions, "/inspace/api")]
pub async fn list_guide_versions(
    guide_id: String,
) -> Result<Vec<instant_domain::guides::GuideVersion>, ServerFnError> {
    let Some(user) = crate::server::auth::current_session().await? else {
        return Err(ServerFnError::new("login required"));
    };
    let guide_uuid =
        uuid::Uuid::parse_str(&guide_id).map_err(|_| ServerFnError::new("invalid guide id"))?;
    let pool = crate::server::db_pool().await?;
    let Some(existing) = instant_db::guides::get_guide(&pool, guide_uuid)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?
    else {
        return Err(ServerFnError::new("guide not found"));
    };
    ensure_guide_editor(&pool, &existing, &user).await?;

    instant_db::guides::list_guide_versions(&pool, guide_uuid)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))
}

/// Phase 4 content versioning: restore a guide from a frozen snapshot.
#[server(RestoreGuideVersion, "/inspace/api")]
pub async fn restore_guide_version(
    guide_id: String,
    version_no: i32,
) -> Result<GuideSummary, ServerFnError> {
    let Some(user) = crate::server::auth::current_session().await? else {
        return Err(ServerFnError::new("login required"));
    };
    let guide_uuid =
        uuid::Uuid::parse_str(&guide_id).map_err(|_| ServerFnError::new("invalid guide id"))?;
    let pool = crate::server::db_pool().await?;
    let Some(existing) = instant_db::guides::get_guide(&pool, guide_uuid)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?
    else {
        return Err(ServerFnError::new("guide not found"));
    };
    ensure_guide_editor(&pool, &existing, &user).await?;

    let snapshot_name = user.name.clone().or(Some(user.email.clone()));
    instant_db::guides::restore_guide_version(
        &pool,
        guide_uuid,
        version_no,
        Some(user.id),
        snapshot_name.as_deref(),
    )
    .await
    .map_err(|err| ServerFnError::new(err.to_string()))
}

/// Phase 5: server-side pagination for the admin guide console. The table holds
/// thousands of guides; the browser must never receive the whole table.
#[server(ListAdminGuidesPage, "/inspace/api")]
pub async fn list_admin_guides_page(
    query: String,
    status: String,
    page: i32,
    page_size: i32,
) -> Result<instant_domain::guides::PaginatedGuides, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::server::auth::require_admin_user().await?;
        let pool = crate::server::db_pool().await?;
        let page = page.max(1);
        let page_size = page_size.clamp(10, 100);
        let offset = ((page - 1) * page_size) as i64;

        let mut result = instant_db::guides::list_all_guides_admin_page(
            &pool,
            if query.trim().is_empty() { None } else { Some(query) },
            if status.trim().is_empty() { None } else { Some(status) },
            page_size as i64,
            offset,
        )
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;
        for guide in result.items.iter_mut() {
            guide.can_edit = true;
        }
        Ok(result)
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (query, status, page, page_size);
        Err(ServerFnError::new("server only"))
    }
}

/// Phase 5: guide status counts for the admin console stat cards.
#[server(GetAdminGuideStats, "/inspace/api")]
pub async fn get_admin_guide_stats() -> Result<instant_domain::guides::GuideStatusCounts, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::server::auth::require_admin_user().await?;
        let pool = crate::server::db_pool().await?;
        instant_db::guides::guide_status_counts(&pool)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("server only"))
    }
}
