use instant_domain::traces::{
    CapsuleSummary, PresenceProof, SpaceChronicle, Trace, CAPSULE_MAX_ATTEMPTS,
};
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct NewTrace {
    pub space_id: Uuid,
    pub author_id: Option<Uuid>,
    pub author_name: String,
    pub body: String,
    pub proof: PresenceProof,
    pub proof_lat: Option<f64>,
    pub proof_lng: Option<f64>,
    pub proof_distance_m: Option<f64>,
    pub weather: Option<String>,
    pub source_message_id: Option<Uuid>,
}

#[derive(Debug, Clone)]
pub struct NewCapsule {
    pub space_id: Uuid,
    pub author_id: Option<Uuid>,
    pub author_name: String,
    pub recipient_hint: String,
    pub body: String,
    pub passphrase_hash: String,
    pub radius_m: i32,
    pub opens_at: Option<time::OffsetDateTime>,
}

/// Everything the open endpoint needs to judge an attempt, fetched in one go.
#[derive(Debug, Clone)]
pub struct CapsuleChallenge {
    pub id: Uuid,
    pub space_id: Uuid,
    pub author_name: String,
    pub body: String,
    pub passphrase_hash: String,
    pub radius_m: i32,
    pub opens_at: Option<time::OffsetDateTime>,
    pub opened_at: Option<time::OffsetDateTime>,
    pub opened_by_name: Option<String>,
    pub failed_attempts: i32,
    pub created_at: time::OffsetDateTime,
    pub space_lat: f64,
    pub space_lng: f64,
}

fn row_to_trace(row: sqlx::postgres::PgRow, viewer: Option<Uuid>, is_admin: bool) -> Result<Trace, sqlx::Error> {
    let author_id: Option<Uuid> = row.try_get("author_id")?;
    let proof: String = row.try_get("proof")?;
    Ok(Trace {
        id: row.try_get("id")?,
        space_id: row.try_get("space_id")?,
        author_id,
        author_name: row.try_get("author_name")?,
        body: row.try_get("body")?,
        proof: PresenceProof::from_db(&proof),
        proof_distance_m: row.try_get("proof_distance_m")?,
        weather: row.try_get("weather")?,
        created_at: row.try_get("created_at")?,
        can_delete: is_admin || (viewer.is_some() && viewer == author_id),
    })
}

pub async fn list_traces(
    pool: &PgPool,
    space_id: Uuid,
    viewer: Option<Uuid>,
    is_admin: bool,
    limit: i64,
    offset: i64,
) -> Result<(Vec<Trace>, i64), sqlx::Error> {
    let total: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM space_traces WHERE space_id = $1 AND NOT hidden",
    )
    .bind(space_id)
    .fetch_one(pool)
    .await?;

    let rows = sqlx::query(
        r#"
        SELECT id, space_id, author_id, author_name, body, proof::text AS proof,
               proof_distance_m, weather, created_at
        FROM space_traces
        WHERE space_id = $1 AND NOT hidden
        ORDER BY created_at DESC, id
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(space_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let traces = rows
        .into_iter()
        .map(|row| row_to_trace(row, viewer, is_admin))
        .collect::<Result<Vec<_>, _>>()?;
    Ok((traces, total))
}

pub async fn create_trace(pool: &PgPool, input: NewTrace) -> Result<Trace, sqlx::Error> {
    let row = sqlx::query(
        r#"
        INSERT INTO space_traces
            (space_id, author_id, author_name, body, proof, proof_lat, proof_lng,
             proof_distance_m, weather, source_message_id)
        VALUES ($1, $2, $3, $4, $5::presence_proof, $6, $7, $8, $9, $10)
        RETURNING id, space_id, author_id, author_name, body, proof::text AS proof,
                  proof_distance_m, weather, created_at
        "#,
    )
    .bind(input.space_id)
    .bind(input.author_id)
    .bind(input.author_name)
    .bind(input.body)
    .bind(input.proof.as_db())
    .bind(input.proof_lat)
    .bind(input.proof_lng)
    .bind(input.proof_distance_m)
    .bind(input.weather)
    .bind(input.source_message_id)
    .fetch_one(pool)
    .await?;

    row_to_trace(row, input.author_id, false)
}

/// Soft delete. A trace is part of a place's record, so it is hidden rather
/// than erased; the chronicle counts stay honest about what was written.
pub async fn hide_trace(
    pool: &PgPool,
    trace_id: Uuid,
    actor: Uuid,
    is_admin: bool,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE space_traces
        SET hidden = true
        WHERE id = $1
          AND NOT hidden
          AND ($3 OR author_id = $2)
        "#,
    )
    .bind(trace_id)
    .bind(actor)
    .bind(is_admin)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn chronicle(pool: &PgPool, space_id: Uuid) -> Result<SpaceChronicle, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT
          (SELECT count(*)::bigint FROM space_traces WHERE space_id = $1 AND NOT hidden) AS trace_count,
          (SELECT count(*)::bigint FROM space_traces
             WHERE space_id = $1 AND NOT hidden AND proof <> 'remote') AS on_site_count,
          (SELECT count(*)::bigint FROM space_capsules WHERE space_id = $1) AS capsule_count,
          (SELECT count(*)::bigint FROM space_capsules
             WHERE space_id = $1 AND opened_at IS NOT NULL) AS capsule_opened_count,
          (SELECT min(created_at) FROM space_traces WHERE space_id = $1 AND NOT hidden) AS first_trace_at,
          (SELECT max(created_at) FROM space_traces WHERE space_id = $1 AND NOT hidden) AS latest_trace_at,
          (SELECT author_name FROM space_traces
             WHERE space_id = $1 AND NOT hidden
             ORDER BY created_at ASC, id LIMIT 1) AS first_trace_author
        "#,
    )
    .bind(space_id)
    .fetch_one(pool)
    .await?;

    Ok(SpaceChronicle {
        trace_count: row.try_get("trace_count")?,
        on_site_count: row.try_get("on_site_count")?,
        capsule_count: row.try_get("capsule_count")?,
        capsule_opened_count: row.try_get("capsule_opened_count")?,
        first_trace_at: row.try_get("first_trace_at")?,
        first_trace_author: row.try_get("first_trace_author")?,
        latest_trace_at: row.try_get("latest_trace_at")?,
    })
}

pub async fn list_capsules(
    pool: &PgPool,
    space_id: Uuid,
    viewer: Option<Uuid>,
) -> Result<Vec<CapsuleSummary>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT id, space_id, author_id, author_name, recipient_hint, radius_m,
               opens_at, opened_at, opened_by_name, created_at
        FROM space_capsules
        WHERE space_id = $1
        ORDER BY opened_at IS NOT NULL, created_at DESC
        "#,
    )
    .bind(space_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            let author_id: Option<Uuid> = row.try_get("author_id")?;
            Ok(CapsuleSummary {
                id: row.try_get("id")?,
                space_id: row.try_get("space_id")?,
                author_name: row.try_get("author_name")?,
                recipient_hint: row.try_get("recipient_hint")?,
                radius_m: row.try_get("radius_m")?,
                opens_at: row.try_get("opens_at")?,
                opened_at: row.try_get("opened_at")?,
                opened_by_name: row.try_get("opened_by_name")?,
                created_at: row.try_get("created_at")?,
                is_mine: viewer.is_some() && viewer == author_id,
            })
        })
        .collect()
}

pub async fn create_capsule(pool: &PgPool, input: NewCapsule) -> Result<Uuid, sqlx::Error> {
    let id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO space_capsules
            (space_id, author_id, author_name, recipient_hint, body, passphrase_hash,
             radius_m, opens_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id
        "#,
    )
    .bind(input.space_id)
    .bind(input.author_id)
    .bind(input.author_name)
    .bind(input.recipient_hint)
    .bind(input.body)
    .bind(input.passphrase_hash)
    .bind(input.radius_m)
    .bind(input.opens_at)
    .fetch_one(pool)
    .await?;

    Ok(id)
}

pub async fn capsule_challenge(
    pool: &PgPool,
    capsule_id: Uuid,
) -> Result<Option<CapsuleChallenge>, sqlx::Error> {
    let Some(row) = sqlx::query(
        r#"
        SELECT c.id, c.space_id, c.author_name, c.body, c.passphrase_hash, c.radius_m, c.opens_at,
               c.opened_at, c.opened_by_name, c.failed_attempts, c.created_at,
               s.lat AS space_lat, s.lng AS space_lng
        FROM space_capsules c
        JOIN spaces s ON s.id = c.space_id
        WHERE c.id = $1
        "#,
    )
    .bind(capsule_id)
    .fetch_optional(pool)
    .await?
    else {
        return Ok(None);
    };

    Ok(Some(CapsuleChallenge {
        id: row.try_get("id")?,
        space_id: row.try_get("space_id")?,
        author_name: row.try_get("author_name")?,
        body: row.try_get("body")?,
        passphrase_hash: row.try_get("passphrase_hash")?,
        radius_m: row.try_get("radius_m")?,
        opens_at: row.try_get("opens_at")?,
        opened_at: row.try_get("opened_at")?,
        opened_by_name: row.try_get("opened_by_name")?,
        failed_attempts: row.try_get("failed_attempts")?,
        created_at: row.try_get("created_at")?,
        space_lat: row.try_get("space_lat")?,
        space_lng: row.try_get("space_lng")?,
    }))
}

pub async fn record_failed_attempt(pool: &PgPool, capsule_id: Uuid) -> Result<i32, sqlx::Error> {
    let attempts: i32 = sqlx::query_scalar(
        r#"
        UPDATE space_capsules
        SET failed_attempts = failed_attempts + 1
        WHERE id = $1
        RETURNING failed_attempts
        "#,
    )
    .bind(capsule_id)
    .fetch_one(pool)
    .await?;
    Ok(attempts)
}

/// Marks the capsule open. The `opened_at IS NULL` guard makes two people
/// racing at the same spot resolve to exactly one opener.
pub async fn mark_capsule_opened(
    pool: &PgPool,
    capsule_id: Uuid,
    opened_by: Option<Uuid>,
    opened_by_name: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE space_capsules
        SET opened_at = now(), opened_by = $2, opened_by_name = $3
        WHERE id = $1 AND opened_at IS NULL
        "#,
    )
    .bind(capsule_id)
    .bind(opened_by)
    .bind(opened_by_name)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub fn is_locked(failed_attempts: i32) -> bool {
    failed_attempts >= CAPSULE_MAX_ATTEMPTS
}

/// Space coordinates, for judging whether a writer is standing at the place.
pub async fn space_coordinates(
    pool: &PgPool,
    space_id: Uuid,
) -> Result<Option<(f64, f64)>, sqlx::Error> {
    let row = sqlx::query("SELECT lat, lng FROM spaces WHERE id = $1")
        .bind(space_id)
        .fetch_optional(pool)
        .await?;

    match row {
        Some(row) => Ok(Some((row.try_get("lat")?, row.try_get("lng")?))),
        None => Ok(None),
    }
}

/// Whether a Space has a Discord community configured, which is what the
/// `discord` presence proof leans on.
pub async fn space_discord_group(
    pool: &PgPool,
    space_id: Uuid,
) -> Result<Option<String>, sqlx::Error> {
    let value: Option<Option<String>> =
        sqlx::query_scalar("SELECT discord_group FROM spaces WHERE id = $1")
            .bind(space_id)
            .fetch_optional(pool)
            .await?;
    Ok(value.flatten().filter(|v| !v.trim().is_empty()))
}
