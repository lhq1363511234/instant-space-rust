use instant_domain::{
    guides::{GuideDetail, GuideSection, GuideStatus, GuideSummary},
    locations::LocationNode,
};
use leptos::prelude::*;

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

    instant_db::guides::create_guide_draft(
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
            author_name: user.name.or(Some(user.email)),
            space_id,
            cover_image_url: clean_optional(cover_image_url),
            images,
            sections,
            status,
            featured,
        },
    )
    .await
    .map_err(|err| ServerFnError::new(err.to_string()))
}

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

    instant_db::guides::update_guide(
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
    .map_err(|err| ServerFnError::new(err.to_string()))
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

    instant_db::guides::archive_guide(&pool, guide_uuid)
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
        crate::server::auth::require_admin_user().await?;
        let guide_uuid = uuid::Uuid::parse_str(&guide_id)
            .map_err(|_| ServerFnError::new("invalid guide id"))?;
        let pool = crate::server::db_pool().await?;
        instant_db::guides::set_guide_status(&pool, guide_uuid, status)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))
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

#[server(ListProvinces, "/inspace/api")]
pub async fn list_provinces() -> Result<Vec<LocationNode>, ServerFnError> {
    let pool = crate::server::db_pool().await?;

    instant_db::locations::provinces(&pool)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))
}

#[server(ListCities, "/inspace/api")]
pub async fn list_cities(province: String) -> Result<Vec<String>, ServerFnError> {
    let pool = crate::server::db_pool().await?;

    instant_db::locations::cities(&pool, province)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))
}

#[server(ListDistricts, "/inspace/api")]
pub async fn list_districts(province: String, city: String) -> Result<Vec<String>, ServerFnError> {
    let pool = crate::server::db_pool().await?;

    instant_db::locations::districts(&pool, province, city)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))
}

#[server(ListSpots, "/inspace/api")]
pub async fn list_spots(
    province: String,
    city: String,
    district: Option<String>,
) -> Result<Vec<String>, ServerFnError> {
    let pool = crate::server::db_pool().await?;

    instant_db::locations::spots(&pool, province, city, district)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))
}
