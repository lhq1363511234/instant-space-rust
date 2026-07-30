-- Homepage editorial curation. This is deliberately a human-set priority,
-- not a synthetic engagement score.
ALTER TABLE spaces
  ADD COLUMN IF NOT EXISTS home_weight integer NOT NULL DEFAULT 0
  CHECK (home_weight BETWEEN 0 AND 1000);

CREATE INDEX IF NOT EXISTS spaces_home_featured_idx
  ON spaces (home_weight DESC, online_count DESC, created_at DESC)
  WHERE status IN ('active', 'expired');
