use instant_domain::spaces::{SpaceStatus, SpaceSummary, SpaceType};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use instant_auth::{generate_password_code, hash_password};
#[cfg(feature = "ssr")]
use instant_db::spaces::{
    apply_resident, archive_template, create_host_space, get_space_summary, list_home_spaces,
    list_host_spaces, list_manageable_spaces, rotate_space_password, set_space_status,
    space_host_user_id, update_host_space, CreateSpaceInput, SpaceFilter, UpdateSpaceInput,
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
    pub generated_password: Option<String>,
    pub hotspot_name: Option<String>,
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
        generated_password: None,
        hotspot_name: None,
    }
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
        },
    )
    .await
    .map_err(|err| ServerFnError::new(err.to_string()))?;

    Ok(rows.into_iter().map(to_marker).collect())
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
            generated_password: Some(password),
            hotspot_name: Some(hotspot),
        })
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("server only"))
    }
}

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
    set_my_space_status(space_id, SpaceStatus::Closed).await
}

#[server(ReactivateMySpace, "/inspace/api")]
pub async fn reactivate_my_space(space_id: String) -> Result<SpaceMarker, ServerFnError> {
    set_my_space_status(space_id, SpaceStatus::Active).await
}

#[server(DeleteMySpace, "/inspace/api")]
pub async fn delete_my_space(space_id: String) -> Result<SpaceMarker, ServerFnError> {
    set_my_space_status(space_id, SpaceStatus::Archived).await
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
        Ok(to_marker(updated))
    }

    #[cfg(not(feature = "ssr"))]
    {
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
        Ok(to_marker(updated))
    }

    #[cfg(not(feature = "ssr"))]
    {
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
        };

        let marker = to_marker(summary);
        assert!(!marker.is_public);
        assert_eq!(marker.name_zh, "私密茶室");
    }
}
