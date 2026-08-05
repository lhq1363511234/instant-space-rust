//! Phase 9 world entry server functions.
//!
//! Reading a Scene and recording Presence are deliberately separate: public
//! visitors may inspect a published Scene, while Presence requires a signed-in
//! user so the user's digital companions can travel with them.

#[cfg(feature = "ssr")]
use instant_domain::world::EntryMethod;
use instant_domain::world::{EnterSpaceOutcome, SceneBundle, WorldPresence};
use leptos::prelude::*;

#[cfg(feature = "ssr")]
use axum::http::HeaderMap;
#[cfg(feature = "ssr")]
use instant_domain::auth::UserRole;
#[cfg(feature = "ssr")]
use uuid::Uuid;

#[cfg(feature = "ssr")]
fn parse_uuid(value: &str, label: &str) -> Result<Uuid, ServerFnError> {
    Uuid::parse_str(value).map_err(|_| ServerFnError::new(format!("invalid {label}")))
}

#[cfg(feature = "ssr")]
fn clean_optional(value: Option<String>, max: usize) -> Result<Option<String>, ServerFnError> {
    value
        .map(|value| {
            let value = value.trim().to_string();
            if value.chars().count() > max {
                return Err(ServerFnError::new("value too long"));
            }
            Ok((!value.is_empty()).then_some(value))
        })
        .transpose()
        .map(Option::flatten)
}

#[cfg(feature = "ssr")]
async fn access_cookie(space_id: Uuid) -> Result<Option<String>, ServerFnError> {
    let headers: HeaderMap = leptos_axum::extract().await?;
    let Some(raw) = headers.get(axum::http::header::COOKIE) else {
        return Ok(None);
    };
    let raw = raw
        .to_str()
        .map_err(|err| ServerFnError::new(err.to_string()))?;
    let name = format!("instant_access_{}", space_id.simple());
    Ok(raw.split(';').find_map(|part| {
        let (key, value) = part.trim().split_once('=')?;
        (key == name).then(|| value.to_string())
    }))
}

/// Returns a verification label suitable for the immutable entry audit event.
#[cfg(feature = "ssr")]
async fn ensure_world_access(
    pool: &sqlx::PgPool,
    space_id: Uuid,
    user: Option<&instant_domain::auth::CurrentUser>,
) -> Result<&'static str, ServerFnError> {
    let Some(meta) = instant_db::spaces::space_access_meta(pool, space_id)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?
    else {
        return Err(ServerFnError::new("space not found"));
    };

    if let Some(user) = user {
        if matches!(user.role, UserRole::Admin | UserRole::SuperAdmin) {
            return Ok("admin");
        }
        if instant_db::world::user_can_manage_scene(pool, space_id, user.id)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?
        {
            return Ok("owner");
        }
    }

    if meta.is_public {
        return Ok("not_required");
    }

    let Some(token) = access_cookie(space_id).await? else {
        return Err(ServerFnError::new("private space access required"));
    };
    let Some(version) = instant_db::chat::has_valid_access_session(pool, space_id, &token)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?
    else {
        return Err(ServerFnError::new("private space access expired"));
    };
    if version != meta.password_version {
        return Err(ServerFnError::new(
            "space password changed; please re-enter password",
        ));
    }
    Ok("verified")
}

#[server(GetSpaceScene, "/inspace/api")]
pub async fn get_space_scene(
    space_id: String,
    scene_id: Option<String>,
) -> Result<SceneBundle, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let space_id = parse_uuid(&space_id, "space id")?;
        let scene_id = scene_id
            .as_deref()
            .map(|id| parse_uuid(id, "scene id"))
            .transpose()?;
        let pool = crate::server::db_pool().await?;
        let user = crate::server::auth::current_session().await?;
        ensure_world_access(&pool, space_id, user.as_ref()).await?;
        if scene_id.is_none() {
            instant_db::world::ensure_default_scene(
                &pool,
                space_id,
                user.as_ref().map(|user| user.id),
            )
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))
        } else {
            instant_db::world::get_scene_bundle(&pool, space_id, scene_id)
                .await
                .map_err(|err| ServerFnError::new(err.to_string()))?
                .ok_or_else(|| ServerFnError::new("scene not found"))
        }
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (space_id, scene_id);
        Err(ServerFnError::new("server only"))
    }
}

#[server(EnterWorldSpace, "/inspace/api")]
pub async fn enter_world_space(
    space_id: String,
    scene_id: Option<String>,
    spawn_key: Option<String>,
    entry_method: String,
    source_space_id: Option<String>,
    source_object_id: Option<String>,
) -> Result<EnterSpaceOutcome, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let Some(user) = crate::server::auth::current_session().await? else {
            return Err(ServerFnError::new("login required to record presence"));
        };
        let space_id = parse_uuid(&space_id, "space id")?;
        let scene_id = scene_id
            .as_deref()
            .map(|id| parse_uuid(id, "scene id"))
            .transpose()?;
        let source_space_id = source_space_id
            .as_deref()
            .map(|id| parse_uuid(id, "source space id"))
            .transpose()?;
        let source_object_id = source_object_id
            .as_deref()
            .map(|id| parse_uuid(id, "source object id"))
            .transpose()?;
        let spawn_key = clean_optional(spawn_key, 64)?;
        let method = EntryMethod::from_db(entry_method.trim());
        let pool = crate::server::db_pool().await?;
        let verification = ensure_world_access(&pool, space_id, Some(&user)).await?;
        instant_db::world::enter_space(
            &pool,
            user.id,
            space_id,
            scene_id,
            spawn_key.as_deref(),
            method,
            source_space_id,
            source_object_id,
            verification,
        )
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (
            space_id,
            scene_id,
            spawn_key,
            entry_method,
            source_space_id,
            source_object_id,
        );
        Err(ServerFnError::new("server only"))
    }
}

#[server(EnsureMyDefaultScene, "/inspace/api")]
pub async fn ensure_my_default_scene(space_id: String) -> Result<SceneBundle, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let Some(user) = crate::server::auth::current_session().await? else {
            return Err(ServerFnError::new("login required"));
        };
        let space_id = parse_uuid(&space_id, "space id")?;
        let pool = crate::server::db_pool().await?;
        let is_admin = matches!(user.role, UserRole::Admin | UserRole::SuperAdmin);
        let can_manage = instant_db::world::user_can_manage_scene(&pool, space_id, user.id)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?;
        if !is_admin && !can_manage {
            return Err(ServerFnError::new("not allowed to manage this scene"));
        }
        instant_db::world::ensure_default_scene(&pool, space_id, Some(user.id))
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = space_id;
        Err(ServerFnError::new("server only"))
    }
}

#[server(GetMyWorldPresence, "/inspace/api")]
pub async fn get_my_world_presence() -> Result<Option<WorldPresence>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let Some(user) = crate::server::auth::current_session().await? else {
            return Ok(None);
        };
        let pool = crate::server::db_pool().await?;
        instant_db::world::get_user_presence(&pool, user.id)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))
    }

    #[cfg(not(feature = "ssr"))]
    {
        Ok(None)
    }
}

#[cfg(feature = "ssr")]
async fn require_governance_actor(
    pool: &sqlx::PgPool,
    space_id: Uuid,
    require_primary: bool,
) -> Result<instant_domain::auth::CurrentUser, ServerFnError> {
    let Some(user) = crate::server::auth::current_session().await? else {
        return Err(ServerFnError::new("login required"));
    };
    if user.role.is_admin() {
        return Ok(user);
    }
    let role = instant_db::world::active_host_role(pool, space_id, user.id)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;
    let allowed = if require_primary {
        matches!(role, Some(instant_domain::world::HostTenureRole::Primary))
    } else {
        role.is_some()
    };
    if !allowed {
        return Err(ServerFnError::new(if require_primary {
            "primary host permission required"
        } else {
            "active host permission required"
        }));
    }
    Ok(user)
}

#[server(GetSpaceHostLineage, "/inspace/api")]
pub async fn get_space_host_lineage(
    space_id: String,
) -> Result<Option<instant_domain::world::SpaceGovernanceSnapshot>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let space_id = parse_uuid(&space_id, "space id")?;
        let pool = crate::server::db_pool().await?;
        let user = crate::server::auth::current_session().await?;
        ensure_world_access(&pool, space_id, user.as_ref()).await?;
        instant_db::world::get_space_governance(
            &pool,
            space_id,
            user.as_ref().map(|user| user.id),
            false,
            user.as_ref().is_some_and(|user| user.role.is_admin()),
        )
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = space_id;
        Err(ServerFnError::new("server only"))
    }
}

#[server(GetMySpaceGovernance, "/inspace/api")]
pub async fn get_my_space_governance(
    space_id: String,
) -> Result<instant_domain::world::SpaceGovernanceSnapshot, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let space_id = parse_uuid(&space_id, "space id")?;
        let pool = crate::server::db_pool().await?;
        let user = require_governance_actor(&pool, space_id, false).await?;
        instant_db::world::get_space_governance(
            &pool,
            space_id,
            Some(user.id),
            true,
            user.role.is_admin(),
        )
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?
        .ok_or_else(|| ServerFnError::new("space not found"))
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = space_id;
        Err(ServerFnError::new("server only"))
    }
}

#[server(AppointSpaceHost, "/inspace/api")]
pub async fn appoint_space_host(
    space_id: String,
    email: String,
    role: String,
    note: Option<String>,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let space_id = parse_uuid(&space_id, "space id")?;
        let email = email.trim();
        if email.is_empty() || email.chars().count() > 254 {
            return Err(ServerFnError::new("valid email required"));
        }
        let note = clean_optional(note, 500)?;
        let pool = crate::server::db_pool().await?;
        let actor = require_governance_actor(&pool, space_id, true).await?;
        let target_user_id = instant_db::spaces::find_user_id_by_email(&pool, email)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?
            .ok_or_else(|| ServerFnError::new("no user with that email"))?;
        if target_user_id == actor.id {
            return Err(ServerFnError::new("you already govern this Space"));
        }
        let role = if role.trim() == "steward" {
            if !actor.role.is_admin() {
                return Err(ServerFnError::new("only admins can appoint a steward"));
            }
            instant_domain::world::HostTenureRole::Steward
        } else {
            instant_domain::world::HostTenureRole::CoHost
        };
        instant_db::world::appoint_supporting_host(
            &pool,
            space_id,
            target_user_id,
            role,
            actor.id,
            note.as_deref(),
        )
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;
        instant_db::admin::record_audit(
            &pool,
            Some(actor.id),
            Some(&actor.email),
            "appoint_space_host",
            "space_governance",
            Some(&space_id.to_string()),
            Some(&format!("role={} target={email}", role.as_db())),
        )
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;
        Ok(())
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = (space_id, email, role, note);
        Err(ServerFnError::new("server only"))
    }
}

#[server(RemoveSpaceHost, "/inspace/api")]
pub async fn remove_space_host(
    space_id: String,
    user_id: String,
    note: Option<String>,
) -> Result<bool, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let space_id = parse_uuid(&space_id, "space id")?;
        let target_user_id = parse_uuid(&user_id, "user id")?;
        let note = clean_optional(note, 500)?;
        let pool = crate::server::db_pool().await?;
        let actor = require_governance_actor(&pool, space_id, true).await?;
        let removed = instant_db::world::remove_supporting_host(
            &pool,
            space_id,
            target_user_id,
            actor.id,
            false,
            note.as_deref(),
        )
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;
        if removed {
            instant_db::admin::record_audit(
                &pool,
                Some(actor.id),
                Some(&actor.email),
                "remove_space_host",
                "space_governance",
                Some(&space_id.to_string()),
                Some(&format!("target={target_user_id}")),
            )
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?;
        }
        Ok(removed)
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = (space_id, user_id, note);
        Err(ServerFnError::new("server only"))
    }
}

#[server(TransferSpaceHost, "/inspace/api")]
pub async fn transfer_space_host(
    space_id: String,
    email: String,
    note: Option<String>,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let space_id = parse_uuid(&space_id, "space id")?;
        let email = email.trim();
        if email.is_empty() || email.chars().count() > 254 {
            return Err(ServerFnError::new("valid successor email required"));
        }
        let note = clean_optional(note, 500)?;
        let pool = crate::server::db_pool().await?;
        let actor = require_governance_actor(&pool, space_id, true).await?;
        let target_user_id = instant_db::spaces::find_user_id_by_email(&pool, email)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?
            .ok_or_else(|| ServerFnError::new("no user with that email"))?;
        if target_user_id == actor.id && !actor.role.is_admin() {
            return Err(ServerFnError::new("successor must be another user"));
        }
        instant_db::world::transfer_primary_host(
            &pool,
            space_id,
            target_user_id,
            actor.id,
            note.as_deref(),
        )
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;
        instant_db::admin::record_audit(
            &pool,
            Some(actor.id),
            Some(&actor.email),
            "transfer_space_host",
            "space_governance",
            Some(&space_id.to_string()),
            Some(&format!("successor={email}")),
        )
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;
        Ok(())
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = (space_id, email, note);
        Err(ServerFnError::new("server only"))
    }
}

#[server(LeaveSpaceHostRole, "/inspace/api")]
pub async fn leave_space_host_role(
    space_id: String,
    note: Option<String>,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let space_id = parse_uuid(&space_id, "space id")?;
        let note = clean_optional(note, 500)?;
        let pool = crate::server::db_pool().await?;
        let Some(actor) = crate::server::auth::current_session().await? else {
            return Err(ServerFnError::new("login required"));
        };
        let role = instant_db::world::active_host_role(&pool, space_id, actor.id)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?
            .ok_or_else(|| ServerFnError::new("you do not hold an active host role"))?;
        if role == instant_domain::world::HostTenureRole::Primary {
            instant_db::world::release_primary_host(
                &pool,
                space_id,
                actor.id,
                instant_domain::world::HostGovernanceState::Recruiting,
                note.as_deref(),
            )
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?;
        } else {
            instant_db::world::remove_supporting_host(
                &pool,
                space_id,
                actor.id,
                actor.id,
                true,
                note.as_deref(),
            )
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?;
        }
        instant_db::admin::record_audit(
            &pool,
            Some(actor.id),
            Some(&actor.email),
            "leave_space_host_role",
            "space_governance",
            Some(&space_id.to_string()),
            None,
        )
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;
        Ok(())
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = (space_id, note);
        Err(ServerFnError::new("server only"))
    }
}

#[server(SetSpaceSystemCare, "/inspace/api")]
pub async fn set_space_system_care(
    space_id: String,
    note: Option<String>,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let space_id = parse_uuid(&space_id, "space id")?;
        let note = clean_optional(note, 500)?;
        let actor = crate::server::auth::require_admin_user().await?;
        let pool = crate::server::db_pool().await?;
        instant_db::world::release_primary_host(
            &pool,
            space_id,
            actor.id,
            instant_domain::world::HostGovernanceState::SystemCare,
            note.as_deref(),
        )
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;
        instant_db::admin::record_audit(
            &pool,
            Some(actor.id),
            Some(&actor.email),
            "set_space_system_care",
            "space_governance",
            Some(&space_id.to_string()),
            note.as_deref(),
        )
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;
        Ok(())
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = (space_id, note);
        Err(ServerFnError::new("server only"))
    }
}

#[server(UpdateSpaceRecruitmentNote, "/inspace/api")]
pub async fn update_space_recruitment_note(
    space_id: String,
    note: Option<String>,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let space_id = parse_uuid(&space_id, "space id")?;
        let note = clean_optional(note, 500)?;
        let pool = crate::server::db_pool().await?;
        let actor = require_governance_actor(&pool, space_id, true).await?;
        instant_db::world::update_recruitment_note(&pool, space_id, actor.id, note.as_deref())
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?;
        Ok(())
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = (space_id, note);
        Err(ServerFnError::new("server only"))
    }
}
