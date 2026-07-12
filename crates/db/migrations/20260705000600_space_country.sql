ALTER TABLE spaces ADD COLUMN IF NOT EXISTS country text;
CREATE INDEX IF NOT EXISTS spaces_country_location_idx ON spaces (country, province, city, district);
