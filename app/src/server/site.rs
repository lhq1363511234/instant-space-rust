use instant_domain::site::{HomePageAdminState, HomePageConfig, SitePageVersion};
use leptos::prelude::*;

#[server(GetPublicHomeConfig, "/inspace/api")]
pub async fn get_public_home_config() -> Result<HomePageConfig, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let fallback = HomePageConfig::default();
        let pool = match crate::server::db_pool().await {
            Ok(pool) => pool,
            Err(err) => {
                tracing::warn!(error = %err, "home config database unavailable; using defaults");
                return Ok(fallback);
            }
        };
        match instant_db::site::get_public_home_config(&pool).await {
            Ok(Some(config)) => Ok(config),
            Ok(None) => Ok(fallback),
            Err(err) => {
                tracing::warn!(error = %err, "published home config unavailable; using defaults");
                Ok(fallback)
            }
        }
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("server only"))
    }
}

#[server(GetAdminHomeConfig, "/inspace/api")]
pub async fn get_admin_home_config() -> Result<HomePageAdminState, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::server::auth::require_admin_user().await?;
        let pool = crate::server::db_pool().await?;
        instant_db::site::get_home_admin_state(&pool)
            .await
            .map(|state| state.unwrap_or_default())
            .map_err(|err| ServerFnError::new(err.to_string()))
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("server only"))
    }
}

#[server(SaveHomeDraft, "/inspace/api")]
pub async fn save_home_draft(config: HomePageConfig) -> Result<HomePageAdminState, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let actor = crate::server::auth::require_admin_user().await?;
        let config = validate_home_config(config)?;
        let pool = crate::server::db_pool().await?;
        let state = instant_db::site::save_home_draft(&pool, &config, actor.id)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?;
        let _ = instant_db::admin::record_audit(
            &pool,
            Some(actor.id),
            Some(&actor.email),
            "save_home_draft",
            "site_page",
            Some("home"),
            None,
        )
        .await;
        Ok(state)
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = config;
        Err(ServerFnError::new("server only"))
    }
}

#[server(PublishHomeConfig, "/inspace/api")]
pub async fn publish_home_config(
    config: HomePageConfig,
) -> Result<HomePageAdminState, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let actor = crate::server::auth::require_admin_user().await?;
        let config = validate_home_config(config)?;
        let pool = crate::server::db_pool().await?;
        let state = instant_db::site::publish_home_config(&pool, &config, actor.id, &actor.email)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?;
        let _ = instant_db::admin::record_audit(
            &pool,
            Some(actor.id),
            Some(&actor.email),
            "publish_home",
            "site_page",
            Some("home"),
            Some(&format!("version={}", state.published_version)),
        )
        .await;
        Ok(state)
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = config;
        Err(ServerFnError::new("server only"))
    }
}

#[server(ListHomeVersions, "/inspace/api")]
pub async fn list_home_versions() -> Result<Vec<SitePageVersion>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::server::auth::require_admin_user().await?;
        let pool = crate::server::db_pool().await?;
        instant_db::site::list_home_versions(&pool)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("server only"))
    }
}

#[server(RestoreHomeVersion, "/inspace/api")]
pub async fn restore_home_version(version_id: String) -> Result<HomePageAdminState, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let actor = crate::server::auth::require_admin_user().await?;
        let version_id = uuid::Uuid::parse_str(&version_id)
            .map_err(|_| ServerFnError::new("invalid version id"))?;
        let pool = crate::server::db_pool().await?;
        let state = instant_db::site::restore_home_version_to_draft(&pool, version_id, actor.id)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?;
        let _ = instant_db::admin::record_audit(
            &pool,
            Some(actor.id),
            Some(&actor.email),
            "restore_home_version",
            "site_page_version",
            Some(&version_id.to_string()),
            Some("restored_to_draft"),
        )
        .await;
        Ok(state)
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = version_id;
        Err(ServerFnError::new("server only"))
    }
}

#[cfg(feature = "ssr")]
fn validate_home_config(mut config: HomePageConfig) -> Result<HomePageConfig, ServerFnError> {
    fn trim_limit(value: &mut String, max: usize) {
        *value = value.trim().chars().take(max).collect();
    }

    fn localized(value: &mut instant_domain::site::LocalizedText, max: usize) {
        trim_limit(&mut value.zh, max);
        trim_limit(&mut value.en, max);
    }

    fn safe_url(value: &mut String) -> Result<(), ServerFnError> {
        trim_limit(value, 500);
        let lower = value.to_ascii_lowercase();
        if value.starts_with('/') || lower.starts_with("https://") {
            Ok(())
        } else {
            Err(ServerFnError::new(
                "button links must use an internal path or https URL",
            ))
        }
    }

    fn color(value: &mut String, fallback: &str) {
        trim_limit(value, 7);
        let valid = value.len() == 7
            && value.starts_with('#')
            && value.chars().skip(1).all(|ch| ch.is_ascii_hexdigit());
        if !valid {
            *value = fallback.to_string();
        }
    }

    for text in [
        &mut config.nav.home,
        &mut config.nav.map,
        &mut config.nav.guides,
        &mut config.nav.my_spaces,
        &mut config.hero.primary_label,
        &mut config.hero.secondary_label,
        &mut config.guide.cta_label,
        &mut config.host.cta_label,
    ] {
        localized(text, 80);
    }
    for text in [
        &mut config.seo.title,
        &mut config.hero.eyebrow,
        &mut config.hero.title,
        &mut config.hero.sample_location,
        &mut config.hero.sample_title,
        &mut config.hero.sample_guide_label,
        &mut config.hero.sample_question,
        &mut config.hero.sample_presence,
        &mut config.journey.eyebrow,
        &mut config.journey.title,
        &mut config.journey.arrive_title,
        &mut config.journey.guide_title,
        &mut config.journey.help_title,
        &mut config.guide.eyebrow,
        &mut config.guide.title,
        &mut config.host.title,
    ] {
        localized(text, 180);
    }
    for text in [
        &mut config.seo.description,
        &mut config.hero.body,
        &mut config.hero.note,
        &mut config.hero.sample_body,
        &mut config.journey.body,
        &mut config.journey.arrive_body,
        &mut config.journey.guide_body,
        &mut config.journey.help_body,
        &mut config.guide.body,
        &mut config.guide.visual_route,
        &mut config.guide.visual_warning,
        &mut config.guide.visual_live,
        &mut config.host.body,
    ] {
        localized(text, 1200);
    }

    if config.hero.title.zh.is_empty() || config.hero.title.en.is_empty() {
        return Err(ServerFnError::new("home hero title cannot be empty"));
    }
    safe_url(&mut config.hero.primary_url)?;
    safe_url(&mut config.hero.secondary_url)?;
    safe_url(&mut config.guide.cta_url)?;

    color(&mut config.theme.primary, "#238EE8");
    color(&mut config.theme.deep, "#061A2B");
    color(&mut config.theme.background, "#EDF6FC");
    if !matches!(config.theme.preset.as_str(), "sky-ocean" | "sky" | "ocean") {
        config.theme.preset = "sky-ocean".to_string();
    }
    if !matches!(config.theme.density.as_str(), "comfortable" | "compact") {
        config.theme.density = "comfortable".to_string();
    }
    if !matches!(config.theme.hero_layout.as_str(), "split" | "centered") {
        config.theme.hero_layout = "split".to_string();
    }
    for order in [
        &mut config.hero.order,
        &mut config.journey.order,
        &mut config.guide.order,
        &mut config.host.order,
    ] {
        *order = (*order).clamp(0, 100);
    }
    Ok(config)
}
