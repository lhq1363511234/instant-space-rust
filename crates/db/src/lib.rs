pub mod admin;
pub mod agent_api;
pub mod chat;
pub mod geo;
pub mod guides;
pub mod lives;
pub mod locations;
pub mod site;
pub mod spaces;
pub mod traces;
pub mod users;
pub mod world;

pub async fn connect(database_url: &str) -> Result<sqlx::PgPool, sqlx::Error> {
    sqlx::PgPool::connect(database_url).await
}

/// Run all embedded migrations against the given pool. This is the single
/// source of truth for the production schema: deploys run this before the
/// service starts serving, and the `_sqlx_migrations` table records applied
/// versions so code and database stay in sync.
pub async fn run_migrations(pool: &sqlx::PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}

#[cfg(test)]
mod tests {
    #[test]
    fn migration_files_are_embedded() {
        let migrator = sqlx::migrate!("./migrations");
        assert!(migrator.iter().count() >= 2);
    }
}

/// Schema contract checked at boot. Migration records can claim success while a
/// table or column is actually missing (this bit production once with
/// `admin_audit_log`). The contract is a curated, minimal set of "if the app
/// runs, these must exist" entries — not a full schema mirror.
const SCHEMA_CONTRACT: &[(&str, &[&str])] = &[
    ("users", &["id", "email", "name", "role", "created_at"]),
    ("sessions", &["id", "user_id", "token_hash", "expires_at"]),
    (
        "spaces",
        &[
            "id",
            "name_zh",
            "space_type",
            "lat",
            "lng",
            "is_public",
            "password_hash",
            "password_version",
            "status",
            "host_user_id",
            "created_at",
            "host_governance_state",
        ],
    ),
    (
        "guides",
        &[
            "id",
            "title_zh",
            "province",
            "city",
            "status",
            "sections",
            "cover_image_url",
            "created_at",
        ],
    ),
    (
        "chat_messages",
        &["id", "space_id", "sender", "body", "kind", "created_at"],
    ),
    (
        "access_sessions",
        &[
            "id",
            "space_id",
            "token_hash",
            "password_version",
            "expires_at",
        ],
    ),
    ("space_traces", &["id", "space_id"]),
    ("space_capsules", &["id", "space_id"]),
    ("helps", &["id", "space_id", "body", "resolved_at"]),
    (
        "guide_versions",
        &["id", "guide_id", "version_no", "title_zh"],
    ),
    ("space_members", &["id", "space_id", "user_id", "role"]),
    (
        "cloud_homes",
        &["id", "owner_id", "space_id", "name", "created_at"],
    ),
    (
        "scenes",
        &["id", "space_id", "kind", "status", "is_default"],
    ),
    (
        "scene_objects",
        &["id", "scene_id", "object_kind", "config"],
    ),
    (
        "scene_spawn_points",
        &["id", "scene_id", "key", "is_default"],
    ),
    (
        "space_relations",
        &["id", "source_space_id", "target_space_id", "relation_kind"],
    ),
    (
        "space_host_tenures",
        &["id", "space_id", "user_id", "role", "status"],
    ),
    (
        "space_governance_events",
        &["id", "space_id", "actor_id", "action", "created_at"],
    ),
    (
        "world_presences",
        &["id", "subject_kind", "subject_id", "space_id", "scene_id"],
    ),
    (
        "space_entry_events",
        &["id", "space_id", "scene_id", "entry_method"],
    ),
    (
        "admin_audit_log",
        &["id", "actor_id", "action", "target_type", "created_at"],
    ),
    (
        "site_page_configs",
        &["page_key", "published_config", "published_version"],
    ),
    (
        "agent_api_keys",
        &[
            "id",
            "user_id",
            "key_prefix",
            "key_hash",
            "scopes",
            "rate_limit_per_minute",
        ],
    ),
];

/// Verify the boot schema contract after migrations run. Returns the list of
/// missing tables/columns (empty means healthy). The service logs a loud error
/// on drift instead of pretending the data layer is fine.
pub async fn verify_schema_contract(pool: &sqlx::PgPool) -> Result<Vec<String>, sqlx::Error> {
    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public'",
    )
    .fetch_all(pool)
    .await?;

    let mut missing = Vec::new();
    for (table, columns) in SCHEMA_CONTRACT {
        if !tables.iter().any(|existing| existing == table) {
            missing.push(format!("missing table: {table}"));
            continue;
        }
        let existing_columns: Vec<String> = sqlx::query_scalar(
            "SELECT column_name FROM information_schema.columns WHERE table_schema = 'public' AND table_name = $1",
        )
        .bind(table)
        .fetch_all(pool)
        .await?;
        for column in *columns {
            if !existing_columns.iter().any(|existing| existing == column) {
                missing.push(format!("missing column: {table}.{column}"));
            }
        }
    }
    Ok(missing)
}
