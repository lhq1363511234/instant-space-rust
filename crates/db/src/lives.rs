//! Database access for digital lives: cloud homes, companions, trails,
//! distilled memorials (digital_lives) and visitor prayers.
//!
//! Copy conventions follow the life-distill skill (Song-style). Subject type
//! is neutral (pet today, human tomorrow).

use instant_domain::lives::{
    BiographyChapter, CloudHome, Companion, CompanionState, CompanionTrail, DigitalLife,
    LifeMapEntry, LifePrayer, PaginatedLives, PrayerKind,
};
use serde_json::Value;
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Valid Argon2 hash whose random preimage was discarded. Normal Space entry
/// must never unlock a cloud home; the home door uses cloud_homes.passphrase_hash.
const CLOUD_HOME_SPACE_PASSWORD_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$L/eaj4KH/9vcKXJqKl+nvg$wb5DLaTJ/X77Oqutf5gYTikcc8Zox+wCa0WW/JGHtHU";

// ---------------------------------------------------------------------------
// Cloud home
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct NewCompanion {
    pub owner_id: Uuid,
    pub home_id: Uuid,
    pub subject_type: String,
    pub name: String,
    pub species: Option<String>,
    pub breed: Option<String>,
    pub gender: Option<String>,
    pub birth_at: Option<time::Date>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NewTrail {
    pub companion_id: Uuid,
    pub owner_id: Uuid,
    pub space_id: Option<Uuid>,
    pub space_name: Option<String>,
    pub place_name: Option<String>,
    pub proof: String,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub snippet: Option<String>,
    pub season_hint: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NewPrayer {
    pub life_id: Uuid,
    pub visitor_id: Option<Uuid>,
    pub visitor_name: String,
    pub kind: PrayerKind,
    pub message: Option<String>,
}

fn row_to_home(row: &sqlx::postgres::PgRow) -> Result<CloudHome, sqlx::Error> {
    let has: Option<String> = row.try_get("passphrase_hash")?;
    Ok(CloudHome {
        id: row.try_get("id")?,
        space_id: row.try_get("space_id")?,
        owner_id: row.try_get("owner_id")?,
        owner_name: row.try_get("owner_name")?,
        name: row.try_get("name")?,
        motto: row.try_get("motto")?,
        has_passphrase: has.as_deref().map(|s| !s.is_empty()).unwrap_or(false),
        door_note: row.try_get("door_note")?,
        created_at: row.try_get("created_at")?,
    })
}

fn row_to_companion(row: &sqlx::postgres::PgRow) -> Result<Companion, sqlx::Error> {
    let state: String = row.try_get("state")?;
    Ok(Companion {
        id: row.try_get("id")?,
        owner_id: row.try_get("owner_id")?,
        home_id: row.try_get("home_id")?,
        subject_type: row.try_get("subject_type")?,
        name: row.try_get("name")?,
        species: row.try_get("species")?,
        breed: row.try_get("breed")?,
        gender: row.try_get("gender")?,
        birth_at: row.try_get("birth_at")?,
        death_at: row.try_get("death_at")?,
        state: CompanionState::from_db(&state),
        avatar_url: row.try_get("avatar_url")?,
        trail_count: row.try_get("trail_count").unwrap_or(0),
        created_at: row.try_get("created_at")?,
    })
}

fn row_to_digital_life(row: &sqlx::postgres::PgRow) -> Result<DigitalLife, sqlx::Error> {
    let biography: Value = row.try_get("biography")?;
    let life_map: Value = row.try_get("life_map")?;
    Ok(DigitalLife {
        id: row.try_get("id")?,
        companion_id: row.try_get("companion_id")?,
        owner_id: row.try_get("owner_id")?,
        subject_type: row.try_get("subject_type")?,
        name: row.try_get("name")?,
        epitaph: row.try_get("epitaph")?,
        biography: serde_json::from_value(biography).unwrap_or_default(),
        inscription: row.try_get("inscription")?,
        life_map: serde_json::from_value(life_map).unwrap_or_default(),
        memorial_date: row.try_get("memorial_date")?,
        incense_count: row.try_get("incense_count")?,
        visitor_count: row.try_get("visitor_count")?,
        distill_version: row.try_get("distill_version")?,
        content_version: row.try_get("content_version")?,
        published: row.try_get("published")?,
        created_at: row.try_get("created_at")?,
    })
}

/// Every user has one cloud home, and that home is itself a private Space.
/// Locking the user row makes first-touch creation idempotent under concurrent
/// requests and prevents orphan Space rows.
pub async fn ensure_cloud_home(pool: &PgPool, owner_id: Uuid) -> Result<CloudHome, sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query("SELECT id FROM users WHERE id = $1 FOR UPDATE")
        .bind(owner_id)
        .fetch_one(&mut *tx)
        .await?;

    if let Some(row) = sqlx::query(
        r#"
        SELECT h.id, h.space_id, h.owner_id, u.name AS owner_name, h.name, h.motto,
               h.passphrase_hash, h.door_note, h.created_at
        FROM cloud_homes h
        LEFT JOIN users u ON u.id = h.owner_id
        WHERE h.owner_id = $1
        "#,
    )
    .bind(owner_id)
    .fetch_optional(&mut *tx)
    .await?
    {
        tx.commit().await?;
        return row_to_home(&row);
    }

    let space_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO spaces (
            name_zh, name_en, space_type, custom_type, category,
            lat, lng, is_public, password_hash, duration_hours, expires_at,
            status, creator_id, host_user_id, description_zh, description_en,
            tag_zh, tag_en
        )
        VALUES (
            '云上家', 'Cloud Home', 'custom'::space_type, '家空间', 'cloud_home',
            0, 0, false, $2, 0, NULL,
            'active'::space_status, $1, $1,
            '这是用户在 inspace 的家。离线时，家人在此歇息；出门时，足迹随行。',
            'The user''s home in inspace. Companions rest here while away and travel with the user when present.',
            '云上家', 'Cloud Home'
        )
        RETURNING id
        "#,
    )
    .bind(owner_id)
    .bind(CLOUD_HOME_SPACE_PASSWORD_HASH)
    .fetch_one(&mut *tx)
    .await?;

    let row = sqlx::query(
        r#"
        INSERT INTO cloud_homes (owner_id, space_id)
        VALUES ($1, $2)
        RETURNING id, space_id, owner_id, NULL::text AS owner_name, name, motto,
                  passphrase_hash, door_note, created_at
        "#,
    )
    .bind(owner_id)
    .bind(space_id)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO space_members (space_id, user_id, role)
        VALUES ($1, $2, 'host')
        ON CONFLICT (space_id, user_id) DO UPDATE SET role = 'host'
        "#,
    )
    .bind(space_id)
    .bind(owner_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO space_host_tenures (space_id, user_id, role, status, started_at)
        VALUES ($1, $2, 'primary', 'active', now())
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(space_id)
    .bind(owner_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    row_to_home(&row)
}

pub async fn get_cloud_home_by_owner(
    pool: &PgPool,
    owner_id: Uuid,
) -> Result<Option<CloudHome>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT h.id, h.space_id, h.owner_id, u.name AS owner_name, h.name, h.motto,
               h.passphrase_hash, h.door_note, h.created_at
        FROM cloud_homes h
        LEFT JOIN users u ON u.id = h.owner_id
        WHERE h.owner_id = $1
        "#,
    )
    .bind(owner_id)
    .fetch_optional(pool)
    .await?;
    row.map(|r| row_to_home(&r)).transpose()
}

pub async fn get_cloud_home_by_id(
    pool: &PgPool,
    home_id: Uuid,
) -> Result<Option<CloudHome>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT h.id, h.space_id, h.owner_id, u.name AS owner_name, h.name, h.motto,
               h.passphrase_hash, h.door_note, h.created_at
        FROM cloud_homes h
        LEFT JOIN users u ON u.id = h.owner_id
        WHERE h.id = $1
        "#,
    )
    .bind(home_id)
    .fetch_optional(pool)
    .await?;
    row.map(|r| row_to_home(&r)).transpose()
}

/// Owner updates their home and its canonical Space in one transaction.
/// The home door key remains the only usable entry key for a home Space.
pub async fn update_cloud_home(
    pool: &PgPool,
    owner_id: Uuid,
    name: &str,
    motto: Option<&str>,
    door_note: Option<&str>,
    passphrase_hash: Option<&str>,
    clear_passphrase: bool,
) -> Result<CloudHome, sqlx::Error> {
    let hash = if clear_passphrase {
        None
    } else if passphrase_hash.is_some_and(|value| !value.is_empty()) {
        passphrase_hash
    } else {
        None
    };
    let mut tx = pool.begin().await?;
    let row = sqlx::query(
        r#"
        UPDATE cloud_homes
        SET name = COALESCE(NULLIF($2, ''), name),
            motto = COALESCE($3, motto),
            door_note = COALESCE($4, door_note),
            passphrase_hash = CASE
                WHEN $5::boolean THEN NULL
                WHEN $6::text IS NOT NULL THEN $6::text
                ELSE passphrase_hash
            END,
            updated_at = now()
        WHERE owner_id = $1
        RETURNING id, space_id, owner_id, NULL::text AS owner_name, name, motto,
                  passphrase_hash, door_note, created_at
        "#,
    )
    .bind(owner_id)
    .bind(name)
    .bind(motto)
    .bind(door_note)
    .bind(clear_passphrase)
    .bind(hash)
    .fetch_one(&mut *tx)
    .await?;

    let space_id: Uuid = row.try_get("space_id")?;
    let home_name: String = row.try_get("name")?;
    let current_door_hash: Option<String> = row.try_get("passphrase_hash")?;
    let space_hash = current_door_hash
        .as_deref()
        .unwrap_or(CLOUD_HOME_SPACE_PASSWORD_HASH);

    sqlx::query(
        r#"
        UPDATE spaces
        SET name_zh = $2,
            password_version = CASE WHEN password_hash <> $3 THEN password_version + 1 ELSE password_version END,
            password_hash = $3,
            updated_at = now()
        WHERE id = $1 AND category = 'cloud_home'
        "#,
    )
    .bind(space_id)
    .bind(home_name)
    .bind(space_hash)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    row_to_home(&row)
}

/// The passphrase hash alone, so the server can verify a visitor's key.
pub async fn home_passphrase_hash(
    pool: &PgPool,
    home_id: Uuid,
) -> Result<Option<String>, sqlx::Error> {
    let hash: Option<Option<String>> =
        sqlx::query_scalar("SELECT passphrase_hash FROM cloud_homes WHERE id = $1")
            .bind(home_id)
            .fetch_optional(pool)
            .await?;
    Ok(hash.flatten())
}

// ---------------------------------------------------------------------------
// Companions (family members)
// ---------------------------------------------------------------------------

pub async fn list_companions(pool: &PgPool, owner_id: Uuid) -> Result<Vec<Companion>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT c.*, COUNT(t.id)::bigint AS trail_count
        FROM companions c
        LEFT JOIN companion_trails t ON t.companion_id = c.id
        WHERE c.owner_id = $1
        GROUP BY c.id
        ORDER BY c.created_at ASC
        "#,
    )
    .bind(owner_id)
    .fetch_all(pool)
    .await?;
    rows.iter().map(row_to_companion).collect()
}

/// What a visitor sees inside a home: companions at home, and memorials.
pub async fn list_companions_by_home(
    pool: &PgPool,
    home_id: Uuid,
) -> Result<Vec<Companion>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT c.*, COUNT(t.id)::bigint AS trail_count
        FROM companions c
        LEFT JOIN companion_trails t ON t.companion_id = c.id
        WHERE c.home_id = $1
        GROUP BY c.id
        ORDER BY
            CASE c.state WHEN 'memorial' THEN 2 ELSE 1 END,
            c.created_at ASC
        "#,
    )
    .bind(home_id)
    .fetch_all(pool)
    .await?;
    rows.iter().map(row_to_companion).collect()
}

pub async fn get_companion(pool: &PgPool, id: Uuid) -> Result<Option<Companion>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT c.*, COUNT(t.id)::bigint AS trail_count
        FROM companions c
        LEFT JOIN companion_trails t ON t.companion_id = c.id
        WHERE c.id = $1
        GROUP BY c.id
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    row.map(|r| row_to_companion(&r)).transpose()
}

pub async fn create_companion(
    pool: &PgPool,
    input: NewCompanion,
) -> Result<Companion, sqlx::Error> {
    let row = sqlx::query(
        r#"
        INSERT INTO companions
            (owner_id, home_id, subject_type, name, species, breed, gender, birth_at, avatar_url)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id, owner_id, home_id, subject_type, name, species, breed, gender,
                  birth_at, death_at, state::text AS state, avatar_url, created_at
        "#,
    )
    .bind(input.owner_id)
    .bind(input.home_id)
    .bind(input.subject_type)
    .bind(input.name)
    .bind(input.species)
    .bind(input.breed)
    .bind(input.gender)
    .bind(input.birth_at)
    .bind(input.avatar_url)
    .fetch_one(pool)
    .await?;
    // trail_count not selected; attach 0 for the fresh row.
    let mut c = row_to_companion(&row)?;
    c.trail_count = 0;
    Ok(c)
}

pub async fn update_companion(
    pool: &PgPool,
    id: Uuid,
    owner_id: Uuid,
    name: Option<&str>,
    species: Option<&str>,
    breed: Option<&str>,
    gender: Option<&str>,
    birth_at: Option<time::Date>,
    avatar_url: Option<&str>,
) -> Result<Option<Companion>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        UPDATE companions
        SET name = COALESCE(NULLIF($3, ''), name),
            species = COALESCE(NULLIF($4, ''), species),
            breed = COALESCE(NULLIF($5, ''), breed),
            gender = COALESCE(NULLIF($6, ''), gender),
            birth_at = COALESCE($7, birth_at),
            avatar_url = COALESCE(NULLIF($8, ''), avatar_url),
            updated_at = now()
        WHERE id = $1 AND owner_id = $2
        RETURNING id, owner_id, home_id, subject_type, name, species, breed, gender,
                  birth_at, death_at, state::text AS state, avatar_url, created_at
        "#,
    )
    .bind(id)
    .bind(owner_id)
    .bind(name.unwrap_or(""))
    .bind(species.unwrap_or(""))
    .bind(breed.unwrap_or(""))
    .bind(gender.unwrap_or(""))
    .bind(birth_at)
    .bind(avatar_url.unwrap_or(""))
    .fetch_optional(pool)
    .await?;
    row.map(|r| row_to_companion(&r)).transpose()
}

pub async fn set_companion_state(
    pool: &PgPool,
    id: Uuid,
    owner_id: Uuid,
    state: CompanionState,
) -> Result<Option<Companion>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        UPDATE companions
        SET state = $3::text, updated_at = now()
        WHERE id = $1 AND owner_id = $2
        RETURNING id, owner_id, home_id, subject_type, name, species, breed, gender,
                  birth_at, death_at, state::text AS state, avatar_url, created_at
        "#,
    )
    .bind(id)
    .bind(owner_id)
    .bind(state.as_db())
    .fetch_optional(pool)
    .await?;
    row.map(|r| row_to_companion(&r)).transpose()
}

/// A companion crossed over: record the date and move to 追远.
pub async fn mark_memorial(
    pool: &PgPool,
    id: Uuid,
    owner_id: Uuid,
    death_at: time::Date,
) -> Result<Option<Companion>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        UPDATE companions
        SET state = 'memorial', death_at = COALESCE($3, death_at), updated_at = now()
        WHERE id = $1 AND owner_id = $2
        RETURNING id, owner_id, home_id, subject_type, name, species, breed, gender,
                  birth_at, death_at, state::text AS state, avatar_url, created_at
        "#,
    )
    .bind(id)
    .bind(owner_id)
    .bind(death_at)
    .fetch_optional(pool)
    .await?;
    row.map(|r| row_to_companion(&r)).transpose()
}

// ---------------------------------------------------------------------------
// Trails (footprints)
// ---------------------------------------------------------------------------

pub async fn record_trail(pool: &PgPool, input: NewTrail) -> Result<CompanionTrail, sqlx::Error> {
    let row = sqlx::query(
        r#"
        INSERT INTO companion_trails
            (companion_id, owner_id, space_id, space_name, place_name, proof,
             lat, lng, snippet, season_hint)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id, companion_id, space_id, space_name, place_name, proof,
                  noted_at, snippet, season_hint
        "#,
    )
    .bind(input.companion_id)
    .bind(input.owner_id)
    .bind(input.space_id)
    .bind(input.space_name)
    .bind(input.place_name)
    .bind(input.proof)
    .bind(input.lat)
    .bind(input.lng)
    .bind(input.snippet)
    .bind(input.season_hint)
    .fetch_one(pool)
    .await?;
    Ok(CompanionTrail {
        id: row.try_get("id")?,
        companion_id: row.try_get("companion_id")?,
        space_id: row.try_get("space_id")?,
        space_name: row.try_get("space_name")?,
        place_name: row.try_get("place_name")?,
        proof: row.try_get("proof")?,
        noted_at: row.try_get("noted_at")?,
        snippet: row.try_get("snippet")?,
        season_hint: row.try_get("season_hint")?,
    })
}

pub async fn list_trails(
    pool: &PgPool,
    companion_id: Uuid,
    limit: i64,
) -> Result<Vec<CompanionTrail>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT id, companion_id, space_id, space_name, place_name, proof,
               noted_at, snippet, season_hint
        FROM companion_trails
        WHERE companion_id = $1
        ORDER BY noted_at DESC
        LIMIT $2
        "#,
    )
    .bind(companion_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    rows.iter()
        .map(|row| {
            Ok(CompanionTrail {
                id: row.try_get("id")?,
                companion_id: row.try_get("companion_id")?,
                space_id: row.try_get("space_id")?,
                space_name: row.try_get("space_name")?,
                place_name: row.try_get("place_name")?,
                proof: row.try_get("proof")?,
                noted_at: row.try_get("noted_at")?,
                snippet: row.try_get("snippet")?,
                season_hint: row.try_get("season_hint")?,
            })
        })
        .collect()
}

/// When the owner proves presence at a space, every living companion records
/// "it was here too" automatically — one row per companion. Only fires while
/// the owner has been active in the last 24h (the online/offline signal).
pub async fn record_trails_for_owner_at_space(
    pool: &PgPool,
    owner_id: Uuid,
    space_id: Uuid,
    proof: &str,
    lat: Option<f64>,
    lng: Option<f64>,
) -> Result<usize, sqlx::Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO companion_trails
            (companion_id, owner_id, space_id, space_name, place_name, proof, lat, lng)
        SELECT c.id, c.owner_id, $2, s.name, s.name, $3, $4, $5
        FROM companions c
        LEFT JOIN spaces s ON s.id = $2
        WHERE c.owner_id = $1
          AND c.state <> 'memorial'
          AND EXISTS (
            SELECT 1 FROM users u
            WHERE u.id = $1 AND u.last_active_at > now() - interval '24 hours'
          )
        "#,
    )
    .bind(owner_id)
    .bind(space_id)
    .bind(proof)
    .bind(lat)
    .bind(lng)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() as usize)
}

// ---------------------------------------------------------------------------
// Digital lives (memorials)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct NewDigitalLife {
    pub companion_id: Uuid,
    pub owner_id: Uuid,
    pub subject_type: String,
    pub name: String,
    pub epitaph: String,
    pub biography: Vec<BiographyChapter>,
    pub inscription: String,
    pub life_map: Vec<LifeMapEntry>,
    pub memorial_date: Option<time::Date>,
    pub distill_version: i32,
}

pub async fn create_digital_life(
    pool: &PgPool,
    input: NewDigitalLife,
) -> Result<DigitalLife, sqlx::Error> {
    let biography = serde_json::to_value(&input.biography).unwrap_or(Value::Array(Vec::new()));
    let life_map = serde_json::to_value(&input.life_map).unwrap_or(Value::Array(Vec::new()));
    let row = sqlx::query(
        r#"
        INSERT INTO digital_lives
            (companion_id, owner_id, subject_type, name, epitaph, biography, inscription,
             life_map, memorial_date, distill_version)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id, companion_id, owner_id, subject_type, name, epitaph, biography,
                  inscription, life_map, memorial_date, incense_count, visitor_count,
                  distill_version, content_version, published, created_at
        "#,
    )
    .bind(input.companion_id)
    .bind(input.owner_id)
    .bind(input.subject_type)
    .bind(input.name)
    .bind(input.epitaph)
    .bind(biography)
    .bind(input.inscription)
    .bind(life_map)
    .bind(input.memorial_date)
    .bind(input.distill_version)
    .fetch_one(pool)
    .await?;
    row_to_digital_life(&row)
}

pub async fn get_digital_life(pool: &PgPool, id: Uuid) -> Result<Option<DigitalLife>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT id, companion_id, owner_id, subject_type, name, epitaph, biography,
               inscription, life_map, memorial_date, incense_count, visitor_count,
               distill_version, content_version, published, created_at
        FROM digital_lives
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    row.map(|r| row_to_digital_life(&r)).transpose()
}

pub async fn get_digital_life_by_companion(
    pool: &PgPool,
    companion_id: Uuid,
) -> Result<Option<DigitalLife>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT id, companion_id, owner_id, subject_type, name, epitaph, biography,
               inscription, life_map, memorial_date, incense_count, visitor_count,
               distill_version, content_version, published, created_at
        FROM digital_lives
        WHERE companion_id = $1
        "#,
    )
    .bind(companion_id)
    .fetch_optional(pool)
    .await?;
    row.map(|r| row_to_digital_life(&r)).transpose()
}

/// Public directory of memorials (追远), newest first, published only.
pub async fn list_digital_lives(
    pool: &PgPool,
    limit: i64,
    offset: i64,
) -> Result<PaginatedLives, sqlx::Error> {
    let total: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM digital_lives WHERE published = true")
            .fetch_one(pool)
            .await?;
    let rows = sqlx::query(
        r#"
        SELECT id, companion_id, owner_id, subject_type, name, epitaph, biography,
               inscription, life_map, memorial_date, incense_count, visitor_count,
               distill_version, content_version, published, created_at
        FROM digital_lives
        WHERE published = true
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    let items = rows
        .iter()
        .map(row_to_digital_life)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(PaginatedLives {
        items,
        total,
        limit,
        offset,
    })
}

/// Owner edits the distilled content. content_version bumps on every save so
/// a rewrite can always be traced back.
pub async fn update_digital_life_content(
    pool: &PgPool,
    id: Uuid,
    owner_id: Uuid,
    epitaph: &str,
    biography: Vec<BiographyChapter>,
    inscription: &str,
    life_map: Vec<LifeMapEntry>,
) -> Result<Option<DigitalLife>, sqlx::Error> {
    let biography = serde_json::to_value(&biography).unwrap_or(Value::Array(Vec::new()));
    let life_map = serde_json::to_value(&life_map).unwrap_or(Value::Array(Vec::new()));
    let row = sqlx::query(
        r#"
        UPDATE digital_lives
        SET epitaph = $3, biography = $4, inscription = $5, life_map = $6,
            content_version = content_version + 1, updated_at = now()
        WHERE id = $1 AND owner_id = $2
        RETURNING id, companion_id, owner_id, subject_type, name, epitaph, biography,
                  inscription, life_map, memorial_date, incense_count, visitor_count,
                  distill_version, content_version, published, created_at
        "#,
    )
    .bind(id)
    .bind(owner_id)
    .bind(epitaph)
    .bind(biography)
    .bind(inscription)
    .bind(life_map)
    .fetch_optional(pool)
    .await?;
    row.map(|r| row_to_digital_life(&r)).transpose()
}

/// Bump visitor count once per viewing session (idempotent enough in v1).
pub async fn bump_visitor(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE digital_lives SET visitor_count = visitor_count + 1 WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Prayers (香 / 花 / 灯 / 留字)
// ---------------------------------------------------------------------------

pub async fn add_prayer(pool: &PgPool, input: NewPrayer) -> Result<LifePrayer, sqlx::Error> {
    let kind = input.kind.as_db();
    let row = sqlx::query(
        r#"
        INSERT INTO life_prayers (life_id, visitor_id, visitor_name, kind, message)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, life_id, visitor_name, kind::text AS kind, message, created_at
        "#,
    )
    .bind(input.life_id)
    .bind(input.visitor_id)
    .bind(input.visitor_name)
    .bind(kind)
    .bind(input.message)
    .fetch_one(pool)
    .await?;
    // A stick of incense adds to the eternal flame; flowers and words do not.
    sqlx::query("UPDATE digital_lives SET incense_count = incense_count + 1 WHERE id = $1 AND $2 = 'incense'")
        .bind(input.life_id)
        .bind(kind)
        .execute(pool)
        .await?;
    Ok(LifePrayer {
        id: row.try_get("id")?,
        life_id: row.try_get("life_id")?,
        visitor_name: row.try_get("visitor_name")?,
        kind: PrayerKind::from_db(&row.try_get::<String, _>("kind")?),
        message: row.try_get("message")?,
        created_at: row.try_get("created_at")?,
    })
}

pub async fn list_prayers(
    pool: &PgPool,
    life_id: Uuid,
    limit: i64,
) -> Result<Vec<LifePrayer>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT id, life_id, visitor_name, kind::text AS kind, message, created_at
        FROM life_prayers
        WHERE life_id = $1
        ORDER BY created_at DESC
        LIMIT $2
        "#,
    )
    .bind(life_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    rows.iter()
        .map(|row| {
            Ok(LifePrayer {
                id: row.try_get("id")?,
                life_id: row.try_get("life_id")?,
                visitor_name: row.try_get("visitor_name")?,
                kind: PrayerKind::from_db(&row.try_get::<String, _>("kind")?),
                message: row.try_get("message")?,
                created_at: row.try_get("created_at")?,
            })
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Activity signal: 24h of activity counts as online (在侧).
// ---------------------------------------------------------------------------

pub async fn touch_last_active(pool: &PgPool, user_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE users SET last_active_at = now() WHERE id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn last_active_at(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<time::OffsetDateTime>, sqlx::Error> {
    sqlx::query_scalar("SELECT last_active_at FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
}
