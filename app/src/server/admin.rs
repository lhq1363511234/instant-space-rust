use instant_domain::admin::{AdminStats, AdminUser, AuditLogEntry, ResidentApplication};
use leptos::prelude::*;

#[server(GetAdminStats, "/inspace/api")]
pub async fn get_admin_stats() -> Result<AdminStats, ServerFnError> {
    #[cfg(feature = "ssr")]
    crate::server::auth::require_admin_user().await?;

    let pool = crate::server::db_pool().await?;

    instant_db::admin::stats(&pool)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))
}

#[server(ListResidentApplications, "/inspace/api")]
pub async fn list_resident_applications() -> Result<Vec<ResidentApplication>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::server::auth::require_admin_user().await?;
        let pool = crate::server::db_pool().await?;
        instant_db::spaces::list_resident_applications(&pool)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("server only"))
    }
}

#[server(ApproveResidentApplication, "/inspace/api")]
pub async fn approve_resident_application(
    space_id: String,
    resident_days: i32,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let actor = crate::server::auth::require_admin_user().await?;
        let space_uuid = uuid::Uuid::parse_str(&space_id)
            .map_err(|_| ServerFnError::new("invalid space id"))?;
        let days = resident_days.clamp(1, 3650);
        let pool = crate::server::db_pool().await?;
        instant_db::spaces::approve_resident_application(&pool, space_uuid, days)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?;
        let _ = instant_db::admin::record_audit(
            &pool,
            Some(actor.id),
            Some(&actor.email),
            "approve_resident",
            "space",
            Some(&space_id),
            Some(&format!("resident_days={days}")),
        )
        .await;
        Ok(())
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (space_id, resident_days);
        Err(ServerFnError::new("server only"))
    }
}

#[server(RejectResidentApplication, "/inspace/api")]
pub async fn reject_resident_application(space_id: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let actor = crate::server::auth::require_admin_user().await?;
        let space_uuid = uuid::Uuid::parse_str(&space_id)
            .map_err(|_| ServerFnError::new("invalid space id"))?;
        let pool = crate::server::db_pool().await?;
        instant_db::spaces::reject_resident_application(&pool, space_uuid)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?;
        let _ = instant_db::admin::record_audit(
            &pool,
            Some(actor.id),
            Some(&actor.email),
            "reject_resident",
            "space",
            Some(&space_id),
            None,
        )
        .await;
        Ok(())
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = space_id;
        Err(ServerFnError::new("server only"))
    }
}

#[server(ListAdminUsers, "/inspace/api")]
pub async fn list_admin_users() -> Result<Vec<AdminUser>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::server::auth::require_admin_user().await?;
        let pool = crate::server::db_pool().await?;
        instant_db::users::list_users(&pool)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("server only"))
    }
}

#[server(SetUserRole, "/inspace/api")]
pub async fn set_user_role(user_id: String, role: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use instant_domain::auth::UserRole;

        // Only super_admin may change roles, to prevent admins from escalating each other.
        let current = crate::server::auth::require_admin_user().await?;
        if !matches!(current.role, UserRole::SuperAdmin) {
            return Err(ServerFnError::new("super admin permission required"));
        }

        let user_uuid = uuid::Uuid::parse_str(&user_id)
            .map_err(|_| ServerFnError::new("invalid user id"))?;

        // Guard against self-demotion locking out the last super admin.
        if user_uuid == current.id && role != "super_admin" {
            return Err(ServerFnError::new("cannot change your own super admin role"));
        }

        let role = match role.as_str() {
            "user" => UserRole::User,
            "admin" => UserRole::Admin,
            "super_admin" => UserRole::SuperAdmin,
            _ => return Err(ServerFnError::new("invalid role")),
        };
        let role_key = match role {
            UserRole::User => "user",
            UserRole::Admin => "admin",
            UserRole::SuperAdmin => "super_admin",
        };
        let pool = crate::server::db_pool().await?;
        instant_db::users::set_user_role(&pool, user_uuid, role)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?;

        let _ = instant_db::admin::record_audit(
            &pool,
            Some(current.id),
            Some(current.email.as_str()),
            "set_user_role",
            "user",
            Some(user_id.as_str()),
            Some(role_key),
        )
        .await;

        Ok(())
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (user_id, role);
        Err(ServerFnError::new("server only"))
    }
}

#[server(ListAuditLog, "/inspace/api")]
pub async fn list_audit_log() -> Result<Vec<AuditLogEntry>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::server::auth::require_admin_user().await?;
        let pool = crate::server::db_pool().await?;
        instant_db::admin::list_audit_log(&pool, 200)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("server only"))
    }
}
