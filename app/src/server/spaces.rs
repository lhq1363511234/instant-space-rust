use instant_domain::{
    spaces::{SpaceStatus, SpaceSummary, SpaceType},
    traces::{FeaturedStory, PresenceProof},
};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use instant_auth::{generate_password_code, hash_password};
#[cfg(feature = "ssr")]
use instant_db::spaces::{
    apply_resident, archive_template, create_host_space, get_space_summary, list_admin_spaces_page,
    list_all_spaces_admin, list_featured_home_spaces, list_home_spaces, list_home_spaces_page,
    list_host_spaces, list_manageable_spaces, rotate_space_password, set_home_weight,
    set_space_status, space_host_user_id, update_host_space, CreateSpaceInput, SpaceFilter,
    UpdateSpaceInput,
};

#[cfg(feature = "ssr")]
use instant_domain::spaces::hotspot_name;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpaceMarker {
    pub id: String,
    pub name_zh: String,
    pub name_en: Option<String>,
    pub space_type: SpaceType,
    pub country: Option<String>,
    pub province: Option<String>,
    pub city: Option<String>,
    pub district: Option<String>,
    pub spot_name: Option<String>,
    pub address_line: Option<String>,
    pub lat: f64,
    pub lng: f64,
    pub is_public: bool,
    pub status: String,
    pub expires_at: Option<String>,
    pub online_count: i32,
    pub home_weight: i32,
    pub generated_password: Option<String>,
    pub hotspot_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpacePageResult {
    pub items: Vec<SpaceMarker>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
    pub total_pages: i32,
}

pub fn to_marker(space: SpaceSummary) -> SpaceMarker {
    SpaceMarker {
        id: space.id.to_string(),
        name_zh: space.name_zh,
        name_en: space.name_en,
        space_type: space.space_type,
        country: space.country,
        province: space.province,
        city: space.city,
        district: space.district,
        spot_name: space.spot_name,
        address_line: space.address_line,
        lat: space.lat,
        lng: space.lng,
        is_public: space.is_public,
        status: space_status_key(&space.status).to_string(),
        expires_at: space.expires_at.map(|value| value.to_string()),
        online_count: space.online_count,
        home_weight: space.home_weight,
        generated_password: None,
        hotspot_name: None,
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HomeStoryView {
    pub id: String,
    pub space_id: String,
    pub space_name_zh: String,
    pub city: Option<String>,
    pub body: String,
    pub author_name: String,
    pub proof: PresenceProof,
    pub created_at: String,
}

fn to_home_story(story: FeaturedStory) -> HomeStoryView {
    HomeStoryView {
        id: story.id.to_string(),
        space_id: story.space_id.to_string(),
        space_name_zh: story.space_name_zh,
        city: story.city,
        body: story.body,
        author_name: story.author_name,
        proof: story.proof,
        created_at: story.created_at.date().to_string(),
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpaceDetailView {
    pub summary: SpaceMarker,
    pub description_zh: Option<String>,
    pub description_en: Option<String>,
    pub tag_zh: Option<String>,
    pub tag_en: Option<String>,
    pub custom_type: Option<String>,
    pub host_name: Option<String>,
    pub host_bio_zh: Option<String>,
    pub host_bio_en: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HostClaimState {
    /// Not logged in — the UI should prompt sign-in before applying.
    Anonymous,
    /// Logged in, no claim yet — show the apply button.
    None,
    /// A claim is awaiting admin review.
    Pending,
    /// The claim was approved; this user is (or is becoming) the host.
    Approved,
    /// The claim was rejected.
    Rejected,
    /// The Space already has a host, so no claim was recorded.
    AlreadyHosted,
}

#[server(ListSpaces, "/inspace/api")]
pub async fn list_spaces(
    q: Option<String>,
    space_type: Option<SpaceType>,
    country: Option<String>,
    province: Option<String>,
    city: Option<String>,
) -> Result<Vec<SpaceMarker>, ServerFnError> {
    let pool = crate::server::db_pool().await?;
    let rows = list_home_spaces(
        &pool,
        SpaceFilter {
            q: clean_optional(q),
            space_type,
            country: clean_optional(country),
            province: clean_optional(province),
            city: clean_optional(city),
            district: None,
            spot_name: None,
        },
    )
    .await
    .map_err(|err| ServerFnError::new(err.to_string()))?;

    Ok(rows.into_iter().map(to_marker).collect())
}

#[server(ListSpacePage, "/inspace/api")]
pub async fn list_space_page(
    q: Option<String>,
    space_type: Option<SpaceType>,
    country: Option<String>,
    province: Option<String>,
    city: Option<String>,
    district: Option<String>,
    spot_name: Option<String>,
    page: i32,
    page_size: i32,
) -> Result<SpacePageResult, ServerFnError> {
    let page_size = page_size.clamp(10, 50);
    let page = page.max(1);
    let pool = crate::server::db_pool().await?;
    let filter = SpaceFilter {
        q: clean_optional(q),
        space_type,
        country: clean_optional(country),
        province: clean_optional(province),
        city: clean_optional(city),
        district: clean_optional(district),
        spot_name: clean_optional(spot_name),
    };
    let fetch = |page: i32| {
        let pool = pool.clone();
        let filter = filter.clone();
        async move {
            list_home_spaces_page(
                &pool,
                filter,
                i64::from(page_size),
                i64::from((page - 1) * page_size),
            )
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))
        }
    };

    let mut result = fetch(page).await?;
    let total_pages = if result.total == 0 {
        1
    } else {
        ((result.total + i64::from(page_size) - 1) / i64::from(page_size)) as i32
    };
    // Out-of-range page numbers (deep links, stale bookmarks) resolve to the
    // last real page instead of silently showing page 1 content.
    let page = page.min(total_pages);
    if result.items.is_empty() && result.total > 0 {
        result = fetch(page).await?;
    }

    Ok(SpacePageResult {
        items: result.items.into_iter().map(to_marker).collect(),
        total: result.total,
        page,
        page_size,
        total_pages,
    })
}

#[server(ListSpaceFilterCountries, "/inspace/api")]
pub async fn list_space_filter_countries() -> Result<Vec<String>, ServerFnError> {
    let pool = crate::server::db_pool().await?;
    instant_db::spaces::discoverable_space_countries(&pool)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))
}

#[server(ListSpaceFilterProvinces, "/inspace/api")]
pub async fn list_space_filter_provinces(
    country: Option<String>,
) -> Result<Vec<String>, ServerFnError> {
    let pool = crate::server::db_pool().await?;
    instant_db::spaces::discoverable_space_provinces(&pool, clean_optional(country))
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))
}

#[server(ListSpaceFilterCities, "/inspace/api")]
pub async fn list_space_filter_cities(
    country: Option<String>,
    province: Option<String>,
) -> Result<Vec<String>, ServerFnError> {
    let pool = crate::server::db_pool().await?;
    instant_db::spaces::discoverable_space_cities(
        &pool,
        clean_optional(country),
        clean_optional(province),
    )
    .await
    .map_err(|err| ServerFnError::new(err.to_string()))
}

#[server(ListSpaceFilterDistricts, "/inspace/api")]
pub async fn list_space_filter_districts(
    country: Option<String>,
    province: Option<String>,
    city: Option<String>,
) -> Result<Vec<String>, ServerFnError> {
    let pool = crate::server::db_pool().await?;
    instant_db::spaces::discoverable_space_districts(
        &pool,
        clean_optional(country),
        clean_optional(province),
        clean_optional(city),
    )
    .await
    .map_err(|err| ServerFnError::new(err.to_string()))
}

#[server(ListSpaceFilterSpots, "/inspace/api")]
pub async fn list_space_filter_spots(
    country: Option<String>,
    province: Option<String>,
    city: Option<String>,
    district: Option<String>,
) -> Result<Vec<String>, ServerFnError> {
    let pool = crate::server::db_pool().await?;
    instant_db::spaces::discoverable_space_spots(
        &pool,
        clean_optional(country),
        clean_optional(province),
        clean_optional(city),
        clean_optional(district),
    )
    .await
    .map_err(|err| ServerFnError::new(err.to_string()))
}

#[server(ListHomeFeaturedSpaces, "/inspace/api")]
pub async fn list_home_featured_spaces(limit: i32) -> Result<Vec<SpaceMarker>, ServerFnError> {
    let pool = crate::server::db_pool().await?;
    let rows = list_featured_home_spaces(&pool, i64::from(limit.clamp(1, 12)))
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;
    Ok(rows.into_iter().map(to_marker).collect())
}

#[server(ListHomeFeaturedStories, "/inspace/api")]
pub async fn list_home_featured_stories(limit: i32) -> Result<Vec<HomeStoryView>, ServerFnError> {
    let pool = crate::server::db_pool().await?;
    let rows = instant_db::traces::list_featured_stories(&pool, i64::from(limit.clamp(1, 8)))
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;
    Ok(rows.into_iter().map(to_home_story).collect())
}

#[server(ListMySpaces, "/inspace/api")]
pub async fn list_my_spaces() -> Result<Vec<SpaceMarker>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let Some(user) = crate::server::auth::current_session().await? else {
            return Err(ServerFnError::new("login required"));
        };
        let pool = crate::server::db_pool().await?;
        let rows = if user.role.is_admin() {
            list_manageable_spaces(&pool).await
        } else {
            list_host_spaces(&pool, user.id).await
        }
        .map_err(|err| ServerFnError::new(err.to_string()))?;

        Ok(rows.into_iter().map(to_marker).collect())
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("server only"))
    }
}

#[server(ListAdminSpacePage, "/inspace/api")]
pub async fn list_admin_space_page(
    q: Option<String>,
    status: Option<String>,
    space_type: Option<SpaceType>,
    page: i32,
    page_size: i32,
) -> Result<SpacePageResult, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::server::auth::require_admin_user().await?;
        let page_size = page_size.clamp(10, 100);
        let requested_page = page.max(1);
        let pool = crate::server::db_pool().await?;
        let status = clean_optional(status).and_then(|value| {
            matches!(
                value.as_str(),
                "managed" | "active" | "expired" | "closed" | "archived" | "template"
            )
            .then_some(value)
        });
        let q = clean_optional(q);
        let result = list_admin_spaces_page(
            &pool,
            q.clone(),
            status.clone(),
            space_type.clone(),
            i64::from(page_size),
            i64::from((requested_page - 1) * page_size),
        )
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;
        let total_pages = if result.total == 0 {
            1
        } else {
            ((result.total + i64::from(page_size) - 1) / i64::from(page_size)) as i32
        };
        let page = requested_page.min(total_pages);

        // A filter change can leave the browser on a page past the new end.
        // Fetch the valid last page instead of returning an empty, confusing table.
        let result = if page != requested_page {
            list_admin_spaces_page(
                &pool,
                q,
                status,
                space_type,
                i64::from(page_size),
                i64::from((page - 1) * page_size),
            )
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?
        } else {
            result
        };

        Ok(SpacePageResult {
            items: result.items.into_iter().map(to_marker).collect(),
            total: result.total,
            page,
            page_size,
            total_pages,
        })
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("server only"))
    }
}

#[server(ListAdminSpaces, "/inspace/api")]
pub async fn list_admin_spaces() -> Result<Vec<SpaceMarker>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::server::auth::require_admin_user().await?;
        let pool = crate::server::db_pool().await?;
        let rows = list_all_spaces_admin(&pool)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?;

        Ok(rows.into_iter().map(to_marker).collect())
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("server only"))
    }
}

#[server(GetSpaceForGuide, "/inspace/api")]
pub async fn get_space_for_guide(space_id: String) -> Result<Option<SpaceMarker>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let space_uuid = parse_space_id(&space_id)?;
        let pool = crate::server::db_pool().await?;
        let Some(space) = get_space_summary(&pool, space_uuid)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?
        else {
            return Ok(None);
        };

        Ok(Some(to_marker(space)))
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("server only"))
    }
}

#[server(GetSpaceDetail, "/inspace/api")]
pub async fn get_space_detail(space_id: String) -> Result<Option<SpaceDetailView>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let space_uuid = parse_space_id(&space_id)?;
        let pool = crate::server::db_pool().await?;
        let Some(detail) = instant_db::spaces::get_space_detail(&pool, space_uuid)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?
        else {
            return Ok(None);
        };

        Ok(Some(SpaceDetailView {
            summary: to_marker(detail.summary),
            description_zh: detail.description_zh,
            description_en: detail.description_en,
            tag_zh: detail.tag_zh,
            tag_en: detail.tag_en,
            custom_type: detail.custom_type,
            host_name: detail.host_name,
            host_bio_zh: detail.host_bio_zh,
            host_bio_en: detail.host_bio_en,
            created_at: detail.created_at,
        }))
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = space_id;
        Err(ServerFnError::new("server only"))
    }
}

#[allow(clippy::too_many_arguments)]
#[server(CreateSpace, "/inspace/api")]
pub async fn create_space(
    name_zh: String,
    name_en: Option<String>,
    country: Option<String>,
    province: String,
    city: String,
    district: Option<String>,
    spot_name: Option<String>,
    address_line: Option<String>,
    lat: f64,
    lng: f64,
    space_type: SpaceType,
    custom_type: Option<String>,
    description_zh: Option<String>,
    description_en: Option<String>,
    tag_zh: Option<String>,
    tag_en: Option<String>,
    is_public: bool,
    duration_hours: i32,
) -> Result<SpaceMarker, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let Some(user) = crate::server::auth::current_session().await? else {
            return Err(ServerFnError::new("login required"));
        };

        if name_zh.trim().is_empty() || province.trim().is_empty() || city.trim().is_empty() {
            return Err(ServerFnError::new("name, province, and city are required"));
        }
        if !lat.is_finite() || !lng.is_finite() {
            return Err(ServerFnError::new("coordinates must be valid numbers"));
        }
        if !(1..=24 * 30).contains(&duration_hours) {
            return Err(ServerFnError::new(
                "duration must be between 1 and 720 hours",
            ));
        }

        let pool = crate::server::db_pool().await?;
        let password = generate_password_code();
        let password_hash =
            hash_password(&password).map_err(|err| ServerFnError::new(err.to_string()))?;
        let hotspot = hotspot_name(&password).map_err(|err| ServerFnError::new(err.to_string()))?;
        let clean_name_zh = name_zh.trim().to_string();
        let clean_name_en = name_en.and_then(|value| {
            let trimmed = value.trim().to_string();
            (!trimmed.is_empty()).then_some(trimmed)
        });
        let clean_province = province.trim().to_string();
        let clean_city = city.trim().to_string();
        let clean_country = clean_optional(country);
        let clean_district = clean_optional(district);
        let clean_spot_name = clean_optional(spot_name);
        let clean_address_line = clean_optional(address_line);
        let clean_custom_type = clean_optional(custom_type);
        let clean_description_zh = clean_optional(description_zh);
        let clean_description_en = clean_optional(description_en);
        let clean_tag_zh = clean_optional(tag_zh);
        let clean_tag_en = clean_optional(tag_en);
        let marker_space_type = space_type.clone();
        let created = create_host_space(
            &pool,
            CreateSpaceInput {
                name_zh: clean_name_zh,
                name_en: clean_name_en.clone(),
                country: clean_country.clone(),
                province: clean_province.clone(),
                city: clean_city.clone(),
                district: clean_district.clone(),
                spot_name: clean_spot_name.clone(),
                address_line: clean_address_line.clone(),
                lat,
                lng,
                space_type,
                custom_type: clean_custom_type,
                description_zh: clean_description_zh,
                description_en: clean_description_en,
                tag_zh: clean_tag_zh,
                tag_en: clean_tag_en,
                is_public,
                duration_hours,
                password_hash,
                host_user_id: user.id,
            },
        )
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;

        Ok(SpaceMarker {
            id: created.id.to_string(),
            name_zh: created.name_zh,
            name_en: clean_name_en,
            space_type: marker_space_type,
            country: clean_country,
            province: Some(clean_province),
            city: Some(clean_city),
            district: clean_district,
            spot_name: clean_spot_name,
            address_line: clean_address_line,
            lat,
            lng,
            is_public,
            status: "active".to_string(),
            expires_at: None,
            online_count: 0,
            home_weight: 0,
            generated_password: Some(password),
            hotspot_name: Some(hotspot),
        })
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("server only"))
    }
}

#[allow(clippy::too_many_arguments)]
#[server(UpdateMySpace, "/inspace/api")]
pub async fn update_my_space(
    space_id: String,
    name_zh: String,
    name_en: Option<String>,
    country: Option<String>,
    province: String,
    city: String,
    district: Option<String>,
    spot_name: Option<String>,
    address_line: Option<String>,
    lat: f64,
    lng: f64,
    is_public: bool,
    custom_type: Option<String>,
    description_zh: Option<String>,
    description_en: Option<String>,
    tag_zh: Option<String>,
    tag_en: Option<String>,
    host_bio_zh: Option<String>,
    host_bio_en: Option<String>,
) -> Result<SpaceMarker, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let space_uuid = parse_space_id(&space_id)?;
        let user = require_space_manager(space_uuid).await?;
        let pool = crate::server::db_pool().await?;
        ensure_space_manager(&pool, space_uuid, &user).await?;

        if name_zh.trim().is_empty() || province.trim().is_empty() || city.trim().is_empty() {
            return Err(ServerFnError::new("name, province, and city are required"));
        }
        if !lat.is_finite() || !lng.is_finite() {
            return Err(ServerFnError::new("coordinates must be valid numbers"));
        }

        let updated = update_host_space(
            &pool,
            space_uuid,
            UpdateSpaceInput {
                name_zh: name_zh.trim().to_string(),
                name_en: clean_optional(name_en),
                country: clean_optional(country),
                province: province.trim().to_string(),
                city: city.trim().to_string(),
                district: clean_optional(district),
                spot_name: clean_optional(spot_name),
                address_line: clean_optional(address_line),
                lat,
                lng,
                is_public,
                custom_type: clean_optional(custom_type),
                description_zh: clean_optional(description_zh),
                description_en: clean_optional(description_en),
                tag_zh: clean_optional(tag_zh),
                tag_en: clean_optional(tag_en),
                host_bio_zh: clean_optional(host_bio_zh),
                host_bio_en: clean_optional(host_bio_en),
            },
        )
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;

        Ok(to_marker(updated))
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("server only"))
    }
}

#[server(CloseMySpace, "/inspace/api")]
pub async fn close_my_space(space_id: String) -> Result<SpaceMarker, ServerFnError> {
    set_my_space_status(space_id, SpaceStatus::Closed, "close_space").await
}

#[server(ReactivateMySpace, "/inspace/api")]
pub async fn reactivate_my_space(space_id: String) -> Result<SpaceMarker, ServerFnError> {
    set_my_space_status(space_id, SpaceStatus::Active, "reactivate_space").await
}

#[server(DeleteMySpace, "/inspace/api")]
pub async fn delete_my_space(space_id: String) -> Result<SpaceMarker, ServerFnError> {
    set_my_space_status(space_id, SpaceStatus::Archived, "delete_space").await
}

#[server(ArchiveMySpaceTemplate, "/inspace/api")]
pub async fn archive_my_space_template(space_id: String) -> Result<SpaceMarker, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let space_uuid = parse_space_id(&space_id)?;
        let user = require_space_manager(space_uuid).await?;
        let pool = crate::server::db_pool().await?;
        ensure_space_manager(&pool, space_uuid, &user).await?;
        archive_template(&pool, space_uuid)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?;

        let updated = set_space_status(&pool, space_uuid, SpaceStatus::Template)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?;
        let _ = instant_db::admin::record_audit(
            &pool,
            Some(user.id),
            Some(&user.email),
            "archive_space_template",
            "space",
            Some(&space_id),
            Some("archived as reusable template"),
        );
        Ok(to_marker(updated))
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("server only"))
    }
}

#[server(SetAdminHomeWeight, "/inspace/api")]
pub async fn set_admin_home_weight(
    space_id: String,
    home_weight: i32,
) -> Result<SpaceMarker, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::server::auth::require_admin_user().await?;
        if !(0..=1000).contains(&home_weight) {
            return Err(ServerFnError::new("home weight must be between 0 and 1000"));
        }
        let pool = crate::server::db_pool().await?;
        let updated = set_home_weight(&pool, parse_space_id(&space_id)?, home_weight)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?;
        Ok(to_marker(updated))
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (space_id, home_weight);
        Err(ServerFnError::new("server only"))
    }
}

#[server(ApplyMySpaceResident, "/inspace/api")]
pub async fn apply_my_space_resident(space_id: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let space_uuid = parse_space_id(&space_id)?;
        let user = require_space_manager(space_uuid).await?;
        let pool = crate::server::db_pool().await?;
        ensure_space_manager(&pool, space_uuid, &user).await?;
        apply_resident(&pool, space_uuid)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?;
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PasswordRotationResult {
    pub password: String,
    pub hotspot_name: String,
    pub password_version: i32,
}

#[server(RegenerateSpacePassword, "/inspace/api")]
pub async fn regenerate_space_password(
    space_id: String,
) -> Result<PasswordRotationResult, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let space_uuid = parse_space_id(&space_id)?;
        let user = require_space_manager(space_uuid).await?;
        let pool = crate::server::db_pool().await?;
        ensure_space_manager(&pool, space_uuid, &user).await?;
        let password = generate_password_code();
        let password_hash =
            hash_password(&password).map_err(|err| ServerFnError::new(err.to_string()))?;
        let password_version = rotate_space_password(&pool, space_uuid, password_hash)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?;
        let hotspot_name =
            hotspot_name(&password).map_err(|err| ServerFnError::new(err.to_string()))?;
        let _ = instant_db::admin::record_audit(
            &pool,
            Some(user.id),
            Some(&user.email),
            "regenerate_space_password",
            "space",
            Some(&space_id),
            Some(&format!("password_version -> {password_version}")),
        );

        Ok(PasswordRotationResult {
            password,
            hotspot_name,
            password_version,
        })
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("server only"))
    }
}

#[cfg(feature = "ssr")]
async fn set_my_space_status(
    space_id: String,
    status: SpaceStatus,
    action: &'static str,
) -> Result<SpaceMarker, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let space_uuid = parse_space_id(&space_id)?;
        let user = require_space_manager(space_uuid).await?;
        let pool = crate::server::db_pool().await?;
        ensure_space_manager(&pool, space_uuid, &user).await?;
        let updated = set_space_status(&pool, space_uuid, status)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?;
        let _ = instant_db::admin::record_audit(
            &pool,
            Some(user.id),
            Some(&user.email),
            action,
            "space",
            Some(&space_id),
            Some(&format!("status -> {:?}", updated.status)),
        );
        Ok(to_marker(updated))
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("server only"))
    }
}

#[server(ApplyHostClaim, "/inspace/api")]
pub async fn apply_host_claim(
    space_id: String,
    message: Option<String>,
) -> Result<HostClaimState, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let space_uuid = parse_space_id(&space_id)?;
        let Some(user) = crate::server::auth::current_session().await? else {
            return Err(ServerFnError::new("login required"));
        };
        let pool = crate::server::db_pool().await?;
        let accepted = instant_db::spaces::apply_host_claim(
            &pool,
            space_uuid,
            user.id,
            clean_optional(message),
        )
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;

        if !accepted {
            return Ok(HostClaimState::AlreadyHosted);
        }
        Ok(HostClaimState::Pending)
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (space_id, message);
        Err(ServerFnError::new("server only"))
    }
}

#[server(MyHostClaim, "/inspace/api")]
pub async fn my_host_claim(space_id: String) -> Result<HostClaimState, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let space_uuid = parse_space_id(&space_id)?;
        let Some(user) = crate::server::auth::current_session().await? else {
            return Ok(HostClaimState::Anonymous);
        };
        let pool = crate::server::db_pool().await?;
        let status = instant_db::spaces::host_claim_status(&pool, space_uuid, user.id)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?;
        Ok(match status.as_deref() {
            Some("pending") => HostClaimState::Pending,
            Some("approved") => HostClaimState::Approved,
            Some("rejected") => HostClaimState::Rejected,
            _ => HostClaimState::None,
        })
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = space_id;
        Err(ServerFnError::new("server only"))
    }
}

#[cfg(feature = "ssr")]
fn clean_optional(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim().to_string();
        (!trimmed.is_empty()).then_some(trimmed)
    })
}

#[cfg(feature = "ssr")]
fn parse_space_id(space_id: &str) -> Result<uuid::Uuid, ServerFnError> {
    uuid::Uuid::parse_str(space_id).map_err(|_| ServerFnError::new("invalid space id"))
}

#[cfg(feature = "ssr")]
async fn require_space_manager(
    _space_id: uuid::Uuid,
) -> Result<instant_domain::auth::CurrentUser, ServerFnError> {
    crate::server::auth::current_session()
        .await?
        .ok_or_else(|| ServerFnError::new("login required"))
}

#[cfg(feature = "ssr")]
async fn ensure_space_manager(
    pool: &sqlx::PgPool,
    space_id: uuid::Uuid,
    user: &instant_domain::auth::CurrentUser,
) -> Result<(), ServerFnError> {
    if user.role.is_admin() {
        return Ok(());
    }

    let owner = space_host_user_id(pool, space_id)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;
    if owner == Some(user.id) {
        Ok(())
    } else {
        Err(ServerFnError::new("space owner permission required"))
    }
}

fn space_status_key(status: &SpaceStatus) -> &'static str {
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
    use crate::server::spaces::to_marker;
    use instant_domain::spaces::{SpaceStatus, SpaceSummary, SpaceType};
    use uuid::Uuid;

    #[test]
    fn map_marker_payload_hides_private_descriptions() {
        let summary = SpaceSummary {
            id: Uuid::nil(),
            name_zh: "私密茶室".to_string(),
            name_en: None,
            space_type: SpaceType::Food,
            country: Some("中国".to_string()),
            province: Some("浙江省".to_string()),
            city: Some("杭州市".to_string()),
            district: None,
            spot_name: None,
            address_line: None,
            lat: 30.2496,
            lng: 120.1303,
            is_public: false,
            status: SpaceStatus::Active,
            expires_at: None,
            online_count: 0,
            home_weight: 0,
        };

        let marker = to_marker(summary);
        assert!(!marker.is_public);
        assert_eq!(marker.name_zh, "私密茶室");
    }
}

#[server(ListMySpaceMembers, "/inspace/api")]
pub async fn list_my_space_members(
    space_id: String,
) -> Result<Vec<instant_domain::spaces::SpaceMember>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let space_uuid = parse_space_id(&space_id)?;
        let user = require_space_manager(space_uuid).await?;
        let pool = crate::server::db_pool().await?;
        ensure_space_manager(&pool, space_uuid, &user).await?;
        instant_db::spaces::list_space_members(&pool, space_uuid)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("server only"))
    }
}

#[server(AddMySpaceMember, "/inspace/api")]
pub async fn add_my_space_member(
    space_id: String,
    email: String,
    role: String,
) -> Result<instant_domain::spaces::SpaceMember, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let space_uuid = parse_space_id(&space_id)?;
        let user = require_space_manager(space_uuid).await?;
        let pool = crate::server::db_pool().await?;
        ensure_space_manager(&pool, space_uuid, &user).await?;
        let member_user_id =
            instant_db::spaces::find_user_id_by_email(&pool, email.trim())
                .await
                .map_err(|err| ServerFnError::new(err.to_string()))?
                .ok_or_else(|| ServerFnError::new("no user with that email"))?;
        let member = instant_db::spaces::set_space_member(&pool, space_uuid, member_user_id, role.trim())
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?;
        let _ = instant_db::admin::record_audit(
            &pool,
            Some(user.id),
            Some(&user.email),
            "add_space_member",
            "space",
            Some(&space_id),
            Some(&format!("member {} role {}", email.trim(), member.role)),
        );
        Ok(member)
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("server only"))
    }
}

#[server(RemoveMySpaceMember, "/inspace/api")]
pub async fn remove_my_space_member(
    space_id: String,
    email: String,
) -> Result<bool, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let space_uuid = parse_space_id(&space_id)?;
        let user = require_space_manager(space_uuid).await?;
        let pool = crate::server::db_pool().await?;
        ensure_space_manager(&pool, space_uuid, &user).await?;
        let member_user_id =
            instant_db::spaces::find_user_id_by_email(&pool, email.trim())
                .await
                .map_err(|err| ServerFnError::new(err.to_string()))?;
        let removed = match member_user_id {
            Some(id) => instant_db::spaces::remove_space_member(&pool, space_uuid, id)
                .await
                .map_err(|err| ServerFnError::new(err.to_string()))?,
            None => false,
        };
        if removed {
            let _ = instant_db::admin::record_audit(
                &pool,
                Some(user.id),
                Some(&user.email),
                "remove_space_member",
                "space",
                Some(&space_id),
                Some(&format!("member {}", email.trim())),
            );
        }
        Ok(removed)
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("server only"))
    }
}
