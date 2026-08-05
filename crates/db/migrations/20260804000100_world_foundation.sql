-- Phase 9: Space world foundation.
--
-- Space is the durable identity; Scene is the environment; Scene Object is a
-- real-world-shaped carrier for content; Portal is exploratory navigation;
-- Presence and entry events support direct teleport from map/search/QR/link.

ALTER TABLE spaces
  ADD COLUMN world_role text NOT NULL DEFAULT 'place'
  CHECK (world_role IN ('hub', 'place', 'micro', 'home', 'memorial'));

UPDATE spaces SET world_role = 'home' WHERE category = 'cloud_home';

CREATE TABLE scenes (
  id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  space_id uuid NOT NULL REFERENCES spaces(id) ON DELETE CASCADE,
  slug text NOT NULL DEFAULT 'main',
  kind text NOT NULL DEFAULT 'place'
    CHECK (kind IN ('hub', 'place', 'home', 'memorial', 'interior')),
  name_zh text NOT NULL,
  name_en text,
  description_zh text,
  description_en text,
  layout jsonb NOT NULL DEFAULT '{}'::jsonb,
  is_default boolean NOT NULL DEFAULT false,
  status text NOT NULL DEFAULT 'draft'
    CHECK (status IN ('draft', 'published', 'archived')),
  version integer NOT NULL DEFAULT 1 CHECK (version > 0),
  created_by uuid REFERENCES users(id) ON DELETE SET NULL,
  published_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (space_id, slug)
);

CREATE UNIQUE INDEX scenes_one_default_per_space_idx
  ON scenes (space_id) WHERE is_default;
CREATE INDEX scenes_space_status_idx ON scenes (space_id, status, updated_at DESC);

CREATE TABLE scene_spawn_points (
  id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  scene_id uuid NOT NULL REFERENCES scenes(id) ON DELETE CASCADE,
  key text NOT NULL,
  name_zh text NOT NULL,
  name_en text,
  x double precision NOT NULL DEFAULT 50 CHECK (x >= 0 AND x <= 100),
  y double precision NOT NULL DEFAULT 82 CHECK (y >= 0 AND y <= 100),
  facing text NOT NULL DEFAULT 'north'
    CHECK (facing IN ('north', 'east', 'south', 'west')),
  is_default boolean NOT NULL DEFAULT false,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (scene_id, key)
);

CREATE UNIQUE INDEX scene_spawn_points_one_default_idx
  ON scene_spawn_points (scene_id) WHERE is_default;

CREATE TABLE scene_objects (
  id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  scene_id uuid NOT NULL REFERENCES scenes(id) ON DELETE CASCADE,
  object_kind text NOT NULL
    CHECK (object_kind IN (
      'tourist_center', 'ai_guide', 'message_wall', 'notice_board',
      'host', 'portal', 'capsule', 'display', 'building', 'decoration'
    )),
  name_zh text NOT NULL,
  name_en text,
  x double precision NOT NULL CHECK (x >= 0 AND x <= 100),
  y double precision NOT NULL CHECK (y >= 0 AND y <= 100),
  width double precision NOT NULL DEFAULT 12 CHECK (width > 0 AND width <= 100),
  height double precision NOT NULL DEFAULT 12 CHECK (height > 0 AND height <= 100),
  z_index integer NOT NULL DEFAULT 1,
  interaction_radius double precision NOT NULL DEFAULT 8 CHECK (interaction_radius > 0),
  content_kind text,
  content_id uuid,
  target_space_id uuid REFERENCES spaces(id) ON DELETE SET NULL,
  target_scene_id uuid REFERENCES scenes(id) ON DELETE SET NULL,
  target_spawn_key text,
  config jsonb NOT NULL DEFAULT '{}'::jsonb,
  status text NOT NULL DEFAULT 'published'
    CHECK (status IN ('draft', 'published', 'archived')),
  created_by uuid REFERENCES users(id) ON DELETE SET NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX scene_objects_scene_status_idx
  ON scene_objects (scene_id, status, z_index, created_at);
CREATE INDEX scene_objects_target_space_idx
  ON scene_objects (target_space_id) WHERE target_space_id IS NOT NULL;

CREATE TABLE space_relations (
  id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  source_space_id uuid NOT NULL REFERENCES spaces(id) ON DELETE CASCADE,
  target_space_id uuid NOT NULL REFERENCES spaces(id) ON DELETE CASCADE,
  relation_kind text NOT NULL
    CHECK (relation_kind IN ('parent', 'child', 'related', 'portal', 'home_of', 'memorial_of')),
  label_zh text,
  label_en text,
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_by uuid REFERENCES users(id) ON DELETE SET NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  CHECK (source_space_id <> target_space_id),
  UNIQUE (source_space_id, target_space_id, relation_kind)
);

CREATE INDEX space_relations_source_idx ON space_relations (source_space_id, relation_kind);
CREATE INDEX space_relations_target_idx ON space_relations (target_space_id, relation_kind);

CREATE TABLE space_host_tenures (
  id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  space_id uuid NOT NULL REFERENCES spaces(id) ON DELETE CASCADE,
  user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  role text NOT NULL DEFAULT 'primary'
    CHECK (role IN ('primary', 'co_host', 'steward')),
  status text NOT NULL DEFAULT 'active'
    CHECK (status IN ('active', 'ended', 'revoked')),
  started_at timestamptz NOT NULL DEFAULT now(),
  ended_at timestamptz,
  granted_by uuid REFERENCES users(id) ON DELETE SET NULL,
  note text,
  created_at timestamptz NOT NULL DEFAULT now(),
  CHECK ((status = 'active' AND ended_at IS NULL) OR status <> 'active')
);

CREATE UNIQUE INDEX space_host_one_primary_idx
  ON space_host_tenures (space_id) WHERE role = 'primary' AND status = 'active';
CREATE INDEX space_host_tenures_user_idx
  ON space_host_tenures (user_id, status, started_at DESC);

INSERT INTO space_host_tenures (space_id, user_id, role, status, started_at)
SELECT id, host_user_id, 'primary', 'active', created_at
FROM spaces
WHERE host_user_id IS NOT NULL
ON CONFLICT DO NOTHING;

CREATE TABLE world_presences (
  id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  subject_kind text NOT NULL CHECK (subject_kind IN ('user', 'companion')),
  subject_id uuid NOT NULL,
  owner_user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  space_id uuid NOT NULL REFERENCES spaces(id) ON DELETE CASCADE,
  scene_id uuid NOT NULL REFERENCES scenes(id) ON DELETE CASCADE,
  spawn_point_id uuid REFERENCES scene_spawn_points(id) ON DELETE SET NULL,
  x double precision NOT NULL DEFAULT 50 CHECK (x >= 0 AND x <= 100),
  y double precision NOT NULL DEFAULT 82 CHECK (y >= 0 AND y <= 100),
  entry_method text NOT NULL DEFAULT 'direct'
    CHECK (entry_method IN (
      'direct', 'search', 'map', 'link', 'qr', 'nfc', 'wifi', 'ai',
      'portal', 'capsule', 'history', 'home'
    )),
  entered_at timestamptz NOT NULL DEFAULT now(),
  last_seen_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (subject_kind, subject_id)
);

CREATE INDEX world_presences_scene_idx ON world_presences (scene_id, last_seen_at DESC);
CREATE INDEX world_presences_owner_idx ON world_presences (owner_user_id, last_seen_at DESC);

CREATE TABLE space_entry_events (
  id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  user_id uuid REFERENCES users(id) ON DELETE SET NULL,
  space_id uuid NOT NULL REFERENCES spaces(id) ON DELETE CASCADE,
  scene_id uuid NOT NULL REFERENCES scenes(id) ON DELETE CASCADE,
  spawn_point_id uuid REFERENCES scene_spawn_points(id) ON DELETE SET NULL,
  entry_method text NOT NULL,
  source_space_id uuid REFERENCES spaces(id) ON DELETE SET NULL,
  source_object_id uuid REFERENCES scene_objects(id) ON DELETE SET NULL,
  verification_state text NOT NULL DEFAULT 'not_required'
    CHECK (verification_state IN ('not_required', 'verified', 'owner', 'admin')),
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  entered_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX space_entry_events_user_idx ON space_entry_events (user_id, entered_at DESC);
CREATE INDEX space_entry_events_space_idx ON space_entry_events (space_id, entered_at DESC);

-- Existing cloud homes become the first concrete Scene sample. Other Spaces
-- are initialized lazily when their host opens the scene editor or a visitor
-- enters the world route for the first time.
INSERT INTO scenes (
  space_id, slug, kind, name_zh, name_en, description_zh, description_en,
  layout, is_default, status, created_by, published_at
)
SELECT
  s.id, 'courtyard', 'home', ch.name, 'Cloud Home',
  COALESCE(ch.motto, '开门见山，进门是家。'),
  'Beyond the door, home.',
  '{"theme":"song_courtyard","width":100,"height":100}'::jsonb,
  true, 'published', ch.owner_id, now()
FROM cloud_homes ch
JOIN spaces s ON s.id = ch.space_id
ON CONFLICT (space_id, slug) DO NOTHING;

INSERT INTO scene_spawn_points (scene_id, key, name_zh, name_en, x, y, facing, is_default)
SELECT id, 'gate', '门前', 'At the gate', 50, 84, 'north', true
FROM scenes
WHERE kind = 'home' AND slug = 'courtyard'
ON CONFLICT (scene_id, key) DO NOTHING;

INSERT INTO scene_objects (scene_id, object_kind, name_zh, name_en, x, y, width, height, z_index, config)
SELECT s.id, seed.kind, seed.name_zh, seed.name_en, seed.x, seed.y, seed.w, seed.h, seed.z, seed.config
FROM scenes s
CROSS JOIN (VALUES
  ('building', '家屋', 'Home', 50.0, 25.0, 28.0, 24.0, 2, '{"action":"home","copy_zh":"屋中收着一家人的日常。","copy_en":"The household keeps its everyday life here."}'::jsonb),
  ('message_wall', '家书墙', 'Family wall', 22.0, 47.0, 14.0, 18.0, 3, '{"action":"stories","copy_zh":"来者可在此读家书、看旧日片段。","copy_en":"Read family notes and remembered moments."}'::jsonb),
  ('display', '足迹册', 'Trail album', 76.0, 48.0, 15.0, 16.0, 3, '{"action":"trails","copy_zh":"主人去过的地方，也记着同行的家人。","copy_en":"Places visited by the owner also remember their companions."}'::jsonb),
  ('decoration', '桂树', 'Osmanthus tree', 73.0, 25.0, 15.0, 25.0, 1, '{"action":"memorial","copy_zh":"生者在侧，逝者归于桂下。","copy_en":"The living stay close; the remembered rest beneath the tree."}'::jsonb)
) AS seed(kind, name_zh, name_en, x, y, w, h, z, config)
WHERE s.kind = 'home' AND s.slug = 'courtyard'
  AND NOT EXISTS (SELECT 1 FROM scene_objects o WHERE o.scene_id = s.id);
