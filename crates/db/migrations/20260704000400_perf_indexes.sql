CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE INDEX spaces_home_visible_created_idx
  ON spaces (status, created_at DESC)
  INCLUDE (space_type, is_public, online_count)
  WHERE status IN ('active', 'expired');

CREATE INDEX spaces_home_type_created_idx
  ON spaces (space_type, created_at DESC)
  WHERE status IN ('active', 'expired');

CREATE INDEX spaces_name_zh_trgm_idx
  ON spaces USING gin (name_zh gin_trgm_ops);

CREATE INDEX spaces_name_en_trgm_idx
  ON spaces USING gin (name_en gin_trgm_ops)
  WHERE name_en IS NOT NULL;

CREATE INDEX guides_published_province_featured_idx
  ON guides (province, featured DESC, created_at DESC)
  WHERE status = 'published';

CREATE INDEX guides_published_featured_idx
  ON guides (featured DESC, created_at DESC)
  WHERE status = 'published';

CREATE INDEX locations_province_source_idx
  ON locations (province, source);
