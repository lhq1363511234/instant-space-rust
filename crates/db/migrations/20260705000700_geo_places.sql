CREATE TABLE IF NOT EXISTS geo_places (
  id bigserial PRIMARY KEY,
  geoname_id bigint UNIQUE,
  country_code text NOT NULL,
  country_name text NOT NULL,
  admin1_code text,
  admin1_name text,
  admin2_code text,
  admin2_name text,
  admin3_code text,
  admin3_name text,
  place_name text NOT NULL,
  feature_class text,
  feature_code text,
  lat double precision NOT NULL,
  lng double precision NOT NULL,
  population bigint NOT NULL DEFAULT 0,
  timezone text,
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS geo_places_country_idx ON geo_places (country_name, country_code);
CREATE INDEX IF NOT EXISTS geo_places_admin_idx ON geo_places (country_name, admin1_name, place_name, admin2_name);
CREATE INDEX IF NOT EXISTS geo_places_lat_lng_idx ON geo_places (lat, lng);
CREATE INDEX IF NOT EXISTS geo_places_population_idx ON geo_places (population DESC);

ALTER TABLE spaces ADD COLUMN IF NOT EXISTS spot_name text;
ALTER TABLE spaces ADD COLUMN IF NOT EXISTS address_line text;
CREATE INDEX IF NOT EXISTS spaces_spot_location_idx ON spaces (country, province, city, district, spot_name);

GRANT SELECT ON geo_places TO instant_space;
GRANT SELECT, INSERT, UPDATE ON spaces TO instant_space;
