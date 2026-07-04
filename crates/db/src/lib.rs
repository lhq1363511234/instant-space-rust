pub mod admin;
pub mod chat;
pub mod guides;
pub mod locations;
pub mod spaces;
pub mod users;

pub async fn connect(database_url: &str) -> Result<sqlx::PgPool, sqlx::Error> {
    sqlx::PgPool::connect(database_url).await
}

#[cfg(test)]
mod tests {
    #[test]
    fn migration_files_are_embedded() {
        let migrator = sqlx::migrate!("./migrations");
        assert!(migrator.iter().count() >= 2);
    }
}
