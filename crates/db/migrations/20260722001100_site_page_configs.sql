-- Controlled site-page configuration. Editors write validated JSON only;
-- arbitrary scripts or HTML are never stored or rendered.
CREATE TABLE site_page_configs (
  page_key text PRIMARY KEY,
  draft_config jsonb NOT NULL,
  published_config jsonb NOT NULL,
  published_version integer NOT NULL DEFAULT 0,
  updated_by uuid,
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE site_page_versions (
  id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  page_key text NOT NULL REFERENCES site_page_configs(page_key) ON DELETE CASCADE,
  version integer NOT NULL,
  config jsonb NOT NULL,
  actor_id uuid,
  actor_email text,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (page_key, version)
);

CREATE INDEX site_page_versions_page_created_idx
  ON site_page_versions (page_key, created_at DESC);
