use instant_domain::world::{
    EnterSpaceOutcome, EntryMethod, HostGovernanceState, HostTenureRole, HostTenureStatus,
    PresenceSubjectKind, Scene, SceneBundle, SceneKind, SceneObject, SceneObjectKind,
    SceneSpawnPoint, SceneStatus, SpaceGovernanceEvent, SpaceGovernanceSnapshot, SpaceHostIdentity,
    SpaceRelation, SpaceRole, WorldPresence,
};
use serde_json::Value;
use sqlx::{PgPool, Postgres, Row, Transaction};
use uuid::Uuid;

fn scene_from_row(row: &sqlx::postgres::PgRow) -> Result<Scene, sqlx::Error> {
    Ok(Scene {
        id: row.try_get("id")?,
        space_id: row.try_get("space_id")?,
        slug: row.try_get("slug")?,
        kind: SceneKind::from_db(&row.try_get::<String, _>("kind")?),
        name_zh: row.try_get("name_zh")?,
        name_en: row.try_get("name_en")?,
        description_zh: row.try_get("description_zh")?,
        description_en: row.try_get("description_en")?,
        layout: row.try_get("layout")?,
        is_default: row.try_get("is_default")?,
        status: SceneStatus::from_db(&row.try_get::<String, _>("status")?),
        version: row.try_get("version")?,
        created_by: row.try_get("created_by")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn object_from_row(row: &sqlx::postgres::PgRow) -> Result<SceneObject, sqlx::Error> {
    Ok(SceneObject {
        id: row.try_get("id")?,
        scene_id: row.try_get("scene_id")?,
        kind: SceneObjectKind::from_db(&row.try_get::<String, _>("object_kind")?),
        name_zh: row.try_get("name_zh")?,
        name_en: row.try_get("name_en")?,
        x: row.try_get("x")?,
        y: row.try_get("y")?,
        width: row.try_get("width")?,
        height: row.try_get("height")?,
        z_index: row.try_get("z_index")?,
        interaction_radius: row.try_get("interaction_radius")?,
        content_kind: row.try_get("content_kind")?,
        content_id: row.try_get("content_id")?,
        target_space_id: row.try_get("target_space_id")?,
        target_scene_id: row.try_get("target_scene_id")?,
        target_spawn_key: row.try_get("target_spawn_key")?,
        config: row.try_get("config")?,
        status: SceneStatus::from_db(&row.try_get::<String, _>("status")?),
    })
}

fn spawn_from_row(row: &sqlx::postgres::PgRow) -> Result<SceneSpawnPoint, sqlx::Error> {
    Ok(SceneSpawnPoint {
        id: row.try_get("id")?,
        scene_id: row.try_get("scene_id")?,
        key: row.try_get("key")?,
        name_zh: row.try_get("name_zh")?,
        name_en: row.try_get("name_en")?,
        x: row.try_get("x")?,
        y: row.try_get("y")?,
        facing: row.try_get("facing")?,
        is_default: row.try_get("is_default")?,
    })
}

fn presence_from_row(row: &sqlx::postgres::PgRow) -> Result<WorldPresence, sqlx::Error> {
    Ok(WorldPresence {
        id: row.try_get("id")?,
        subject_kind: PresenceSubjectKind::from_db(&row.try_get::<String, _>("subject_kind")?),
        subject_id: row.try_get("subject_id")?,
        owner_user_id: row.try_get("owner_user_id")?,
        space_id: row.try_get("space_id")?,
        scene_id: row.try_get("scene_id")?,
        spawn_point_id: row.try_get("spawn_point_id")?,
        x: row.try_get("x")?,
        y: row.try_get("y")?,
        entry_method: EntryMethod::from_db(&row.try_get::<String, _>("entry_method")?),
        entered_at: row.try_get("entered_at")?,
        last_seen_at: row.try_get("last_seen_at")?,
    })
}

fn role_to_scene(role: SpaceRole) -> SceneKind {
    match role {
        SpaceRole::Hub => SceneKind::Hub,
        SpaceRole::Home => SceneKind::Home,
        SpaceRole::Memorial => SceneKind::Memorial,
        SpaceRole::Place | SpaceRole::Micro => SceneKind::Place,
    }
}

/// Lazily establish the first published Scene for a Space. Existing Spaces do
/// not need a bulk rewrite; the world layer grows only where it is used.
pub async fn ensure_default_scene(
    pool: &PgPool,
    space_id: Uuid,
    created_by: Option<Uuid>,
) -> Result<SceneBundle, sqlx::Error> {
    if let Some(bundle) = get_scene_bundle(pool, space_id, None).await? {
        let mut tx = pool.begin().await?;
        seed_default_objects(
            &mut tx,
            space_id,
            bundle.scene.id,
            bundle.scene.kind,
            created_by,
        )
        .await?;
        bind_scene_content(&mut tx, space_id, bundle.scene.id).await?;
        seed_relation_portals(&mut tx, space_id, bundle.scene.id, created_by).await?;
        tx.commit().await?;
        return get_scene_bundle(pool, space_id, Some(bundle.scene.id))
            .await?
            .ok_or(sqlx::Error::RowNotFound);
    }

    let mut tx = pool.begin().await?;
    let space = sqlx::query(
        r#"
        SELECT id, name_zh, name_en, world_role, is_public
        FROM spaces
        WHERE id = $1 AND status IN ('active', 'expired')
        FOR UPDATE
        "#,
    )
    .bind(space_id)
    .fetch_one(&mut *tx)
    .await?;

    let role = SpaceRole::from_db(&space.try_get::<String, _>("world_role")?);
    let scene_kind = role_to_scene(role);
    let scene_name_zh: String = space.try_get("name_zh")?;
    let scene_name_en: Option<String> = space.try_get("name_en")?;
    let slug = match scene_kind {
        SceneKind::Home => "courtyard",
        SceneKind::Hub => "hall",
        SceneKind::Memorial => "memorial-garden",
        SceneKind::Place | SceneKind::Interior => "main",
    };
    let layout = match scene_kind {
        SceneKind::Home => serde_json::json!({"theme":"song_courtyard","width":100,"height":100}),
        SceneKind::Hub => serde_json::json!({"theme":"song_city_hall","width":100,"height":100}),
        SceneKind::Memorial => {
            serde_json::json!({"theme":"song_memorial_garden","width":100,"height":100})
        }
        SceneKind::Place | SceneKind::Interior => {
            serde_json::json!({"theme":"song_place","width":100,"height":100})
        }
    };

    let scene_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO scenes (
          space_id, slug, kind, name_zh, name_en, layout, is_default,
          status, version, created_by, published_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, true, 'published', 1, $7, now())
        ON CONFLICT (space_id, slug) DO UPDATE
        SET is_default = true,
            status = CASE WHEN scenes.status = 'archived' THEN 'published' ELSE scenes.status END,
            updated_at = now()
        RETURNING id
        "#,
    )
    .bind(space_id)
    .bind(slug)
    .bind(scene_kind.as_db())
    .bind(&scene_name_zh)
    .bind(&scene_name_en)
    .bind(layout)
    .bind(created_by)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO scene_spawn_points (
          scene_id, key, name_zh, name_en, x, y, facing, is_default
        )
        VALUES ($1, 'entrance', '入口', 'Entrance', 50, 84, 'north', true)
        ON CONFLICT (scene_id, key) DO NOTHING
        "#,
    )
    .bind(scene_id)
    .execute(&mut *tx)
    .await?;

    seed_default_objects(&mut tx, space_id, scene_id, scene_kind, created_by).await?;
    bind_scene_content(&mut tx, space_id, scene_id).await?;
    seed_relation_portals(&mut tx, space_id, scene_id, created_by).await?;
    tx.commit().await?;

    get_scene_bundle(pool, space_id, Some(scene_id))
        .await?
        .ok_or(sqlx::Error::RowNotFound)
}

async fn seed_default_objects(
    tx: &mut Transaction<'_, Postgres>,
    space_id: Uuid,
    scene_id: Uuid,
    kind: SceneKind,
    created_by: Option<Uuid>,
) -> Result<(), sqlx::Error> {
    let objects: Vec<(&str, &str, &str, f64, f64, f64, f64, i32, Value)> = match kind {
        SceneKind::Hub => vec![
            (
                "tourist_center",
                "游客中心",
                "Visitor Center",
                50.0,
                24.0,
                24.0,
                22.0,
                2,
                serde_json::json!({"action":"guide","copy_zh":"在这里读这座城的空间志，也领取通往真实地点的线索。","copy_en":"Read the city journal and collect routes to real places."}),
            ),
            (
                "ai_guide",
                "问路人",
                "Local Guide",
                28.0,
                52.0,
                10.0,
                16.0,
                3,
                serde_json::json!({"action":"ai","copy_zh":"它先从现有空间志里替你找一条路；更完整的 AI 导游能力仍在接入。","copy_en":"It starts with the Space journals already here while the full AI guide is being connected."}),
            ),
            (
                "message_wall",
                "城中旧闻",
                "City Stories",
                70.0,
                52.0,
                15.0,
                17.0,
                3,
                serde_json::json!({"action":"stories","copy_zh":"来者留下的故事，在这里慢慢聚成一座城。","copy_en":"Visitor stories slowly gather into a city."}),
            ),
            (
                "notice_board",
                "今日城事",
                "Today in the City",
                50.0,
                66.0,
                15.0,
                12.0,
                2,
                serde_json::json!({"action":"notice","copy_zh":"活动、提醒与此刻在场的话，贴在这里。","copy_en":"Events, notices and live conversation are posted here."}),
            ),
        ],
        SceneKind::Home => vec![
            (
                "building",
                "家屋",
                "Home",
                50.0,
                24.0,
                28.0,
                24.0,
                2,
                serde_json::json!({"action":"home","copy_zh":"屋中收着一家人的日常。","copy_en":"The household keeps its everyday life here."}),
            ),
            (
                "message_wall",
                "家书墙",
                "Family Wall",
                24.0,
                50.0,
                14.0,
                18.0,
                3,
                serde_json::json!({"action":"stories","copy_zh":"来者可在此读家书、看旧日片段。","copy_en":"Read family notes and remembered moments."}),
            ),
            (
                "display",
                "足迹册",
                "Trail Album",
                76.0,
                50.0,
                15.0,
                16.0,
                3,
                serde_json::json!({"action":"trails","copy_zh":"主人去过的地方，也记着同行的家人。","copy_en":"Places visited by the owner remember their companions."}),
            ),
        ],
        SceneKind::Memorial => vec![
            (
                "display",
                "小传",
                "Life Record",
                50.0,
                28.0,
                20.0,
                20.0,
                2,
                serde_json::json!({"action":"biography","copy_zh":"一生不必很长，也自有可记之处。","copy_en":"Every life leaves something worth remembering."}),
            ),
            (
                "message_wall",
                "追思壁",
                "Remembrance Wall",
                30.0,
                55.0,
                15.0,
                18.0,
                3,
                serde_json::json!({"action":"stories","copy_zh":"香、花、灯与旧日话语，皆留在这里。","copy_en":"Incense, flowers, light and remembered words remain here."}),
            ),
        ],
        SceneKind::Place | SceneKind::Interior => vec![
            (
                "tourist_center",
                "空间志",
                "Space Journal",
                50.0,
                24.0,
                22.0,
                21.0,
                2,
                serde_json::json!({"action":"guide","copy_zh":"先在这里认识这个地方；不同类型的空间，会留下不同的使用方法与来历。","copy_en":"Begin here: every kind of Space keeps its own ways, history and practical knowledge."}),
            ),
            (
                "host",
                "主理人",
                "Host",
                25.0,
                51.0,
                11.0,
                17.0,
                3,
                serde_json::json!({"action":"host","copy_zh":"听主理人说，这里为何值得长久照料。","copy_en":"Hear why this place deserves long-term care."}),
            ),
            (
                "message_wall",
                "留言墙",
                "Message Wall",
                72.0,
                50.0,
                15.0,
                18.0,
                3,
                serde_json::json!({"action":"stories","copy_zh":"来过的人，把一段话留在这里。","copy_en":"People who came leave a few words here."}),
            ),
            (
                "notice_board",
                "今日所见",
                "Today Here",
                47.0,
                65.0,
                15.0,
                12.0,
                2,
                serde_json::json!({"action":"notice","copy_zh":"活动、提醒与现场变化，都写在这里。","copy_en":"Events, notices and changes are posted here."}),
            ),
            (
                "capsule",
                "埋信处",
                "Capsule Grove",
                82.0,
                68.0,
                12.0,
                14.0,
                3,
                serde_json::json!({"action":"capsule","copy_zh":"把一封信留给后来抵达的人；现场与双重口令仍由原有门禁守护。","copy_en":"Leave a letter for someone who arrives later; the existing on-site and two-key gate still protects it."}),
            ),
        ],
    };

    for (object_kind, name_zh, name_en, x, y, width, height, z_index, config) in objects {
        let action = config
            .get("action")
            .and_then(Value::as_str)
            .unwrap_or("detail")
            .to_string();
        sqlx::query(
            r#"
            INSERT INTO scene_objects (
              scene_id, object_kind, name_zh, name_en, x, y, width, height,
              z_index, config, status, created_by
            )
            SELECT $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 'published', $11
            WHERE NOT EXISTS (
              SELECT 1 FROM scene_objects
              WHERE scene_id = $1
                AND status <> 'archived'
                AND object_kind = $2
                AND COALESCE(config->>'action', 'detail') = $12
            )
            "#,
        )
        .bind(scene_id)
        .bind(object_kind)
        .bind(name_zh)
        .bind(name_en)
        .bind(x)
        .bind(y)
        .bind(width)
        .bind(height)
        .bind(z_index)
        .bind(config)
        .bind(created_by)
        .bind(action)
        .execute(&mut **tx)
        .await?;
    }

    // Keep the parameter explicit: object content is always bound in the
    // context of its durable Space, never inferred from coordinates.
    let _ = space_id;
    Ok(())
}

async fn bind_scene_content(
    tx: &mut Transaction<'_, Postgres>,
    space_id: Uuid,
    scene_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE scene_objects object
        SET content_kind = CASE object.object_kind
              WHEN 'tourist_center' THEN 'guides'
              WHEN 'ai_guide' THEN 'guide_recommendation'
              WHEN 'message_wall' THEN 'stories'
              WHEN 'notice_board' THEN 'discussion'
              WHEN 'host' THEN 'host'
              WHEN 'capsule' THEN 'capsules'
              ELSE object.content_kind
            END,
            content_id = CASE object.object_kind
              WHEN 'tourist_center' THEN (
                SELECT guide.id FROM guides guide
                WHERE guide.space_id = $1 AND guide.status = 'published'
                ORDER BY guide.featured DESC, guide.updated_at DESC LIMIT 1
              )
              WHEN 'ai_guide' THEN (
                SELECT guide.id FROM guides guide
                WHERE guide.space_id = $1 AND guide.status = 'published'
                ORDER BY guide.featured DESC, guide.updated_at DESC LIMIT 1
              )
              WHEN 'message_wall' THEN (
                SELECT trace.id FROM space_traces trace
                WHERE trace.space_id = $1 AND NOT trace.hidden
                ORDER BY trace.created_at DESC LIMIT 1
              )
              WHEN 'host' THEN (
                SELECT space.host_user_id FROM spaces space WHERE space.id = $1
              )
              WHEN 'capsule' THEN (
                SELECT capsule.id FROM space_capsules capsule
                WHERE capsule.space_id = $1 AND capsule.opened_at IS NULL
                ORDER BY capsule.created_at DESC LIMIT 1
              )
              ELSE object.content_id
            END,
            updated_at = now()
        WHERE object.scene_id = $2
          AND object.status = 'published'
          AND object.object_kind IN (
            'tourist_center', 'ai_guide', 'message_wall',
            'notice_board', 'host', 'capsule'
          )
        "#,
    )
    .bind(space_id)
    .bind(scene_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn seed_relation_portals(
    tx: &mut Transaction<'_, Postgres>,
    source_space_id: Uuid,
    scene_id: Uuid,
    created_by: Option<Uuid>,
) -> Result<(), sqlx::Error> {
    let relations = sqlx::query(
        r#"
        SELECT relation.target_space_id, relation.label_zh, relation.label_en,
               target.name_zh AS target_name_zh, target.name_en AS target_name_en
        FROM space_relations relation
        JOIN spaces target ON target.id = relation.target_space_id
        WHERE relation.source_space_id = $1
          AND relation.relation_kind IN ('child', 'portal')
          AND target.status IN ('active', 'expired')
        ORDER BY relation.created_at, target.name_zh
        LIMIT 8
        "#,
    )
    .bind(source_space_id)
    .fetch_all(&mut **tx)
    .await?;

    for (index, row) in relations.into_iter().enumerate() {
        let target_space_id: Uuid = row.try_get("target_space_id")?;
        let target_name_zh: String = row.try_get("target_name_zh")?;
        let target_name_en: Option<String> = row.try_get("target_name_en")?;
        let label_zh: Option<String> = row.try_get("label_zh")?;
        let label_en: Option<String> = row.try_get("label_en")?;
        let column = index % 4;
        let row_index = index / 4;
        let x = 20.0 + column as f64 * 20.0;
        let y = 78.0 + row_index as f64 * 9.0;
        let name_zh = label_zh.unwrap_or(target_name_zh);
        let name_en = label_en.or(target_name_en);
        let config = serde_json::json!({
            "action": "portal",
            "copy_zh": format!("穿过这里，可直接抵达「{}」。", name_zh),
            "copy_en": format!("Pass through to reach {} directly.", name_en.as_deref().unwrap_or(&name_zh)),
        });
        sqlx::query(
            r#"
            INSERT INTO scene_objects (
              scene_id, object_kind, name_zh, name_en, x, y, width, height,
              z_index, interaction_radius, target_space_id, target_spawn_key,
              config, status, created_by
            )
            SELECT $1, 'portal', $2, $3, $4, $5, 11, 14, 4, 10, $6,
                   'entrance', $7, 'published', $8
            WHERE NOT EXISTS (
              SELECT 1 FROM scene_objects
              WHERE scene_id = $1 AND object_kind = 'portal'
                AND target_space_id = $6 AND status <> 'archived'
            )
            "#,
        )
        .bind(scene_id)
        .bind(name_zh)
        .bind(name_en)
        .bind(x)
        .bind(y)
        .bind(target_space_id)
        .bind(config)
        .bind(created_by)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

pub async fn get_scene_bundle(
    pool: &PgPool,
    space_id: Uuid,
    scene_id: Option<Uuid>,
) -> Result<Option<SceneBundle>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT sc.id, sc.space_id, sc.slug, sc.kind, sc.name_zh, sc.name_en,
               sc.description_zh, sc.description_en, sc.layout, sc.is_default,
               sc.status, sc.version, sc.created_by, sc.created_at, sc.updated_at,
               sp.name_zh AS space_name_zh, sp.name_en AS space_name_en,
               sp.world_role, sp.is_public
        FROM scenes sc
        JOIN spaces sp ON sp.id = sc.space_id
        WHERE sc.space_id = $1
          AND ($2::uuid IS NULL OR sc.id = $2)
          AND sc.status = 'published'
          AND sp.status IN ('active', 'expired')
        ORDER BY CASE WHEN sc.id = $2 THEN 0 WHEN sc.is_default THEN 1 ELSE 2 END,
                 sc.updated_at DESC
        LIMIT 1
        "#,
    )
    .bind(space_id)
    .bind(scene_id)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };
    let scene = scene_from_row(&row)?;
    let objects = sqlx::query(
        r#"
        SELECT id, scene_id, object_kind, name_zh, name_en, x, y, width, height,
               z_index, interaction_radius, content_kind, content_id,
               target_space_id, target_scene_id, target_spawn_key, config, status
        FROM scene_objects
        WHERE scene_id = $1 AND status = 'published'
        ORDER BY z_index, created_at
        "#,
    )
    .bind(scene.id)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| object_from_row(&row))
    .collect::<Result<Vec<_>, _>>()?;

    let spawn_points = sqlx::query(
        r#"
        SELECT id, scene_id, key, name_zh, name_en, x, y, facing, is_default
        FROM scene_spawn_points
        WHERE scene_id = $1
        ORDER BY is_default DESC, created_at
        "#,
    )
    .bind(scene.id)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| spawn_from_row(&row))
    .collect::<Result<Vec<_>, _>>()?;

    Ok(Some(SceneBundle {
        space_id,
        space_name_zh: row.try_get("space_name_zh")?,
        space_name_en: row.try_get("space_name_en")?,
        space_role: SpaceRole::from_db(&row.try_get::<String, _>("world_role")?),
        is_public: row.try_get("is_public")?,
        scene,
        objects,
        spawn_points,
    }))
}

pub async fn list_space_relations(
    pool: &PgPool,
    space_id: Uuid,
) -> Result<Vec<SpaceRelation>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT id, source_space_id, target_space_id, relation_kind,
               label_zh, label_en, metadata, created_at
        FROM space_relations
        WHERE source_space_id = $1
        ORDER BY relation_kind, created_at
        "#,
    )
    .bind(space_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(SpaceRelation {
                id: row.try_get("id")?,
                source_space_id: row.try_get("source_space_id")?,
                target_space_id: row.try_get("target_space_id")?,
                kind: instant_domain::world::RelationKind::from_db(
                    &row.try_get::<String, _>("relation_kind")?,
                ),
                label_zh: row.try_get("label_zh")?,
                label_en: row.try_get("label_en")?,
                metadata: row.try_get("metadata")?,
                created_at: row.try_get("created_at")?,
            })
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
pub async fn enter_space(
    pool: &PgPool,
    user_id: Uuid,
    space_id: Uuid,
    scene_id: Option<Uuid>,
    spawn_key: Option<&str>,
    entry_method: EntryMethod,
    source_space_id: Option<Uuid>,
    source_object_id: Option<Uuid>,
    verification_state: &str,
) -> Result<EnterSpaceOutcome, sqlx::Error> {
    let bundle = if scene_id.is_none() {
        ensure_default_scene(pool, space_id, Some(user_id)).await?
    } else {
        match get_scene_bundle(pool, space_id, scene_id).await? {
            Some(bundle) => bundle,
            None => ensure_default_scene(pool, space_id, Some(user_id)).await?,
        }
    };
    let spawn = bundle
        .spawn_points
        .iter()
        .find(|point| spawn_key.is_some_and(|key| point.key == key))
        .or_else(|| bundle.spawn_points.iter().find(|point| point.is_default))
        .or_else(|| bundle.spawn_points.first())
        .cloned()
        .ok_or(sqlx::Error::RowNotFound)?;

    let mut tx = pool.begin().await?;
    let presence_row = sqlx::query(
        r#"
        INSERT INTO world_presences (
          subject_kind, subject_id, owner_user_id, space_id, scene_id,
          spawn_point_id, x, y, entry_method, entered_at, last_seen_at
        )
        VALUES ('user', $1, $1, $2, $3, $4, $5, $6, $7, now(), now())
        ON CONFLICT (subject_kind, subject_id) DO UPDATE
        SET owner_user_id = EXCLUDED.owner_user_id,
            space_id = EXCLUDED.space_id,
            scene_id = EXCLUDED.scene_id,
            spawn_point_id = EXCLUDED.spawn_point_id,
            x = EXCLUDED.x,
            y = EXCLUDED.y,
            entry_method = EXCLUDED.entry_method,
            entered_at = now(),
            last_seen_at = now()
        RETURNING id, subject_kind, subject_id, owner_user_id, space_id,
                  scene_id, spawn_point_id, x, y, entry_method,
                  entered_at, last_seen_at
        "#,
    )
    .bind(user_id)
    .bind(space_id)
    .bind(bundle.scene.id)
    .bind(spawn.id)
    .bind(spawn.x)
    .bind(spawn.y)
    .bind(entry_method.as_db())
    .fetch_one(&mut *tx)
    .await?;
    let presence = presence_from_row(&presence_row)?;

    let companions = sqlx::query(
        r#"
        INSERT INTO world_presences (
          subject_kind, subject_id, owner_user_id, space_id, scene_id,
          spawn_point_id, x, y, entry_method, entered_at, last_seen_at
        )
        SELECT 'companion', c.id, c.owner_id, $2, $3, $4,
               LEAST(96, $5 + ((ROW_NUMBER() OVER (ORDER BY c.created_at) - 1) * 3)),
               LEAST(96, $6 + 3), $7, now(), now()
        FROM companions c
        WHERE c.owner_id = $1
          AND c.death_at IS NULL
          AND c.state <> 'memorial'
        ON CONFLICT (subject_kind, subject_id) DO UPDATE
        SET owner_user_id = EXCLUDED.owner_user_id,
            space_id = EXCLUDED.space_id,
            scene_id = EXCLUDED.scene_id,
            spawn_point_id = EXCLUDED.spawn_point_id,
            x = EXCLUDED.x,
            y = EXCLUDED.y,
            entry_method = EXCLUDED.entry_method,
            entered_at = now(),
            last_seen_at = now()
        "#,
    )
    .bind(user_id)
    .bind(space_id)
    .bind(bundle.scene.id)
    .bind(spawn.id)
    .bind(spawn.x)
    .bind(spawn.y)
    .bind(entry_method.as_db())
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "UPDATE companions SET state = 'following', updated_at = now() WHERE owner_id = $1 AND death_at IS NULL AND state <> 'memorial'",
    )
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO space_entry_events (
          user_id, space_id, scene_id, spawn_point_id, entry_method,
          source_space_id, source_object_id, verification_state
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind(user_id)
    .bind(space_id)
    .bind(bundle.scene.id)
    .bind(spawn.id)
    .bind(entry_method.as_db())
    .bind(source_space_id)
    .bind(source_object_id)
    .bind(verification_state)
    .execute(&mut *tx)
    .await?;

    sqlx::query("UPDATE users SET last_active_at = now(), updated_at = now() WHERE id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(EnterSpaceOutcome {
        bundle,
        spawn,
        presence,
        companions_moved: companions.rows_affected() as i64,
    })
}

pub async fn get_user_presence(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<WorldPresence>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT id, subject_kind, subject_id, owner_user_id, space_id,
               scene_id, spawn_point_id, x, y, entry_method,
               entered_at, last_seen_at
        FROM world_presences
        WHERE subject_kind = 'user' AND subject_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    row.map(|row| presence_from_row(&row)).transpose()
}

pub async fn user_can_manage_scene(
    pool: &PgPool,
    space_id: Uuid,
    user_id: Uuid,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT EXISTS (
          SELECT 1 FROM spaces s
          WHERE s.id = $1 AND (s.host_user_id = $2 OR s.creator_id = $2)
          UNION ALL
          SELECT 1 FROM space_host_tenures ht
          WHERE ht.space_id = $1 AND ht.user_id = $2 AND ht.status = 'active'
        )
        "#,
    )
    .bind(space_id)
    .bind(user_id)
    .fetch_one(pool)
    .await
}

fn host_identity_from_row(
    row: &sqlx::postgres::PgRow,
    include_email: bool,
) -> Result<SpaceHostIdentity, sqlx::Error> {
    Ok(SpaceHostIdentity {
        tenure_id: row.try_get("tenure_id")?,
        user_id: row.try_get("user_id")?,
        display_name: row.try_get("display_name")?,
        email: if include_email {
            row.try_get("email")?
        } else {
            None
        },
        role: HostTenureRole::from_db(&row.try_get::<String, _>("role")?),
        status: HostTenureStatus::from_db(&row.try_get::<String, _>("status")?),
        started_at: row.try_get("started_at")?,
        ended_at: row.try_get("ended_at")?,
        note: row.try_get("note")?,
    })
}

pub async fn active_host_role(
    pool: &PgPool,
    space_id: Uuid,
    user_id: Uuid,
) -> Result<Option<HostTenureRole>, sqlx::Error> {
    let role: Option<String> = sqlx::query_scalar(
        r#"
        SELECT role
        FROM space_host_tenures
        WHERE space_id = $1 AND user_id = $2 AND status = 'active'
        ORDER BY CASE role WHEN 'primary' THEN 0 WHEN 'co_host' THEN 1 ELSE 2 END
        LIMIT 1
        "#,
    )
    .bind(space_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    Ok(role.as_deref().map(HostTenureRole::from_db))
}

pub async fn get_space_governance(
    pool: &PgPool,
    space_id: Uuid,
    viewer_user_id: Option<Uuid>,
    include_private: bool,
    admin_override: bool,
) -> Result<Option<SpaceGovernanceSnapshot>, sqlx::Error> {
    let state_row = sqlx::query(
        "SELECT host_governance_state, host_recruitment_note FROM spaces WHERE id = $1",
    )
    .bind(space_id)
    .fetch_optional(pool)
    .await?;
    let Some(state_row) = state_row else {
        return Ok(None);
    };

    let host_rows = sqlx::query(
        r#"
        SELECT ht.id AS tenure_id, ht.user_id,
               COALESCE(NULLIF(u.name, ''), split_part(u.email, '@', 1)) AS display_name,
               u.email, ht.role, ht.status, ht.started_at, ht.ended_at, ht.note
        FROM space_host_tenures ht
        JOIN users u ON u.id = ht.user_id
        WHERE ht.space_id = $1
        ORDER BY CASE WHEN ht.status = 'active' THEN 0 ELSE 1 END,
                 CASE ht.role WHEN 'primary' THEN 0 WHEN 'co_host' THEN 1 ELSE 2 END,
                 ht.started_at DESC
        "#,
    )
    .bind(space_id)
    .fetch_all(pool)
    .await?;

    let mut active_hosts = Vec::new();
    let mut past_hosts = Vec::new();
    for row in host_rows {
        let host = host_identity_from_row(&row, include_private)?;
        if host.status == HostTenureStatus::Active {
            active_hosts.push(host);
        } else {
            past_hosts.push(host);
        }
    }

    let current_user_role = match viewer_user_id {
        Some(user_id) => active_host_role(pool, space_id, user_id).await?,
        None => None,
    };
    let can_manage_content = admin_override || current_user_role.is_some();
    let can_manage_governance =
        admin_override || matches!(current_user_role, Some(HostTenureRole::Primary));

    let events = if include_private {
        let rows = sqlx::query(
            r#"
            SELECT e.id, e.action,
                   COALESCE(NULLIF(actor.name, ''), split_part(actor.email, '@', 1)) AS actor_name,
                   COALESCE(NULLIF(from_user.name, ''), split_part(from_user.email, '@', 1)) AS from_name,
                   COALESCE(NULLIF(to_user.name, ''), split_part(to_user.email, '@', 1)) AS to_name,
                   e.note, e.created_at
            FROM space_governance_events e
            LEFT JOIN users actor ON actor.id = e.actor_id
            LEFT JOIN users from_user ON from_user.id = e.from_user_id
            LEFT JOIN users to_user ON to_user.id = e.to_user_id
            WHERE e.space_id = $1
            ORDER BY e.created_at DESC
            LIMIT 100
            "#,
        )
        .bind(space_id)
        .fetch_all(pool)
        .await?;
        rows.into_iter()
            .map(|row| {
                Ok(SpaceGovernanceEvent {
                    id: row.try_get("id")?,
                    action: row.try_get("action")?,
                    actor_name: row.try_get("actor_name")?,
                    from_name: row.try_get("from_name")?,
                    to_name: row.try_get("to_name")?,
                    note: row.try_get("note")?,
                    created_at: row.try_get("created_at")?,
                })
            })
            .collect::<Result<Vec<_>, sqlx::Error>>()?
    } else {
        Vec::new()
    };

    Ok(Some(SpaceGovernanceSnapshot {
        space_id,
        state: HostGovernanceState::from_db(
            &state_row.try_get::<String, _>("host_governance_state")?,
        ),
        recruitment_note: state_row.try_get("host_recruitment_note")?,
        current_user_role,
        can_manage_content,
        can_manage_governance,
        active_hosts,
        past_hosts,
        events,
    }))
}

async fn record_governance_event(
    tx: &mut Transaction<'_, Postgres>,
    space_id: Uuid,
    actor_id: Uuid,
    action: &str,
    from_user_id: Option<Uuid>,
    to_user_id: Option<Uuid>,
    note: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO space_governance_events
          (space_id, actor_id, action, from_user_id, to_user_id, note)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(space_id)
    .bind(actor_id)
    .bind(action)
    .bind(from_user_id)
    .bind(to_user_id)
    .bind(note)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn appoint_supporting_host(
    pool: &PgPool,
    space_id: Uuid,
    target_user_id: Uuid,
    role: HostTenureRole,
    actor_id: Uuid,
    note: Option<&str>,
) -> Result<(), sqlx::Error> {
    debug_assert!(matches!(
        role,
        HostTenureRole::CoHost | HostTenureRole::Steward
    ));
    let mut tx = pool.begin().await?;
    sqlx::query("SELECT id FROM spaces WHERE id = $1 FOR UPDATE")
        .bind(space_id)
        .fetch_one(&mut *tx)
        .await?;
    sqlx::query(
        "UPDATE space_host_tenures SET status = 'ended', ended_at = now() WHERE space_id = $1 AND user_id = $2 AND status = 'active'",
    )
    .bind(space_id)
    .bind(target_user_id)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "INSERT INTO space_host_tenures (space_id, user_id, role, status, granted_by, note) VALUES ($1, $2, $3, 'active', $4, $5)",
    )
    .bind(space_id)
    .bind(target_user_id)
    .bind(role.as_db())
    .bind(actor_id)
    .bind(note)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "INSERT INTO space_members (space_id, user_id, role) VALUES ($1, $2, $3) ON CONFLICT (space_id, user_id) DO UPDATE SET role = EXCLUDED.role",
    )
    .bind(space_id)
    .bind(target_user_id)
    .bind(role.as_db())
    .execute(&mut *tx)
    .await?;
    record_governance_event(
        &mut tx,
        space_id,
        actor_id,
        if role == HostTenureRole::CoHost {
            "appoint_co_host"
        } else {
            "appoint_steward"
        },
        None,
        Some(target_user_id),
        note,
    )
    .await?;
    tx.commit().await
}

pub async fn remove_supporting_host(
    pool: &PgPool,
    space_id: Uuid,
    target_user_id: Uuid,
    actor_id: Uuid,
    self_leave: bool,
    note: Option<&str>,
) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("SELECT id FROM spaces WHERE id = $1 FOR UPDATE")
        .bind(space_id)
        .fetch_one(&mut *tx)
        .await?;
    let result = sqlx::query(
        "UPDATE space_host_tenures SET status = 'ended', ended_at = now(), note = COALESCE($3, note) WHERE space_id = $1 AND user_id = $2 AND status = 'active' AND role <> 'primary'",
    )
    .bind(space_id)
    .bind(target_user_id)
    .bind(note)
    .execute(&mut *tx)
    .await?;
    if result.rows_affected() == 0 {
        tx.rollback().await?;
        return Ok(false);
    }
    sqlx::query("UPDATE space_members SET role = 'member' WHERE space_id = $1 AND user_id = $2")
        .bind(space_id)
        .bind(target_user_id)
        .execute(&mut *tx)
        .await?;
    record_governance_event(
        &mut tx,
        space_id,
        actor_id,
        if self_leave {
            "leave_host"
        } else {
            "remove_host"
        },
        Some(target_user_id),
        None,
        note,
    )
    .await?;
    tx.commit().await?;
    Ok(true)
}

pub async fn transfer_primary_host(
    pool: &PgPool,
    space_id: Uuid,
    target_user_id: Uuid,
    actor_id: Uuid,
    note: Option<&str>,
) -> Result<Option<Uuid>, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let old_primary: Option<Uuid> =
        sqlx::query_scalar("SELECT host_user_id FROM spaces WHERE id = $1 FOR UPDATE")
            .bind(space_id)
            .fetch_one(&mut *tx)
            .await?;
    if old_primary == Some(target_user_id) {
        tx.rollback().await?;
        return Ok(old_primary);
    }
    sqlx::query(
        "UPDATE space_host_tenures SET status = 'ended', ended_at = now() WHERE space_id = $1 AND status = 'active' AND (role = 'primary' OR user_id = $2)",
    )
    .bind(space_id)
    .bind(target_user_id)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "INSERT INTO space_host_tenures (space_id, user_id, role, status, granted_by, note) VALUES ($1, $2, 'primary', 'active', $3, $4)",
    )
    .bind(space_id)
    .bind(target_user_id)
    .bind(actor_id)
    .bind(note)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "UPDATE spaces SET host_user_id = $2, host_governance_state = 'hosted', updated_at = now() WHERE id = $1",
    )
    .bind(space_id)
    .bind(target_user_id)
    .execute(&mut *tx)
    .await?;
    if let Some(old_user_id) = old_primary.filter(|id| *id != target_user_id) {
        sqlx::query(
            "UPDATE space_members SET role = 'member' WHERE space_id = $1 AND user_id = $2",
        )
        .bind(space_id)
        .bind(old_user_id)
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query(
        "INSERT INTO space_members (space_id, user_id, role) VALUES ($1, $2, 'host') ON CONFLICT (space_id, user_id) DO UPDATE SET role = 'host'",
    )
    .bind(space_id)
    .bind(target_user_id)
    .execute(&mut *tx)
    .await?;
    record_governance_event(
        &mut tx,
        space_id,
        actor_id,
        "transfer_primary",
        old_primary,
        Some(target_user_id),
        note,
    )
    .await?;
    tx.commit().await?;
    Ok(old_primary)
}

pub async fn release_primary_host(
    pool: &PgPool,
    space_id: Uuid,
    actor_id: Uuid,
    next_state: HostGovernanceState,
    note: Option<&str>,
) -> Result<Option<Uuid>, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let old_primary: Option<Uuid> =
        sqlx::query_scalar("SELECT host_user_id FROM spaces WHERE id = $1 FOR UPDATE")
            .bind(space_id)
            .fetch_one(&mut *tx)
            .await?;
    let Some(old_primary_id) = old_primary else {
        sqlx::query(
            "UPDATE spaces SET host_governance_state = $2, host_recruitment_note = COALESCE($3, host_recruitment_note), updated_at = now() WHERE id = $1",
        )
        .bind(space_id)
        .bind(next_state.as_db())
        .bind(note)
        .execute(&mut *tx)
        .await?;
        record_governance_event(
            &mut tx,
            space_id,
            actor_id,
            if next_state == HostGovernanceState::SystemCare {
                "place_in_system_care"
            } else {
                "release_to_recruiting"
            },
            None,
            None,
            note,
        )
        .await?;
        tx.commit().await?;
        return Ok(None);
    };
    sqlx::query(
        "UPDATE space_host_tenures SET status = 'ended', ended_at = now(), note = COALESCE($3, note) WHERE space_id = $1 AND user_id = $2 AND role = 'primary' AND status = 'active'",
    )
    .bind(space_id)
    .bind(old_primary_id)
    .bind(note)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "UPDATE spaces SET host_user_id = NULL, host_governance_state = $2, host_recruitment_note = COALESCE($3, host_recruitment_note), updated_at = now() WHERE id = $1",
    )
    .bind(space_id)
    .bind(next_state.as_db())
    .bind(note)
    .execute(&mut *tx)
    .await?;
    sqlx::query("UPDATE space_members SET role = 'member' WHERE space_id = $1 AND user_id = $2")
        .bind(space_id)
        .bind(old_primary_id)
        .execute(&mut *tx)
        .await?;
    record_governance_event(
        &mut tx,
        space_id,
        actor_id,
        if next_state == HostGovernanceState::SystemCare {
            "place_in_system_care"
        } else {
            "release_to_recruiting"
        },
        Some(old_primary_id),
        None,
        note,
    )
    .await?;
    tx.commit().await?;
    Ok(Some(old_primary_id))
}

pub async fn update_recruitment_note(
    pool: &PgPool,
    space_id: Uuid,
    actor_id: Uuid,
    note: Option<&str>,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("UPDATE spaces SET host_recruitment_note = $2, updated_at = now() WHERE id = $1")
        .bind(space_id)
        .bind(note)
        .execute(&mut *tx)
        .await?;
    record_governance_event(
        &mut tx,
        space_id,
        actor_id,
        "update_recruitment_note",
        None,
        None,
        note,
    )
    .await?;
    tx.commit().await
}
