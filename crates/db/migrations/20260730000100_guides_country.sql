-- Guides need a first-class country so the directory can filter by nation the
-- same way the map/explore surfaces do. Every published guide is bound to a
-- space, and spaces already carry country, so we backfill from there and keep
-- the two in one vocabulary (normalise the stray Chinese "中国" to "China").
ALTER TABLE guides ADD COLUMN IF NOT EXISTS country text;

UPDATE guides g
SET country = s.country
FROM spaces s
WHERE g.space_id = s.id
  AND (g.country IS NULL OR g.country = '');

UPDATE guides SET country = 'China' WHERE country = '中国';

CREATE INDEX IF NOT EXISTS guides_country_idx ON guides (country);
