CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TYPE space_type AS ENUM ('scenic', 'food', 'park', 'transit', 'event', 'custom');
CREATE TYPE space_status AS ENUM ('active', 'expired', 'closed', 'archived', 'template');
CREATE TYPE guide_status AS ENUM ('draft', 'published', 'archived');

CREATE TABLE users (
  id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  email text NOT NULL UNIQUE,
  name text,
  password_hash text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE sessions (
  id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  user_id uuid REFERENCES users(id) ON DELETE CASCADE,
  admin_username text,
  token_hash text NOT NULL UNIQUE,
  expires_at timestamptz NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  CHECK ((user_id IS NOT NULL) OR (admin_username IS NOT NULL))
);

CREATE TABLE spaces (
  id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  name_en text,
  name_zh text NOT NULL,
  space_type space_type NOT NULL,
  custom_type text,
  category text,
  province text,
  city text,
  district text,
  lat double precision NOT NULL,
  lng double precision NOT NULL,
  online_count integer NOT NULL DEFAULT 0,
  tag_en text,
  tag_zh text,
  description_en text,
  description_zh text,
  is_public boolean NOT NULL DEFAULT true,
  password_hash text NOT NULL,
  password_version integer NOT NULL DEFAULT 1,
  duration_hours integer NOT NULL DEFAULT 24,
  expires_at timestamptz,
  status space_status NOT NULL DEFAULT 'active',
  resident boolean NOT NULL DEFAULT false,
  resident_days integer,
  resident_apply_at timestamptz,
  discord_group text,
  qq_group text,
  closed_at timestamptz,
  closed_by text,
  creator_id uuid REFERENCES users(id) ON DELETE SET NULL,
  host_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
  template_id uuid,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE space_members (
  id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  space_id uuid NOT NULL REFERENCES spaces(id) ON DELETE CASCADE,
  user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  role text NOT NULL DEFAULT 'member',
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (space_id, user_id)
);

CREATE TABLE space_templates (
  id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  source_space_id uuid REFERENCES spaces(id) ON DELETE SET NULL,
  name_zh text NOT NULL,
  name_en text,
  data jsonb NOT NULL,
  created_by uuid REFERENCES users(id) ON DELETE SET NULL,
  created_at timestamptz NOT NULL DEFAULT now()
);

ALTER TABLE spaces
  ADD CONSTRAINT spaces_template_fk
  FOREIGN KEY (template_id) REFERENCES space_templates(id) ON DELETE SET NULL;

CREATE TABLE access_sessions (
  id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  space_id uuid NOT NULL REFERENCES spaces(id) ON DELETE CASCADE,
  token_hash text NOT NULL UNIQUE,
  password_version integer NOT NULL,
  expires_at timestamptz NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE guides (
  id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  title_zh text NOT NULL,
  title_en text,
  summary_zh text,
  summary_en text,
  content_zh text,
  content_en text,
  guide_type text NOT NULL DEFAULT 'attraction',
  category text,
  province text NOT NULL,
  city text NOT NULL,
  district text,
  spot_name text,
  status guide_status NOT NULL DEFAULT 'draft',
  featured boolean NOT NULL DEFAULT false,
  author_id uuid REFERENCES users(id) ON DELETE SET NULL,
  author_name text,
  space_id uuid REFERENCES spaces(id) ON DELETE SET NULL,
  cover_image_url text,
  images jsonb NOT NULL DEFAULT '[]'::jsonb,
  sections jsonb NOT NULL DEFAULT '[]'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE chat_messages (
  id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  space_id uuid NOT NULL REFERENCES spaces(id) ON DELETE CASCADE,
  sender text NOT NULL,
  sender_id uuid REFERENCES users(id) ON DELETE SET NULL,
  body text NOT NULL,
  password_version integer NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE helps (
  id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  space_id uuid REFERENCES spaces(id) ON DELETE CASCADE,
  body text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE games (
  id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  space_id uuid REFERENCES spaces(id) ON DELETE CASCADE,
  name text NOT NULL,
  state jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE locations (
  id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  province text NOT NULL,
  city text,
  district text,
  spot_name text,
  source text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (province, city, district, spot_name, source)
);

CREATE INDEX spaces_home_map_idx ON spaces (status, is_public, space_type);
CREATE INDEX spaces_location_idx ON spaces (province, city, district);
CREATE INDEX guides_hierarchy_idx ON guides (province, city, district, spot_name, status);
CREATE INDEX chat_messages_space_created_idx ON chat_messages (space_id, created_at);
