#![cfg_attr(not(feature = "ssr"), allow(dead_code))]

use instant_domain::traces::{CapsuleOpenResult, CapsuleSummary, SpaceChronicle, Trace};
#[cfg(feature = "ssr")]
use instant_domain::traces::{PresenceProof, CAPSULE_MAX_CHARS, TRACE_MAX_CHARS};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// What the browser can tell us about where the writer is standing.
///
/// `scanned` comes from arriving through the Space QR code, which physically
/// only exists at the location. `lat`/`lng` come from the Geolocation API.
/// Neither is tamper-proof and neither is treated as one: they decide which
/// badge a trace carries, and — for capsules — whether the attempt is even
/// considered.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PresenceClaim {
    pub scanned: bool,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub discord_member: bool,
    /// The access code printed at the place — on the WiFi card, the hotspot
    /// SSID, the sign by the till. Checked against the Space's password hash,
    /// which is why this is the only claim here a visitor cannot fake.
    pub onsite_code: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TracePage {
    pub items: Vec<Trace>,
    pub total: i64,
    pub chronicle: SpaceChronicle,
}

/// How close counts as "here" when writing a trace. Deliberately generous: a
/// phone GPS fix in a city centre is routinely 100m off, and a visitor
/// standing at the far end of a large site is still standing at the site.
#[cfg(feature = "ssr")]
const TRACE_ON_SITE_RADIUS_M: f64 = 800.0;

#[cfg(feature = "ssr")]
fn clean(value: String, max: usize) -> Result<String, ServerFnError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ServerFnError::new("empty"));
    }
    if trimmed.chars().count() > max {
        return Err(ServerFnError::new("too long"));
    }
    Ok(trimmed.to_string())
}

/// Checks the code a visitor read off something physical in the room.
///
/// This deliberately reuses the Space access code that already exists: every
/// Space has a six-digit code, and the host is already told to broadcast it as
/// the hotspot name `InstantSpace_<code>`. Somebody in the room sees it in
/// their WiFi list; somebody at home does not. Verified with Argon2 against the
/// stored hash, so the answer never travels to the browser.
#[cfg(feature = "ssr")]
async fn verify_onsite_code(
    pool: &sqlx::PgPool,
    space_id: uuid::Uuid,
    code: Option<&str>,
) -> Result<bool, ServerFnError> {
    let Some(code) = code.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(false);
    };
    let Some((hash, _version)) = instant_db::spaces::space_password_hash(pool, space_id)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?
    else {
        return Ok(false);
    };
    Ok(instant_auth::verify_password(code, &hash).unwrap_or(false))
}

/// Decides which badge a claim earns, and how far off the writer was.
///
/// A scan wins outright — the code is at the place. Otherwise a coordinate
/// inside the radius earns `Geo`. A Discord-vouched member of a Space that
/// actually has a Discord community earns `Discord`. Everything else is
/// honestly labelled remote.
#[cfg(feature = "ssr")]
async fn judge_presence(
    pool: &sqlx::PgPool,
    space_id: uuid::Uuid,
    claim: &PresenceClaim,
    radius_m: f64,
) -> Result<(PresenceProof, Option<f64>), ServerFnError> {
    let distance = match (claim.lat, claim.lng) {
        (Some(lat), Some(lng)) if lat.is_finite() && lng.is_finite() => {
            match instant_db::traces::space_coordinates(pool, space_id)
                .await
                .map_err(|err| ServerFnError::new(err.to_string()))?
            {
                Some((space_lat, space_lng)) => Some(
                    instant_domain::traces::distance_metres(lat, lng, space_lat, space_lng),
                ),
                None => None,
            }
        }
        _ => None,
    };

    // Checked first, because it is the only claim here the server can actually
    // verify: the visitor had to read it off something in the room, and we
    // compare it against a hash the browser never receives.
    if verify_onsite_code(pool, space_id, claim.onsite_code.as_deref()).await? {
        return Ok((PresenceProof::OnSite, distance));
    }

    // A scan is only an assertion — `?via=qr` is a string anyone can append.
    // We honour it, but not against contradicting evidence: if the browser also
    // handed us coordinates a thousand kilometres away, the scan is a lie and
    // the coordinates are the more embarrassing thing to have volunteered.
    if claim.scanned && !distance.is_some_and(|d| d > radius_m) {
        return Ok((PresenceProof::Scan, distance));
    }

    if distance.is_some_and(|d| d <= radius_m) {
        return Ok((PresenceProof::Geo, distance));
    }

    if claim.discord_member {
        let has_discord = instant_db::traces::space_discord_group(pool, space_id)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?
            .is_some();
        if has_discord {
            return Ok((PresenceProof::Discord, distance));
        }
    }

    Ok((PresenceProof::Remote, distance))
}

/// Confirms a visitor is holding the code that is posted at the place.
///
/// Deliberately says nothing except yes or no: a wrong guess must not reveal
/// how wrong it was. Rate limiting is not applied here because the code is
/// already six digits behind Argon2, and the honest failure mode — a guest
/// mistyping the WiFi password — has to stay forgiving.
#[server(CheckOnsiteCode, "/inspace/api")]
pub async fn check_onsite_code(space_id: String, code: String) -> Result<bool, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let id =
            uuid::Uuid::parse_str(&space_id).map_err(|_| ServerFnError::new("invalid space id"))?;
        let pool = crate::server::db_pool().await?;
        verify_onsite_code(&pool, id, Some(code.as_str())).await
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (space_id, code);
        Err(ServerFnError::new("server only"))
    }
}

#[server(ListTraces, "/inspace/api")]
pub async fn list_traces(
    space_id: String,
    page: i32,
    page_size: i32,
) -> Result<TracePage, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let id =
            uuid::Uuid::parse_str(&space_id).map_err(|_| ServerFnError::new("invalid space id"))?;
        let page_size = page_size.clamp(5, 50);
        let page = page.max(1);
        let pool = crate::server::db_pool().await?;
        let user = crate::server::auth::current_session().await.ok().flatten();
        let is_admin = user.as_ref().is_some_and(|u| u.role.is_admin());

        let (items, total) = instant_db::traces::list_traces(
            &pool,
            id,
            user.as_ref().map(|u| u.id),
            is_admin,
            i64::from(page_size),
            i64::from((page - 1) * page_size),
        )
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;

        let chronicle = instant_db::traces::chronicle(&pool, id)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?;

        Ok(TracePage {
            items,
            total,
            chronicle,
        })
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (space_id, page, page_size);
        Err(ServerFnError::new("server only"))
    }
}

#[server(LeaveTrace, "/inspace/api")]
pub async fn leave_trace(
    space_id: String,
    body: String,
    weather: Option<String>,
    scanned: bool,
    lat: Option<f64>,
    lng: Option<f64>,
    discord_member: bool,
    onsite_code: Option<String>,
    source_message_id: Option<String>,
) -> Result<Trace, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let id =
            uuid::Uuid::parse_str(&space_id).map_err(|_| ServerFnError::new("invalid space id"))?;
        let body = clean(body, TRACE_MAX_CHARS)?;
        let Some(user) = crate::server::auth::current_session().await? else {
            return Err(ServerFnError::new("login required"));
        };
        let pool = crate::server::db_pool().await?;

        let claim = PresenceClaim {
            scanned,
            lat,
            lng,
            discord_member,
            onsite_code,
        };
        let (proof, distance) =
            judge_presence(&pool, id, &claim, TRACE_ON_SITE_RADIUS_M).await?;

        let author_name = user
            .name
            .clone()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| user.email.clone());

        let trace = instant_db::traces::create_trace(
            &pool,
            instant_db::traces::NewTrace {
                space_id: id,
                author_id: Some(user.id),
                author_name,
                body,
                proof,
                proof_lat: lat,
                proof_lng: lng,
                proof_distance_m: distance,
                weather: weather
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty() && value.chars().count() <= 40),
                source_message_id: source_message_id
                    .and_then(|value| uuid::Uuid::parse_str(&value).ok()),
            },
        )
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;

        Ok(trace)
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (
            space_id,
            body,
            weather,
            scanned,
            lat,
            lng,
            discord_member,
            onsite_code,
            source_message_id,
        );
        Err(ServerFnError::new("server only"))
    }
}

#[server(HideTrace, "/inspace/api")]
pub async fn hide_trace(trace_id: String) -> Result<bool, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let id =
            uuid::Uuid::parse_str(&trace_id).map_err(|_| ServerFnError::new("invalid trace id"))?;
        let Some(user) = crate::server::auth::current_session().await? else {
            return Err(ServerFnError::new("login required"));
        };
        let pool = crate::server::db_pool().await?;
        instant_db::traces::hide_trace(&pool, id, user.id, user.role.is_admin())
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = trace_id;
        Err(ServerFnError::new("server only"))
    }
}

#[server(ListCapsules, "/inspace/api")]
pub async fn list_capsules(space_id: String) -> Result<Vec<CapsuleSummary>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let id =
            uuid::Uuid::parse_str(&space_id).map_err(|_| ServerFnError::new("invalid space id"))?;
        let pool = crate::server::db_pool().await?;
        let user = crate::server::auth::current_session().await.ok().flatten();
        instant_db::traces::list_capsules(&pool, id, user.map(|u| u.id))
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = space_id;
        Err(ServerFnError::new("server only"))
    }
}

#[server(SealCapsule, "/inspace/api")]
pub async fn seal_capsule(
    space_id: String,
    recipient_hint: String,
    body: String,
    passphrase: String,
    radius_m: i32,
    opens_at: Option<String>,
) -> Result<String, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let id =
            uuid::Uuid::parse_str(&space_id).map_err(|_| ServerFnError::new("invalid space id"))?;
        let recipient_hint = clean(recipient_hint, 80)?;
        let body = clean(body, CAPSULE_MAX_CHARS)?;
        let passphrase = passphrase.trim().to_string();
        if passphrase.chars().count() < 2 {
            return Err(ServerFnError::new("passphrase too short"));
        }

        let Some(user) = crate::server::auth::current_session().await? else {
            return Err(ServerFnError::new("login required"));
        };
        let pool = crate::server::db_pool().await?;

        // Only the hash is stored. The author tells the recipient the words
        // themselves; nobody, including this server, can recover them.
        let passphrase_hash = instant_auth::hash_password(&passphrase)
            .map_err(|err| ServerFnError::new(err.to_string()))?;

        let opens_at = opens_at
            .filter(|value| !value.trim().is_empty())
            .and_then(|value| {
                // The date input hands back `YYYY-MM-DD`; treat it as midnight UTC.
                let date = time::Date::parse(
                    value.trim(),
                    &time::macros::format_description!("[year]-[month]-[day]"),
                )
                .ok()?;
                Some(date.midnight().assume_utc())
            });

        let author_name = user
            .name
            .clone()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| user.email.clone());

        let capsule_id = instant_db::traces::create_capsule(
            &pool,
            instant_db::traces::NewCapsule {
                space_id: id,
                author_id: Some(user.id),
                author_name,
                recipient_hint,
                body,
                passphrase_hash,
                radius_m: radius_m.clamp(50, 5000),
                opens_at,
            },
        )
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;

        Ok(capsule_id.to_string())
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (space_id, recipient_hint, body, passphrase, radius_m, opens_at);
        Err(ServerFnError::new("server only"))
    }
}

#[server(OpenCapsule, "/inspace/api")]
pub async fn open_capsule(
    capsule_id: String,
    passphrase: String,
    lat: Option<f64>,
    lng: Option<f64>,
    scanned: bool,
    onsite_code: Option<String>,
) -> Result<CapsuleOpenResult, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let id = uuid::Uuid::parse_str(&capsule_id)
            .map_err(|_| ServerFnError::new("invalid capsule id"))?;
        let pool = crate::server::db_pool().await?;
        let Some(challenge) = instant_db::traces::capsule_challenge(&pool, id)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?
        else {
            return Err(ServerFnError::new("capsule not found"));
        };

        if let (Some(opened_at), name) = (challenge.opened_at, challenge.opened_by_name.clone()) {
            return Ok(CapsuleOpenResult::AlreadyOpened {
                opened_at: opened_at.date().to_string(),
                opened_by_name: name,
            });
        }

        if instant_db::traces::is_locked(challenge.failed_attempts) {
            return Ok(CapsuleOpenResult::Locked);
        }

        if let Some(opens_at) = challenge.opens_at {
            if opens_at > time::OffsetDateTime::now_utc() {
                return Ok(CapsuleOpenResult::NotYet {
                    opens_at: opens_at.date().to_string(),
                });
            }
        }

        // Being there is not optional. A scan proves it outright; otherwise we
        // need coordinates, and they have to be close enough.
        let distance = match (lat, lng) {
            (Some(lat), Some(lng)) if lat.is_finite() && lng.is_finite() => Some(
                instant_domain::traces::distance_metres(
                    lat,
                    lng,
                    challenge.space_lat,
                    challenge.space_lng,
                ),
            ),
            _ => None,
        };

        // The on-site code is the only proof of presence that cannot be forged
        // by editing the request: the visitor had to read it off something in
        // the room, and it is checked against a hash they never see. It alone
        // is enough.
        let code_ok = verify_onsite_code(&pool, challenge.space_id, onsite_code.as_deref()).await?;

        if !code_ok {
            // A scan is merely asserted, so it cannot outrank coordinates that
            // contradict it — otherwise appending `?via=qr` would open every
            // capsule on the site from anywhere in the world.
            let too_far = distance.is_some_and(|d| d > f64::from(challenge.radius_m));

            if too_far {
                return Ok(CapsuleOpenResult::TooFar {
                    distance_m: distance.unwrap_or_default().round(),
                    radius_m: challenge.radius_m,
                });
            }

            if !scanned && distance.is_none() {
                return Ok(CapsuleOpenResult::PresenceRequired);
            }
        }

        let matches = instant_auth::verify_password(passphrase.trim(), &challenge.passphrase_hash)
            .unwrap_or(false);
        if !matches {
            let attempts = instant_db::traces::record_failed_attempt(&pool, id)
                .await
                .map_err(|err| ServerFnError::new(err.to_string()))?;
            if instant_db::traces::is_locked(attempts) {
                return Ok(CapsuleOpenResult::Locked);
            }
            return Ok(CapsuleOpenResult::WrongPassphrase);
        }

        let user = crate::server::auth::current_session().await.ok().flatten();
        let opener_name = user
            .as_ref()
            .and_then(|u| u.name.clone())
            .filter(|value| !value.trim().is_empty())
            .or_else(|| user.as_ref().map(|u| u.email.clone()))
            .unwrap_or_else(|| "一位到访者".to_string());

        let claimed = instant_db::traces::mark_capsule_opened(
            &pool,
            id,
            user.as_ref().map(|u| u.id),
            &opener_name,
        )
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;

        if !claimed {
            // Someone else opened it between the check above and here.
            return Ok(CapsuleOpenResult::AlreadyOpened {
                opened_at: time::OffsetDateTime::now_utc().date().to_string(),
                opened_by_name: None,
            });
        }

        Ok(CapsuleOpenResult::Opened {
            body: challenge.body,
            author_name: challenge.author_name,
            created_at: challenge.created_at.date().to_string(),
        })
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (capsule_id, passphrase, lat, lng, scanned, onsite_code);
        Err(ServerFnError::new("server only"))
    }
}
