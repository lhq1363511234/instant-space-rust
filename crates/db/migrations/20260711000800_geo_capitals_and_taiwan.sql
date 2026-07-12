-- Capitals lookup table for accurate country fly-to.
CREATE TABLE IF NOT EXISTS geo_capitals (
  country_name text PRIMARY KEY,
  capital_name text NOT NULL,
  lat double precision NOT NULL,
  lng double precision NOT NULL,
  zoom double precision NOT NULL DEFAULT 5.8,
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS geo_capitals_name_idx ON geo_capitals (country_name);

-- Remap Taiwan places under China as a province (not a country).
UPDATE geo_places
SET feature_code = 'PPLA'
WHERE (country_code = 'TW' OR country_name ILIKE 'Taiwan')
  AND feature_code = 'PPLC';

UPDATE geo_places
SET
  country_name = 'China',
  country_code = 'CN',
  admin1_name = CASE
    WHEN admin1_name IS NULL OR btrim(admin1_name) = '' OR admin1_name ILIKE 'Taiwan' OR admin1_name ILIKE 'Takao' OR admin1_name ILIKE 'Fukien'
      THEN 'Taiwan'
    ELSE admin1_name
  END
WHERE country_code = 'TW' OR country_name ILIKE 'Taiwan';

-- Seed capitals from GeoNames national capitals (PPLC).
INSERT INTO geo_capitals (country_name, capital_name, lat, lng, zoom)
SELECT
  country_name,
  place_name,
  lat,
  lng,
  CASE
    WHEN population >= 5000000 THEN 5.6
    WHEN population >= 1000000 THEN 5.9
    ELSE 6.2
  END AS zoom
FROM (
  SELECT DISTINCT ON (country_name)
    country_name, place_name, lat, lng, population
  FROM geo_places
  WHERE feature_code = 'PPLC'
  ORDER BY country_name, population DESC NULLS LAST
) cap
ON CONFLICT (country_name) DO UPDATE
SET
  capital_name = EXCLUDED.capital_name,
  lat = EXCLUDED.lat,
  lng = EXCLUDED.lng,
  zoom = EXCLUDED.zoom,
  updated_at = now();

-- Ensure China capital is Beijing.
INSERT INTO geo_capitals (country_name, capital_name, lat, lng, zoom)
VALUES ('China', 'Beijing', 39.9075, 116.39723, 5.8)
ON CONFLICT (country_name) DO UPDATE
SET capital_name = 'Beijing', lat = 39.9075, lng = 116.39723, zoom = 5.8, updated_at = now();

-- Ensure United Kingdom capital is London (exact).
INSERT INTO geo_capitals (country_name, capital_name, lat, lng, zoom)
VALUES ('United Kingdom', 'London', 51.50853, -0.12574, 6.0)
ON CONFLICT (country_name) DO UPDATE
SET capital_name = 'London', lat = 51.50853, lng = -0.12574, zoom = 6.0, updated_at = now();

-- Common aliases for search/fly.
INSERT INTO geo_capitals (country_name, capital_name, lat, lng, zoom)
VALUES
  ('UK', 'London', 51.50853, -0.12574, 6.0),
  ('Great Britain', 'London', 51.50853, -0.12574, 6.0),
  ('Britain', 'London', 51.50853, -0.12574, 6.0),
  ('USA', 'Washington', 38.89511, -77.03637, 5.5),
  ('US', 'Washington', 38.89511, -77.03637, 5.5),
  ('United States of America', 'Washington', 38.89511, -77.03637, 5.5),
  ('中国', 'Beijing', 39.9075, 116.39723, 5.8),
  ('英国', 'London', 51.50853, -0.12574, 6.0),
  ('美国', 'Washington', 38.89511, -77.03637, 5.5),
  ('日本', 'Tokyo', 35.6895, 139.69171, 5.8),
  ('韩国', 'Seoul', 37.5665, 126.978, 6.0)
ON CONFLICT (country_name) DO UPDATE
SET capital_name = EXCLUDED.capital_name, lat = EXCLUDED.lat, lng = EXCLUDED.lng, zoom = EXCLUDED.zoom, updated_at = now();

GRANT SELECT ON geo_capitals TO instant_space;
