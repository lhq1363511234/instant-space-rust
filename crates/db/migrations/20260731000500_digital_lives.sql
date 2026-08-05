-- Digital lives: the pets (and, one day, the people) that live with us.
--
-- Design (v3 "云上家"):
--   cloud_homes      — every user has one cloud home. Friends may visit, but
--                      entering requires the home passphrase, like a door key.
--                      The owner never needs it.
--   companions       — family members: pets now, humans later. Three states:
--                      following (owner active -> travels with them),
--                      at_home (owner idle -> waits at the cloud home),
--                      memorial (only after death -> a memorial space).
--   companion_trails — footprints. When the owner proves presence at a space,
--                      the companion records "it was here too" automatically,
--                      plus optional handwritten snippets for distillation.
--   digital_lives    — the distilled memorial, created only after death:
--                      epitaph, biography chapters, inscription, life map.
--                      Versioned (distill_version / content_version).
--   life_prayers     — incense, flowers, lanterns, and words left by visitors.
--
-- Copy follows the life-distill skill's Song-style rules: 白描、短句、留白、
-- 时节与地点意象、哀而不伤. The subject_type column is deliberately neutral
-- (pet today, human tomorrow) so the same model can be upgraded later.

-- The online/offline signal: 24h of activity counts as "online". Touched on
-- login and on authenticated lives API calls.
ALTER TABLE users ADD COLUMN last_active_at timestamptz;

CREATE TABLE cloud_homes (
  id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  owner_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE UNIQUE,
  name text NOT NULL DEFAULT '云上家',
  -- A motto / 门联 shown at the top of the home page.
  motto text,
  -- Entering requires the passphrase (owner never needs it).
  passphrase_hash text,
  -- A short note shown to visitors before they knock.
  door_note text,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX cloud_homes_owner_idx ON cloud_homes (owner_id);

CREATE TABLE companions (
  id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  owner_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  home_id uuid NOT NULL REFERENCES cloud_homes(id) ON DELETE CASCADE,
  -- pet today, human tomorrow; the model is deliberately neutral.
  subject_type text NOT NULL DEFAULT 'pet',
  name text NOT NULL,
  species text,
  breed text,
  gender text,
  birth_at date,
  death_at date,
  state text NOT NULL DEFAULT 'at_home'
    CHECK (state IN ('following', 'at_home', 'memorial')),
  avatar_url text,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX companions_owner_idx ON companions (owner_id, state);

CREATE TABLE companion_trails (
  id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  companion_id uuid NOT NULL REFERENCES companions(id) ON DELETE CASCADE,
  owner_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  space_id uuid REFERENCES spaces(id) ON DELETE SET NULL,
  space_name text,
  place_name text,
  proof text NOT NULL DEFAULT 'remote',
  lat double precision,
  lng double precision,
  noted_at timestamptz NOT NULL DEFAULT now(),
  snippet text,
  season_hint text
);

CREATE INDEX companion_trails_companion_idx
  ON companion_trails (companion_id, noted_at DESC);

CREATE TABLE digital_lives (
  id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  companion_id uuid NOT NULL REFERENCES companions(id) ON DELETE CASCADE UNIQUE,
  owner_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  subject_type text NOT NULL DEFAULT 'pet',
  name text NOT NULL,
  -- 碑额
  epitaph text NOT NULL DEFAULT '',
  -- 小传: free chapters [{title, body}]
  biography jsonb NOT NULL DEFAULT '[]',
  -- 铭文
  inscription text NOT NULL DEFAULT '',
  -- 生命地图: [{place, season, deed}]
  life_map jsonb NOT NULL DEFAULT '[]',
  memorial_date date,
  incense_count integer NOT NULL DEFAULT 0,
  visitor_count integer NOT NULL DEFAULT 0,
  distill_version integer NOT NULL DEFAULT 1,
  content_version integer NOT NULL DEFAULT 1,
  published boolean NOT NULL DEFAULT true,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX digital_lives_owner_idx ON digital_lives (owner_id);

CREATE TABLE life_prayers (
  id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  life_id uuid NOT NULL REFERENCES digital_lives(id) ON DELETE CASCADE,
  visitor_id uuid REFERENCES users(id) ON DELETE SET NULL,
  visitor_name text NOT NULL,
  kind text NOT NULL DEFAULT 'incense'
    CHECK (kind IN ('incense', 'flower', 'lantern', 'word')),
  message text,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX life_prayers_life_idx ON life_prayers (life_id, created_at DESC);
