use instant_domain::spaces::{SpaceSummary, SpaceType};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use instant_db::spaces::{list_home_spaces, SpaceFilter};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpaceMarker {
    pub id: String,
    pub name_zh: String,
    pub name_en: Option<String>,
    pub space_type: SpaceType,
    pub province: Option<String>,
    pub city: Option<String>,
    pub district: Option<String>,
    pub lat: f64,
    pub lng: f64,
    pub is_public: bool,
    pub online_count: i32,
}

pub fn to_marker(space: SpaceSummary) -> SpaceMarker {
    SpaceMarker {
        id: space.id.to_string(),
        name_zh: space.name_zh,
        name_en: space.name_en,
        space_type: space.space_type,
        province: space.province,
        city: space.city,
        district: space.district,
        lat: space.lat,
        lng: space.lng,
        is_public: space.is_public,
        online_count: space.online_count,
    }
}

#[server(ListSpaces, "/api")]
pub async fn list_spaces(
    q: Option<String>,
    space_type: Option<SpaceType>,
) -> Result<Vec<SpaceMarker>, ServerFnError> {
    let database_url =
        std::env::var("DATABASE_URL").map_err(|err| ServerFnError::new(err.to_string()))?;
    let pool = instant_db::connect(&database_url)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;
    let rows = list_home_spaces(&pool, SpaceFilter { q, space_type })
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;

    Ok(rows.into_iter().map(to_marker).collect())
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
            province: Some("浙江省".to_string()),
            city: Some("杭州市".to_string()),
            district: None,
            lat: 30.2496,
            lng: 120.1303,
            is_public: false,
            status: SpaceStatus::Active,
            online_count: 0,
        };

        let marker = to_marker(summary);
        assert!(!marker.is_public);
        assert_eq!(marker.name_zh, "私密茶室");
    }
}
