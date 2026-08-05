//! Server functions for digital lives: cloud homes, companions, trails,
//! distilled memorials, and visitor prayers.
//!
//! Design notes (v3 "云上家"):
//! - Every user has one cloud home; friends may visit but entering requires
//!   the home passphrase — a door key, not a Wi-Fi join. Owner never needs it.
//! - Online/offline signal is activity: 24h of activity counts as online.
//!   While the owner is active, living companions travel with them (在侧);
//!   when idle they wait at the cloud home (在家). After death: 追远.
//! - When the owner proves presence at a space, every living companion
//!   automatically records "it was here too".

#[cfg_attr(not(feature = "ssr"), allow(unused_imports))]
use instant_domain::lives::{
    BiographyChapter, CloudHome, Companion, CompanionState, CompanionTrail, DigitalLife,
    LifeMapEntry, LifePrayer, PaginatedLives, PrayerKind,
};
use leptos::prelude::*;

#[cfg(feature = "ssr")]
use time::{Duration, OffsetDateTime};

#[cfg(feature = "ssr")]
const ONLINE_WINDOW: Duration = Duration::hours(24);

#[cfg(feature = "ssr")]
async fn require_user_id() -> Result<uuid::Uuid, ServerFnError> {
    let Some(user) = crate::server::auth::current_session().await? else {
        return Err(ServerFnError::new("login required"));
    };
    Ok(user.id)
}

#[cfg(feature = "ssr")]
async fn touch_activity(pool: &sqlx::PgPool, user_id: uuid::Uuid) {
    let _ = instant_db::lives::touch_last_active(pool, user_id).await;
}

/// The display state of a living companion: following while the owner is
/// active, at home otherwise. Memorials never change.
#[cfg(feature = "ssr")]
async fn with_display_state(pool: &sqlx::PgPool, owner_id: uuid::Uuid, companion: &mut Companion) {
    if companion.state == CompanionState::Memorial {
        return;
    }
    let active = match instant_db::lives::last_active_at(pool, owner_id).await {
        Ok(Some(t)) => t + ONLINE_WINDOW >= OffsetDateTime::now_utc(),
        _ => false,
    };
    companion.state = if active {
        CompanionState::Following
    } else {
        CompanionState::AtHome
    };
}

#[cfg(feature = "ssr")]
fn clean(value: String, max: usize) -> Result<String, ServerFnError> {
    let trimmed = value.trim().to_string();
    if trimmed.chars().count() > max {
        return Err(ServerFnError::new("too long"));
    }
    Ok(trimmed)
}

#[cfg(feature = "ssr")]
fn parse_uuid(value: &str) -> Result<uuid::Uuid, ServerFnError> {
    uuid::Uuid::parse_str(value).map_err(|_| ServerFnError::new("invalid id"))
}

#[server(GetMyCloudHome, "/inspace/api")]
pub async fn get_my_cloud_home() -> Result<CloudHome, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let user_id = require_user_id().await?;
        let pool = crate::server::db_pool().await?;
        touch_activity(&pool, user_id).await;
        instant_db::lives::ensure_cloud_home(&pool, user_id)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("server only"))
    }
}

#[server(UpdateMyCloudHome, "/inspace/api")]
pub async fn update_my_cloud_home(
    name: String,
    motto: Option<String>,
    door_note: Option<String>,
    passphrase: Option<String>,
    clear_passphrase: bool,
) -> Result<CloudHome, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let user_id = require_user_id().await?;
        let name = clean(name, 24)?;
        let motto = motto
            .map(|v| clean(v, 60))
            .transpose()?
            .filter(|v| !v.is_empty());
        let door_note = door_note
            .map(|v| clean(v, 120))
            .transpose()?
            .filter(|v| !v.is_empty());
        let hash = match passphrase {
            Some(p) if !p.trim().is_empty() => {
                let p = clean(p, 40)?;
                if p.chars().count() < 4 {
                    return Err(ServerFnError::new("口令至少四位"));
                }
                Some(
                    instant_auth::hash_password(&p)
                        .map_err(|e| ServerFnError::new(e.to_string()))?,
                )
            }
            _ => None,
        };
        let pool = crate::server::db_pool().await?;
        touch_activity(&pool, user_id).await;
        let _ = instant_db::lives::ensure_cloud_home(&pool, user_id).await?;
        instant_db::lives::update_cloud_home(
            &pool,
            user_id,
            &name,
            motto.as_deref(),
            door_note.as_deref(),
            hash.as_deref(),
            clear_passphrase,
        )
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (name, motto, door_note, passphrase, clear_passphrase);
        Err(ServerFnError::new("server only"))
    }
}

/// A home visit: the door view (public) plus, once the passphrase is right,
/// the companions waiting inside.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CloudHomeVisit {
    pub home: CloudHome,
    pub companions: Vec<Companion>,
    pub entered: bool,
}

/// The door view: name, motto, door note. No companions until you knock.
#[server(GetCloudHome, "/inspace/api")]
pub async fn get_cloud_home(home_id: String) -> Result<CloudHome, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let id = parse_uuid(&home_id)?;
        let pool = crate::server::db_pool().await?;
        instant_db::lives::get_cloud_home_by_id(&pool, id)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?
            .ok_or_else(|| ServerFnError::new("no such home"))
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = home_id;
        Err(ServerFnError::new("server only"))
    }
}

#[server(VisitCloudHome, "/inspace/api")]
pub async fn visit_cloud_home(
    home_id: String,
    passphrase: Option<String>,
) -> Result<CloudHomeVisit, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let id = parse_uuid(&home_id)?;
        let pool = crate::server::db_pool().await?;
        let home = instant_db::lives::get_cloud_home_by_id(&pool, id)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?
            .ok_or_else(|| ServerFnError::new("no such home"))?;

        let owner = home.owner_id;
        let entered = if home.has_passphrase {
            let Some(key) = passphrase else {
                return Ok(CloudHomeVisit {
                    home,
                    companions: vec![],
                    entered: false,
                });
            };
            let stored = instant_db::lives::home_passphrase_hash(&pool, id)
                .await
                .map_err(|err| ServerFnError::new(err.to_string()))?
                .ok_or_else(|| ServerFnError::new("home has no key"))?;
            if !instant_auth::verify_password(&key, &stored)
                .map_err(|e| ServerFnError::new(e.to_string()))?
            {
                // Wrong key: the door stays shut. Not an error — the visitor
                // simply has not entered yet.
                return Ok(CloudHomeVisit {
                    home,
                    companions: vec![],
                    entered: false,
                });
            }
            true
        } else {
            true
        };

        let mut companions = instant_db::lives::list_companions_by_home(&pool, id)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?;
        for c in &mut companions {
            with_display_state(&pool, owner, c).await;
        }
        Ok(CloudHomeVisit {
            home,
            companions,
            entered,
        })
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (home_id, passphrase);
        Err(ServerFnError::new("server only"))
    }
}

#[server(ListMyCompanions, "/inspace/api")]
pub async fn list_my_companions() -> Result<Vec<Companion>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let user_id = require_user_id().await?;
        let pool = crate::server::db_pool().await?;
        touch_activity(&pool, user_id).await;
        let _ = instant_db::lives::ensure_cloud_home(&pool, user_id).await?;
        let mut companions = instant_db::lives::list_companions(&pool, user_id)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?;
        for c in &mut companions {
            with_display_state(&pool, user_id, c).await;
        }
        Ok(companions)
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("server only"))
    }
}

#[server(CreateCompanion, "/inspace/api")]
pub async fn create_companion(
    name: String,
    species: Option<String>,
    breed: Option<String>,
    gender: Option<String>,
    birth_at: Option<String>,
    avatar_url: Option<String>,
) -> Result<Companion, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let user_id = require_user_id().await?;
        let name = clean(name, 24)?;
        if name.is_empty() {
            return Err(ServerFnError::new("给个名字"));
        }
        let birth = birth_at.as_deref().and_then(|v| {
            time::Date::parse(v, &time::format_description::well_known::Iso8601::DATE).ok()
        });
        let pool = crate::server::db_pool().await?;
        touch_activity(&pool, user_id).await;
        let home = instant_db::lives::ensure_cloud_home(&pool, user_id).await?;
        instant_db::lives::create_companion(
            &pool,
            instant_db::lives::NewCompanion {
                owner_id: user_id,
                home_id: home.id,
                subject_type: "pet".to_string(),
                name,
                species: species
                    .map(|v| v.trim().to_string())
                    .filter(|v| !v.is_empty()),
                breed: breed
                    .map(|v| v.trim().to_string())
                    .filter(|v| !v.is_empty()),
                gender: gender
                    .map(|v| v.trim().to_string())
                    .filter(|v| !v.is_empty()),
                birth_at: birth,
                avatar_url: avatar_url
                    .map(|v| v.trim().to_string())
                    .filter(|v| !v.is_empty()),
            },
        )
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (name, species, breed, gender, birth_at, avatar_url);
        Err(ServerFnError::new("server only"))
    }
}

#[server(UpdateCompanion, "/inspace/api")]
pub async fn update_companion(
    companion_id: String,
    name: Option<String>,
    species: Option<String>,
    breed: Option<String>,
    gender: Option<String>,
    birth_at: Option<String>,
    avatar_url: Option<String>,
) -> Result<Companion, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let user_id = require_user_id().await?;
        let id = parse_uuid(&companion_id)?;
        let birth = birth_at.as_deref().and_then(|v| {
            time::Date::parse(v, &time::format_description::well_known::Iso8601::DATE).ok()
        });
        let pool = crate::server::db_pool().await?;
        touch_activity(&pool, user_id).await;
        instant_db::lives::update_companion(
            &pool,
            id,
            user_id,
            name.as_deref(),
            species.as_deref(),
            breed.as_deref(),
            gender.as_deref(),
            birth,
            avatar_url.as_deref(),
        )
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?
        .ok_or_else(|| ServerFnError::new("no such companion"))
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (
            companion_id,
            name,
            species,
            breed,
            gender,
            birth_at,
            avatar_url,
        );
        Err(ServerFnError::new("server only"))
    }
}

#[server(ListCompanionTrails, "/inspace/api")]
pub async fn list_companion_trails(
    companion_id: String,
    limit: Option<i64>,
) -> Result<Vec<CompanionTrail>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let id = parse_uuid(&companion_id)?;
        let pool = crate::server::db_pool().await?;
        instant_db::lives::list_trails(&pool, id, limit.unwrap_or(200).clamp(1, 500))
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (companion_id, limit);
        Err(ServerFnError::new("server only"))
    }
}

/// A handwritten snippet the owner adds to a place, feeding the distillation.
#[server(AddCompanionSnippet, "/inspace/api")]
pub async fn add_companion_snippet(
    companion_id: String,
    place_name: String,
    snippet: String,
    season_hint: Option<String>,
) -> Result<CompanionTrail, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let user_id = require_user_id().await?;
        let id = parse_uuid(&companion_id)?;
        let place_name = clean(place_name, 60)?;
        let snippet = clean(snippet, 120)?;
        if snippet.is_empty() {
            return Err(ServerFnError::new("写一句话"));
        }
        let pool = crate::server::db_pool().await?;
        touch_activity(&pool, user_id).await;
        instant_db::lives::record_trail(
            &pool,
            instant_db::lives::NewTrail {
                companion_id: id,
                owner_id: user_id,
                space_id: None,
                space_name: None,
                place_name: Some(place_name),
                proof: "snippet".to_string(),
                lat: None,
                lng: None,
                snippet: Some(snippet),
                season_hint: season_hint
                    .map(|v| v.trim().to_string())
                    .filter(|v| !v.is_empty()),
            },
        )
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (companion_id, place_name, snippet, season_hint);
        Err(ServerFnError::new("server only"))
    }
}

#[server(CreateDigitalLife, "/inspace/api")]
pub async fn create_digital_life(
    companion_id: String,
    death_at: String,
    epitaph: String,
    biography: Vec<BiographyChapter>,
    inscription: String,
    life_map: Vec<LifeMapEntry>,
    memorial_date: Option<String>,
    distill_version: i32,
) -> Result<DigitalLife, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let user_id = require_user_id().await?;
        let id = parse_uuid(&companion_id)?;
        let death = time::Date::parse(
            &death_at,
            &time::format_description::well_known::Iso8601::DATE,
        )
        .map_err(|_| ServerFnError::new("invalid date"))?;
        let epitaph = clean(epitaph, 120)?;
        let inscription = clean(inscription, 300)?;
        let memorial = memorial_date.as_deref().and_then(|v| {
            time::Date::parse(v, &time::format_description::well_known::Iso8601::DATE).ok()
        });
        let pool = crate::server::db_pool().await?;
        touch_activity(&pool, user_id).await;

        let companion = instant_db::lives::get_companion(&pool, id)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?
            .ok_or_else(|| ServerFnError::new("no such companion"))?;
        if companion.owner_id != user_id {
            return Err(ServerFnError::new("not yours"));
        }
        if instant_db::lives::get_digital_life_by_companion(&pool, id)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?
            .is_some()
        {
            return Err(ServerFnError::new("already distilled"));
        }

        let life = instant_db::lives::create_digital_life(
            &pool,
            instant_db::lives::NewDigitalLife {
                companion_id: id,
                owner_id: user_id,
                subject_type: companion.subject_type.clone(),
                name: companion.name.clone(),
                epitaph,
                biography,
                inscription,
                life_map,
                memorial_date: memorial,
                distill_version: distill_version.max(1),
            },
        )
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;

        let _ = instant_db::lives::mark_memorial(&pool, id, user_id, death).await?;
        Ok(life)
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (
            companion_id,
            death_at,
            epitaph,
            biography,
            inscription,
            life_map,
            memorial_date,
            distill_version,
        );
        Err(ServerFnError::new("server only"))
    }
}

#[server(GetDigitalLife, "/inspace/api")]
pub async fn get_digital_life(life_id: String) -> Result<DigitalLife, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let id = parse_uuid(&life_id)?;
        let pool = crate::server::db_pool().await?;
        let life = instant_db::lives::get_digital_life(&pool, id)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?
            .ok_or_else(|| ServerFnError::new("no such life"))?;
        let _ = instant_db::lives::bump_visitor(&pool, id).await;
        Ok(life)
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = life_id;
        Err(ServerFnError::new("server only"))
    }
}

#[server(ListDigitalLives, "/inspace/api")]
pub async fn list_digital_lives(
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<PaginatedLives, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let pool = crate::server::db_pool().await?;
        instant_db::lives::list_digital_lives(
            &pool,
            limit.unwrap_or(24).clamp(1, 100),
            offset.unwrap_or(0).max(0),
        )
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (limit, offset);
        Err(ServerFnError::new("server only"))
    }
}

#[server(UpdateDigitalLifeContent, "/inspace/api")]
pub async fn update_digital_life_content(
    life_id: String,
    epitaph: String,
    biography: Vec<BiographyChapter>,
    inscription: String,
    life_map: Vec<LifeMapEntry>,
) -> Result<DigitalLife, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let user_id = require_user_id().await?;
        let id = parse_uuid(&life_id)?;
        let pool = crate::server::db_pool().await?;
        touch_activity(&pool, user_id).await;
        instant_db::lives::update_digital_life_content(
            &pool,
            id,
            user_id,
            &epitaph,
            biography,
            &inscription,
            life_map,
        )
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?
        .ok_or_else(|| ServerFnError::new("no such life"))
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (life_id, epitaph, biography, inscription, life_map);
        Err(ServerFnError::new("server only"))
    }
}

#[server(LeavePrayer, "/inspace/api")]
pub async fn leave_prayer(
    life_id: String,
    kind: String,
    message: Option<String>,
) -> Result<LifePrayer, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let id = parse_uuid(&life_id)?;
        let kind = match kind.as_str() {
            "flower" => PrayerKind::Flower,
            "lantern" => PrayerKind::Lantern,
            "word" => PrayerKind::Word,
            _ => PrayerKind::Incense,
        };
        let message = message
            .map(|v| clean(v, 200))
            .transpose()?
            .filter(|v| !v.is_empty());
        let pool = crate::server::db_pool().await?;
        let user = crate::server::auth::current_session().await?;
        let visitor_name = user
            .as_ref()
            .and_then(|u| u.name.clone())
            .filter(|v| !v.trim().is_empty())
            .or_else(|| user.as_ref().map(|u| u.email.clone()))
            .unwrap_or_else(|| "路人".to_string());
        instant_db::lives::add_prayer(
            &pool,
            instant_db::lives::NewPrayer {
                life_id: id,
                visitor_id: user.map(|u| u.id),
                visitor_name,
                kind,
                message,
            },
        )
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (life_id, kind, message);
        Err(ServerFnError::new("server only"))
    }
}

#[server(ListPrayers, "/inspace/api")]
pub async fn list_prayers(
    life_id: String,
    limit: Option<i64>,
) -> Result<Vec<LifePrayer>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let id = parse_uuid(&life_id)?;
        let pool = crate::server::db_pool().await?;
        instant_db::lives::list_prayers(&pool, id, limit.unwrap_or(50).clamp(1, 200))
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (life_id, limit);
        Err(ServerFnError::new("server only"))
    }
}
