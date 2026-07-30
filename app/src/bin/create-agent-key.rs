use anyhow::{anyhow, Context};
use instant_auth::{generate_token, hash_password};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let email = args.next().ok_or_else(|| {
        anyhow!("usage: create-agent-key <user-email> [key-name] [comma-separated-scopes]")
    })?;
    let name = args.next().unwrap_or_else(|| "author-agent".to_string());
    let scopes = args
        .next()
        .unwrap_or_else(|| "spaces:read,spaces:write,guides:read,guides:write".to_string())
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    if scopes.is_empty() {
        return Err(anyhow!("at least one scope is required"));
    }

    let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL is required")?;
    let pool = instant_db::connect(&database_url).await?;
    let Some((user_id, _, _)) = instant_db::users::find_user_password_hash(&pool, &email).await?
    else {
        return Err(anyhow!("user not found: {email}"));
    };

    let raw_key = format!("isp_live_{}", generate_token());
    let key_prefix = raw_key.chars().take(16).collect::<String>();
    let key_hash = hash_password(&raw_key)?;
    sqlx::query(
        r#"
        INSERT INTO agent_api_keys (user_id, name, key_prefix, key_hash, scopes)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(user_id)
    .bind(&name)
    .bind(&key_prefix)
    .bind(key_hash)
    .bind(scopes)
    .execute(&pool)
    .await?;

    println!("InSpace Agent API key created for {email} ({name}).");
    println!("Save it now; it will not be shown again:");
    println!("{raw_key}");
    Ok(())
}
