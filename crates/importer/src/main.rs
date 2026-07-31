//! Legacy SQLite → PostgreSQL importer for the China Interactive Map era data.
//!
//! Usage:
//!   instant-importer <legacy.db>                     # dry run: counts only
//!   instant-importer <legacy.db> --import --pg POSTGRES_URL   # real import
//!
//! The legacy Prisma schema has drifted across versions, so the importer maps
//! columns dynamically: whichever of the known legacy columns exist are read,
//! and missing ones get safe defaults. This keeps the tool usable against
//! multiple snapshots of the old database.

use anyhow::{bail, Context, Result};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::Row;
use std::collections::HashSet;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let sqlite_path = args
        .first()
        .cloned()
        .unwrap_or_else(|| "../china-interactive-map/prisma/dev.db".to_string());
    let import = args.iter().any(|arg| arg == "--import");
    let pg_url = args
        .iter()
        .position(|arg| arg == "--pg")
        .and_then(|index| args.get(index + 1).cloned());

    let canonical = std::fs::canonicalize(&sqlite_path)
        .with_context(|| format!("cannot find legacy database {sqlite_path}"))?;
    let options = SqliteConnectOptions::new()
        .filename(canonical)
        .read_only(true);
    let sqlite = SqlitePoolOptions::new().connect_with(options).await?;

    let tables = list_tables(&sqlite).await?;
    let legacy_spaces = tables.iter().any(|table| table == "spaces");
    let legacy_guides = tables.iter().any(|table| table == "guides");
    let legacy_users = tables.iter().any(|table| table == "users");

    let space_count = count_rows(&sqlite, "spaces").await?;
    let guide_count = count_rows(&sqlite, "guides").await?;
    let user_count = count_rows(&sqlite, "users").await?;
    println!("legacy tables={tables:?}");
    println!("spaces={space_count} guides={guide_count} users={user_count}");

    if !import {
        println!("dry run: pass --import --pg DATABASE_URL to migrate");
        return Ok(());
    }
    let pg_url =
        pg_url.context("--import requires --pg DATABASE_URL (postgres://user:pass@host/db)")?;
    if !legacy_users && !legacy_spaces && !legacy_guides {
        bail!("legacy database has none of spaces/guides/users; nothing to import");
    }

    let pg_options: PgConnectOptions = pg_url.parse()?;
    let pg = PgPoolOptions::new()
        .max_connections(4)
        .connect_with(pg_options)
        .await
        .context("cannot connect to postgres")?;

    // Idempotency guard: never import into a database that already has real
    // content rows (seed data or a previous run).
    let existing_spaces: i64 = sqlx::query_scalar("SELECT count(*) FROM spaces")
        .fetch_one(&pg)
        .await?;
    if existing_spaces > 0 {
        bail!(
            "target database already has {existing_spaces} spaces; refusing to import (would double data)"
        );
    }

    let mut imported_users = 0usize;
    let mut imported_spaces = 0usize;
    let mut imported_guides = 0usize;

    // Users first so guide author_id / space creator_id can map. The legacy
    // user id is text; we synthesize a stable UUID from it.
    if legacy_users {
        let columns = table_columns(&sqlite, "users").await?;
        let email_ok = columns.contains("email");
        let name_ok = columns.contains("name");
        let password_ok = columns.contains("password") || columns.contains("password_hash");
        if email_ok && password_ok {
            let rows = sqlx::query("SELECT * FROM users").fetch_all(&sqlite).await?;
            for row in rows {
                let id: String = row.try_get("id").unwrap_or_else(|_| Uuid::new_v4().to_string());
                let email: String = row.try_get("email").unwrap_or_else(|_| {
                    format!("legacy-{}@inspace.local", Uuid::new_v4().simple())
                });
                let password: String = if columns.contains("password") {
                    row.try_get("password").unwrap_or_else(|_| "!legacy!".to_string())
                } else {
                    row.try_get("password_hash").unwrap_or_else(|_| "!legacy!".to_string())
                };
                let name: Option<String> = if name_ok {
                    row.try_get("name").ok()
                } else {
                    None
                };
                let user_id = stable_uuid(&id);
                sqlx::query(
                    r#"
                    INSERT INTO users (id, email, name, password_hash)
                    VALUES ($1, $2, $3, $4)
                    ON CONFLICT (id) DO NOTHING
                    "#,
                )
                .bind(user_id)
                .bind(&email)
                .bind(name)
                .bind(placeholder_hash(&password))
                .execute(&pg)
                .await?;
                imported_users += 1;
            }
        } else {
            println!("legacy users table lacks email/password columns; skipping users");
        }
    }

    // Spaces: map legacy columns that exist; synthesize the rest.
    if legacy_spaces {
        let columns = table_columns(&sqlite, "spaces").await?;
        let name_zh = columns.iter().any(|c| c == "name_zh" || c == "name");
        if name_zh {
            let rows = sqlx::query("SELECT * FROM spaces").fetch_all(&sqlite).await?;
            for row in rows {
                let id: String = row.try_get("id").unwrap_or_else(|_| Uuid::new_v4().to_string());
                let name: String = if columns.contains("name_zh") {
                    row.try_get("name_zh").unwrap_or_else(|_| "未命名空间".to_string())
                } else {
                    row.try_get("name").unwrap_or_else(|_| "Unnamed".to_string())
                };
                let name_en: Option<String> = if columns.contains("name_en") {
                    row.try_get("name_en").ok()
                } else {
                    None
                };
                let lat: f64 = row.try_get("lat").unwrap_or(0.0);
                let lng: f64 = row.try_get("lng").unwrap_or(0.0);
                let province: String = get_text(&row, "province").unwrap_or_else(|| "未知".to_string());
                let city: String = get_text(&row, "city").unwrap_or_else(|| "未知".to_string());
                let district: Option<String> = get_text(&row, "district");
                let spot: Option<String> = get_text(&row, "spot_name");
                let space_type: String = get_text(&row, "space_type")
                    .map(|value| normalize_space_type(&value).to_string())
                    .unwrap_or_else(|| "custom".to_string());
                let creator: Option<String> = get_text(&row, "creator_id").or_else(|| get_text(&row, "userId"));
                let space_id = stable_uuid(&id);
                sqlx::query(
                    r#"
                    INSERT INTO spaces (
                        id, name_zh, name_en, space_type, province, city, district,
                        spot_name, lat, lng, is_public, password_hash, duration_hours,
                        status, creator_id
                    )
                    VALUES (
                        $1, $2, $3, $4::space_type, $5, $6, $7, $8, $9, $10,
                        TRUE, $11, 24, 'active', $12
                    )
                    ON CONFLICT (id) DO NOTHING
                    "#,
                )
                .bind(space_id)
                .bind(&name)
                .bind(name_en)
                .bind(&space_type)
                .bind(&province)
                .bind(&city)
                .bind(district)
                .bind(spot)
                .bind(lat)
                .bind(lng)
                .bind(placeholder_hash("legacy"))
                .bind(creator.map(|value| stable_uuid(&value)))
                .execute(&pg)
                .await?;
                imported_spaces += 1;
            }
        } else {
            println!("legacy spaces table lacks a name column; skipping spaces");
        }
    }

    // Guides: best-effort. The old guide rows are mostly text; new guides need
    // province/city/status which are filled from the legacy row when present.
    if legacy_guides {
        let columns = table_columns(&sqlite, "guides").await?;
        let title_ok = columns.iter().any(|c| c == "title" || c == "title_zh");
        if title_ok {
            let rows = sqlx::query("SELECT * FROM guides").fetch_all(&sqlite).await?;
            for row in rows {
                let id: String = row.try_get("id").unwrap_or_else(|_| Uuid::new_v4().to_string());
                let title: String = if columns.contains("title_zh") {
                    row.try_get("title_zh").unwrap_or_else(|_| "(无标题)".to_string())
                } else {
                    row.try_get("title").unwrap_or_else(|_| "(untitled)".to_string())
                };
                let title_en: Option<String> = get_text(&row, "title_en");
                let summary: Option<String> = get_text(&row, "summary")
                    .or_else(|| get_text(&row, "description"));
                let content: Option<String> = get_text(&row, "content")
                    .or_else(|| get_text(&row, "body"));
                let province: String = get_text(&row, "province").unwrap_or_else(|| "未知".to_string());
                let city: String = get_text(&row, "city").unwrap_or_else(|| "未知".to_string());
                let district: Option<String> = get_text(&row, "district");
                let spot: Option<String> = get_text(&row, "spot_name");
                let author: Option<String> = get_text(&row, "author_id");
                let space: Option<String> = get_text(&row, "space_id");
                let guide_id = stable_uuid(&id);
                sqlx::query(
                    r#"
                    INSERT INTO guides (
                        id, title_zh, title_en, summary_zh, content_zh, guide_type,
                        province, city, district, spot_name, status, author_id, space_id
                    )
                    VALUES (
                        $1, $2, $3, $4, $5, 'attraction',
                        $6, $7, $8, $9, 'published', $10, $11
                    )
                    ON CONFLICT (id) DO NOTHING
                    "#,
                )
                .bind(guide_id)
                .bind(&title)
                .bind(title_en)
                .bind(summary)
                .bind(content)
                .bind(&province)
                .bind(&city)
                .bind(district)
                .bind(spot)
                .bind(author.map(|value| stable_uuid(&value)))
                .bind(space.map(|value| stable_uuid(&value)))
                .execute(&pg)
                .await?;
                imported_guides += 1;
            }
        } else {
            println!("legacy guides table lacks a title column; skipping guides");
        }
    }

    println!(
        "import finished: users={imported_users} spaces={imported_spaces} guides={imported_guides}"
    );
    Ok(())
}

/// Map legacy space_type text onto the PostgreSQL enum labels.
fn normalize_space_type(value: &str) -> &'static str {
    match value.to_lowercase().as_str() {
        "scenic" | "景点" | "景区" | "attraction" => "scenic",
        "food" | "餐饮" | "美食" | "restaurant" => "food",
        "park" | "公园" => "park",
        "transit" | "交通" | "station" => "transit",
        "event" | "活动" => "event",
        _ => "custom",
    }
}

/// The legacy `id` values are arbitrary strings; derive a deterministic UUID.
fn stable_uuid(value: &str) -> Uuid {
    Uuid::new_v5(&Uuid::NAMESPACE_URL, value.as_bytes())
}

/// Legacy password hashes are often plain text or unknown algorithm; never
/// accept them as credentials. Mark the row so the user must reset the
/// password before signing in again.
fn placeholder_hash(_legacy: &str) -> String {
    format!("!legacy-needs-reset:{}", Uuid::new_v4().simple())
}

async fn list_tables(pool: &sqlx::SqlitePool) -> Result<Vec<String>> {
    let rows: Vec<(String,)> =
        sqlx::query_as("SELECT name FROM sqlite_master WHERE type = 'table'")
            .fetch_all(pool)
            .await?;
    Ok(rows.into_iter().map(|(name,)| name).collect())
}

async fn table_columns(pool: &sqlx::SqlitePool, table: &str) -> Result<HashSet<String>> {
    let rows: Vec<String> = sqlx::query_scalar("SELECT name FROM pragma_table_info($1)")
        .bind(table)
        .fetch_all(pool)
        .await?;
    Ok(rows.into_iter().collect())
}

async fn count_rows(pool: &sqlx::SqlitePool, table: &str) -> Result<i64> {
    let result = sqlx::query_scalar::<_, i64>(&format!("SELECT COUNT(*) FROM {table}"))
        .fetch_one(pool)
        .await?;
    Ok(result)
}

fn get_text(row: &sqlx::sqlite::SqliteRow, column: &str) -> Option<String> {
    row.try_get::<String, _>(column).ok()
}
