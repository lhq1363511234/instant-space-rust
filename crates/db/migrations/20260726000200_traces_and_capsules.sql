-- Traces and time capsules: what a place remembers after everyone leaves.
--
-- Chat is for whoever is standing there right now and scrolls away. These two
-- tables are the opposite: they accumulate, and they are addressed to people
-- who are not here yet.
--
-- `space_traces`  — the guest book / wall / lock. Public, permanent, ordered.
-- `space_capsules` — a sealed letter. Opening it requires a passphrase the
--                    author hands over privately AND standing at the place.

-- How the writer proved they were actually there. Order matters: a scanned QR
-- at the location is stronger evidence than a browser geolocation fix.
CREATE TYPE presence_proof AS ENUM ('scan', 'geo', 'discord', 'remote');

CREATE TABLE space_traces (
  id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  space_id uuid NOT NULL REFERENCES spaces(id) ON DELETE CASCADE,
  author_id uuid REFERENCES users(id) ON DELETE SET NULL,
  author_name text NOT NULL,
  body text NOT NULL,
  proof presence_proof NOT NULL DEFAULT 'remote',
  -- Where the writer actually stood, when we know it. Kept for the record,
  -- never shown at full precision.
  proof_lat double precision,
  proof_lng double precision,
  proof_distance_m double precision,
  -- Weather / season at the moment of writing, so that reading this back in
  -- three years carries the day with it.
  weather text,
  -- A trace can be lifted out of the chat transcript rather than written fresh.
  source_message_id uuid REFERENCES chat_messages(id) ON DELETE SET NULL,
  hidden boolean NOT NULL DEFAULT false,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX space_traces_space_idx ON space_traces (space_id, created_at DESC) WHERE NOT hidden;
CREATE INDEX space_traces_author_idx ON space_traces (author_id);

CREATE TABLE space_capsules (
  id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  space_id uuid NOT NULL REFERENCES spaces(id) ON DELETE CASCADE,
  author_id uuid REFERENCES users(id) ON DELETE SET NULL,
  author_name text NOT NULL,
  -- Shown before opening: who it is for, so a stranger knows to walk away.
  recipient_hint text NOT NULL,
  body text NOT NULL,
  -- Only the hash. The author tells the recipient the passphrase themselves;
  -- if it is lost the capsule stays shut, which is the point of burying it.
  passphrase_hash text NOT NULL,
  -- Standing at the place is always required. This is the slack we allow.
  radius_m integer NOT NULL DEFAULT 300,
  opens_at timestamptz,
  opened_at timestamptz,
  opened_by uuid REFERENCES users(id) ON DELETE SET NULL,
  opened_by_name text,
  -- Failed attempts are counted so a capsule cannot be brute-forced by a
  -- stranger who happens to be standing in the right spot.
  failed_attempts integer NOT NULL DEFAULT 0,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX space_capsules_space_idx ON space_capsules (space_id, created_at DESC);
CREATE INDEX space_capsules_open_idx ON space_capsules (space_id) WHERE opened_at IS NULL;
