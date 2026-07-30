-- Author-owned Agent REST API. Raw keys are never stored: only an Argon2 hash
-- and a short non-secret lookup prefix are persisted.
CREATE TABLE IF NOT EXISTS agent_api_keys (
  id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  name text NOT NULL,
  key_prefix text NOT NULL UNIQUE,
  key_hash text NOT NULL,
  scopes text[] NOT NULL DEFAULT ARRAY[]::text[],
  rate_limit_per_minute integer NOT NULL DEFAULT 60 CHECK (rate_limit_per_minute BETWEEN 1 AND 600),
  created_at timestamptz NOT NULL DEFAULT now(),
  last_used_at timestamptz,
  revoked_at timestamptz
);

CREATE INDEX IF NOT EXISTS agent_api_keys_user_idx
  ON agent_api_keys (user_id, created_at DESC);

CREATE TABLE IF NOT EXISTS agent_api_audit_log (
  id bigserial PRIMARY KEY,
  key_id uuid REFERENCES agent_api_keys(id) ON DELETE SET NULL,
  user_id uuid REFERENCES users(id) ON DELETE SET NULL,
  method text NOT NULL,
  path text NOT NULL,
  status_code integer NOT NULL,
  target_type text,
  target_id text,
  remote_addr text,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS agent_api_audit_key_time_idx
  ON agent_api_audit_log (key_id, created_at DESC);
CREATE INDEX IF NOT EXISTS agent_api_audit_user_time_idx
  ON agent_api_audit_log (user_id, created_at DESC);
