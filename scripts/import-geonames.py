#!/usr/bin/env python3
"""Import a practical global server-side geography database from GeoNames.

Downloads country, admin1, admin2 and cities5000 data, normalizes it into
geo_places, and loads it with psql COPY. Intended for production bootstrap; it is
safe to re-run because geoname_id is unique and upserted.
"""
from __future__ import annotations

import csv
import os
import subprocess
import tempfile
import urllib.request
import zipfile
from pathlib import Path

BASE = "https://download.geonames.org/export/dump"
DB_NAME = os.environ.get("GEONAMES_DB", "instant_space_rust")
PSQL_USER = os.environ.get("GEONAMES_PSQL_USER", "postgres")
MIN_POPULATION = int(os.environ.get("GEONAMES_MIN_POPULATION", "5000"))


def download_text(url: str) -> str:
    with urllib.request.urlopen(url, timeout=120) as response:
        return response.read().decode("utf-8", errors="replace")


def download_zip_member(url: str, member: str, dest: Path) -> Path:
    archive = dest / Path(url).name
    urllib.request.urlretrieve(url, archive)
    with zipfile.ZipFile(archive) as zf:
        zf.extract(member, dest)
    return dest / member


def load_countries() -> dict[str, str]:
    rows = {}
    for line in download_text(f"{BASE}/countryInfo.txt").splitlines():
        if not line or line.startswith("#"):
            continue
        parts = line.split("\t")
        if len(parts) > 4:
            rows[parts[0]] = parts[4]
    return rows


def load_admin(path: str) -> dict[str, str]:
    rows = {}
    for line in download_text(f"{BASE}/{path}").splitlines():
        if not line:
            continue
        parts = line.split("\t")
        if len(parts) >= 2:
            rows[parts[0]] = parts[1]
    return rows


def clean(value: str | None) -> str:
    return (value or "").replace("\t", " ").replace("\n", " ").strip()


def main() -> None:
    with tempfile.TemporaryDirectory(prefix="instant-geonames-") as tmp:
        tmpdir = Path(tmp)
        countries = load_countries()
        admin1 = load_admin("admin1CodesASCII.txt")
        admin2 = load_admin("admin2Codes.txt")
        cities_file = download_zip_member(f"{BASE}/cities5000.zip", "cities5000.txt", tmpdir)
        tsv = tmpdir / "geo_places.tsv"

        count = 0
        with cities_file.open("r", encoding="utf-8", errors="replace") as source, tsv.open(
            "w", encoding="utf-8", newline=""
        ) as out:
            writer = csv.writer(out, delimiter="\t", lineterminator="\n")
            for line in source:
                parts = line.rstrip("\n").split("\t")
                if len(parts) < 19:
                    continue
                geoname_id = parts[0]
                name = clean(parts[1])
                lat = parts[4]
                lng = parts[5]
                feature_class = parts[6]
                feature_code = parts[7]
                country_code = parts[8]
                admin1_code = parts[10]
                admin2_code = parts[11]
                admin3_code = parts[12]
                admin4_code = parts[13]
                population = int(parts[14] or 0)
                timezone = clean(parts[17])
                if population < MIN_POPULATION and feature_class != "P":
                    continue
                country_name = countries.get(country_code, country_code)
                admin1_key = f"{country_code}.{admin1_code}" if admin1_code else ""
                admin2_key = f"{country_code}.{admin1_code}.{admin2_code}" if admin2_code else ""
                writer.writerow(
                    [
                        geoname_id,
                        country_code,
                        clean(country_name),
                        admin1_code,
                        clean(admin1.get(admin1_key, admin1_code)),
                        admin2_code,
                        clean(admin2.get(admin2_key, admin2_code)),
                        admin3_code,
                        clean(admin4_code),
                        name,
                        feature_class,
                        feature_code,
                        lat,
                        lng,
                        population,
                        timezone,
                    ]
                )
                count += 1

        sql = f"""
CREATE TEMP TABLE geo_places_import (
  geoname_id bigint, country_code text, country_name text, admin1_code text, admin1_name text,
  admin2_code text, admin2_name text, admin3_code text, admin3_name text, place_name text,
  feature_class text, feature_code text, lat double precision, lng double precision,
  population bigint, timezone text
);
\\copy geo_places_import FROM '{tsv}' WITH (FORMAT csv, DELIMITER E'\\t')
INSERT INTO geo_places (
  geoname_id, country_code, country_name, admin1_code, admin1_name, admin2_code, admin2_name,
  admin3_code, admin3_name, place_name, feature_class, feature_code, lat, lng, population, timezone, updated_at
)
SELECT geoname_id, country_code, country_name, NULLIF(admin1_code, ''), NULLIF(admin1_name, ''),
       NULLIF(admin2_code, ''), NULLIF(admin2_name, ''), NULLIF(admin3_code, ''), NULLIF(admin3_name, ''),
       place_name, feature_class, feature_code, lat, lng, population, NULLIF(timezone, ''), now()
FROM geo_places_import
ON CONFLICT (geoname_id) DO UPDATE SET
  country_code = EXCLUDED.country_code,
  country_name = EXCLUDED.country_name,
  admin1_code = EXCLUDED.admin1_code,
  admin1_name = EXCLUDED.admin1_name,
  admin2_code = EXCLUDED.admin2_code,
  admin2_name = EXCLUDED.admin2_name,
  admin3_code = EXCLUDED.admin3_code,
  admin3_name = EXCLUDED.admin3_name,
  place_name = EXCLUDED.place_name,
  feature_class = EXCLUDED.feature_class,
  feature_code = EXCLUDED.feature_code,
  lat = EXCLUDED.lat,
  lng = EXCLUDED.lng,
  population = EXCLUDED.population,
  timezone = EXCLUDED.timezone,
  updated_at = now();
"""
        sql_file = tmpdir / "import.sql"
        sql_file.write_text(sql, encoding="utf-8")
        os.chmod(tmpdir, 0o755)
        os.chmod(tsv, 0o644)
        os.chmod(sql_file, 0o644)
        print(f"Importing {count} GeoNames rows into {DB_NAME}...")
        subprocess.run(["su", "-", PSQL_USER, "-c", f"psql -d {DB_NAME} -f {sql_file}"], check=True)


if __name__ == "__main__":
    main()
