pub mod admin;
pub mod chat;
pub mod geo;
pub mod guides;
pub mod locations;
pub mod site;
pub mod spaces;
pub mod users;

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
