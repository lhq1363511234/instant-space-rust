-- Phase 4: guide content version history.
-- Every create/update writes a snapshot so hosts and admins can review and
-- restore previous versions instead of losing work on a bad edit.
CREATE TABLE IF NOT EXISTS guide_versions (
  id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  guide_id uuid NOT NULL REFERENCES guides(id) ON DELETE CASCADE,
  version_no integer NOT NULL,
  title_zh text NOT NULL,
  title_en text,
  summary_zh text,
  summary_en text,
  content_zh text,
  content_en text,
  sections jsonb NOT NULL DEFAULT '[]'::jsonb,
  images jsonb NOT NULL DEFAULT '[]'::jsonb,
  cover_image_url text,
  edited_by uuid REFERENCES users(id) ON DELETE SET NULL,
  edited_by_name text,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (guide_id, version_no)
);

CREATE INDEX IF NOT EXISTS idx_guide_versions_guide
  ON guide_versions (guide_id, version_no DESC);
