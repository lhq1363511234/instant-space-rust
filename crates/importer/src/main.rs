use anyhow::Result;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

#[tokio::main]
async fn main() -> Result<()> {
    let sqlite_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "../china-interactive-map/prisma/dev.db".to_string());
    let canonical = std::fs::canonicalize(&sqlite_path)?;
    let options = SqliteConnectOptions::new()
        .filename(canonical)
        .read_only(true);
    let pool = SqlitePoolOptions::new().connect_with(options).await?;

    let spaces: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces")
        .fetch_one(&pool)
        .await?;
    let guides: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM guides")
        .fetch_one(&pool)
        .await?;
    let users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await?;

    println!("spaces={spaces}");
    println!("guides={guides}");
    println!("users={users}");
    Ok(())
}
