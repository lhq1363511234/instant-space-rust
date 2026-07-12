use instant_domain::locations::{GeoMatch, GeoOption};
use sqlx::{PgPool, Row};

const OPTION_LIMIT: i64 = 800;

pub async fn countries(pool: &PgPool) -> Result<Vec<GeoOption>, sqlx::Error> {
    // Taiwan is a province of China — never list it as a country.
    let rows = sqlx::query(
        r#"
        SELECT country_name AS value, country_name AS label, max(population) AS rank
        FROM geo_places
        WHERE country_name IS NOT NULL
          AND btrim(country_name) <> ''
          AND country_name NOT ILIKE 'Taiwan'
          AND country_code <> 'TW'
        GROUP BY country_name
        ORDER BY rank DESC NULLS LAST, country_name
        LIMIT $1
        "#,
    )
    .bind(OPTION_LIMIT)
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(row_to_option).collect()
}

pub async fn regions(pool: &PgPool, country: String) -> Result<Vec<GeoOption>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT admin1_name AS value, admin1_name AS label, max(population) AS rank
        FROM geo_places
        WHERE (country_name = $1 OR country_name ILIKE $1) AND admin1_name IS NOT NULL AND admin1_name <> ''
        GROUP BY admin1_name
        ORDER BY rank DESC NULLS LAST, admin1_name
        LIMIT $2
        "#,
    )
    .bind(country)
    .bind(OPTION_LIMIT)
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(row_to_option).collect()
}

pub async fn cities(
    pool: &PgPool,
    country: String,
    region: Option<String>,
) -> Result<Vec<GeoOption>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT place_name AS value, place_name AS label, max(population) AS rank
        FROM geo_places
        WHERE (country_name = $1 OR country_name ILIKE $1)
          AND ($2::text IS NULL OR admin1_name = $2 OR admin1_name ILIKE $2)
          AND place_name IS NOT NULL AND place_name <> ''
        GROUP BY place_name
        ORDER BY rank DESC NULLS LAST, place_name
        LIMIT $3
        "#,
    )
    .bind(country)
    .bind(clean_optional(region))
    .bind(OPTION_LIMIT)
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(row_to_option).collect()
}

pub async fn districts(
    pool: &PgPool,
    country: String,
    region: Option<String>,
    city: Option<String>,
) -> Result<Vec<GeoOption>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT admin2_name AS value, admin2_name AS label, max(population) AS rank
        FROM geo_places
        WHERE country_name = $1
          AND ($2::text IS NULL OR admin1_name = $2)
          AND ($3::text IS NULL OR place_name = $3)
          AND admin2_name IS NOT NULL AND admin2_name <> ''
        GROUP BY admin2_name
        ORDER BY rank DESC NULLS LAST, admin2_name
        LIMIT $4
        "#,
    )
    .bind(country)
    .bind(clean_optional(region))
    .bind(clean_optional(city))
    .bind(OPTION_LIMIT)
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(row_to_option).collect()
}

pub async fn nearest_place(
    pool: &PgPool,
    lat: f64,
    lng: f64,
) -> Result<Option<GeoMatch>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT country_name, admin1_name, place_name, admin2_name, lat, lng
        FROM geo_places
        WHERE lat BETWEEN $1 - 2.0 AND $1 + 2.0
          AND lng BETWEEN $2 - 2.0 AND $2 + 2.0
        ORDER BY ((lat - $1) * (lat - $1) + (lng - $2) * (lng - $2)) ASC,
                 population DESC
        LIMIT 1
        "#,
    )
    .bind(lat)
    .bind(lng)
    .fetch_optional(pool)
    .await?;

    row.map(row_to_match).transpose()
}

pub async fn place_center(
    pool: &PgPool,
    country: Option<String>,
    region: Option<String>,
    city: Option<String>,
) -> Result<Option<(f64, f64, f64)>, sqlx::Error> {
    // Returns (lng, lat, suggested_zoom)
    // Country-only: fly to national capital (PPLC), not geographic average.
    let country = clean_optional(country);
    let region = clean_optional(region);
    let city = clean_optional(city);

    if country.is_none() && region.is_none() && city.is_none() {
        return Ok(None);
    }

    if country.is_some() && region.is_none() && city.is_none() {
        if let Some(cap) = country_capital(pool, country.as_deref().unwrap_or("")).await? {
            return Ok(Some(cap));
        }
    }

    let row = sqlx::query(
        r#"
        SELECT lng::float8 AS lng, lat::float8 AS lat, population
        FROM geo_places
        WHERE ($1::text IS NULL OR country_name = $1 OR country_name ILIKE $1)
          AND ($2::text IS NULL OR admin1_name = $2 OR admin1_name ILIKE $2)
          AND ($3::text IS NULL OR place_name = $3 OR place_name ILIKE $3)
        ORDER BY
          CASE WHEN $3::text IS NOT NULL AND place_name ILIKE $3 THEN 0 ELSE 1 END,
          population DESC NULLS LAST
        LIMIT 1
        "#,
    )
    .bind(country.as_deref())
    .bind(region.as_deref())
    .bind(city.as_deref())
    .fetch_optional(pool)
    .await?;

    if let Some(row) = row {
        let lng: Option<f64> = row.try_get("lng")?;
        let lat: Option<f64> = row.try_get("lat")?;
        if let (Some(lng), Some(lat)) = (lng, lat) {
            let zoom = if city.is_some() {
                10.5
            } else if region.is_some() {
                7.5
            } else {
                5.8
            };
            return Ok(Some((lng, lat, zoom)));
        }
    }

    let row = sqlx::query(
        r#"
        SELECT avg(lng)::float8 AS lng, avg(lat)::float8 AS lat, count(*)::int AS n
        FROM geo_places
        WHERE ($1::text IS NULL OR country_name = $1 OR country_name ILIKE $1)
          AND ($2::text IS NULL OR admin1_name = $2 OR admin1_name ILIKE $2)
          AND ($3::text IS NULL OR place_name = $3 OR place_name ILIKE $3)
        "#,
    )
    .bind(country.as_deref())
    .bind(region.as_deref())
    .bind(city.as_deref())
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };
    let lng: Option<f64> = row.try_get("lng")?;
    let lat: Option<f64> = row.try_get("lat")?;
    let n: i32 = row.try_get("n").unwrap_or(0);
    if n <= 0 {
        return Ok(None);
    }
    let (Some(lng), Some(lat)) = (lng, lat) else {
        return Ok(None);
    };
    let zoom = if city.is_some() {
        10.0
    } else if region.is_some() {
        7.0
    } else if n > 200 {
        4.2
    } else if n > 50 {
        5.0
    } else {
        5.8
    };
    Ok(Some((lng, lat, zoom)))
}

/// National capital center.
/// Priority: geo_capitals table → GeoNames PPLC → largest city.
pub async fn country_capital(
    pool: &PgPool,
    country: &str,
) -> Result<Option<(f64, f64, f64)>, sqlx::Error> {
    let country = country.trim();
    if country.is_empty() {
        return Ok(None);
    }
    // Taiwan is China province, not a country capital target.
    let country_key = if country.eq_ignore_ascii_case("taiwan") || country == "台湾" || country == "台灣" {
        "China"
    } else {
        country
    };

    // 1) Dedicated capitals table (all countries seeded)
    let row = sqlx::query(
        r#"
        SELECT lng::float8 AS lng, lat::float8 AS lat, zoom::float8 AS zoom, capital_name
        FROM geo_capitals
        WHERE country_name = $1 OR country_name ILIKE $1
        LIMIT 1
        "#,
    )
    .bind(country_key)
    .fetch_optional(pool)
    .await?;

    if let Some(row) = row {
        let lng: Option<f64> = row.try_get("lng")?;
        let lat: Option<f64> = row.try_get("lat")?;
        let zoom: Option<f64> = row.try_get("zoom")?;
        if let (Some(lng), Some(lat)) = (lng, lat) {
            return Ok(Some((lng, lat, zoom.unwrap_or(5.8))));
        }
    }

    // 2) GeoNames national capital marker
    let row = sqlx::query(
        r#"
        SELECT lng::float8 AS lng, lat::float8 AS lat, place_name
        FROM geo_places
        WHERE (country_name = $1 OR country_name ILIKE $1)
          AND feature_code = 'PPLC'
        ORDER BY population DESC NULLS LAST
        LIMIT 1
        "#,
    )
    .bind(country_key)
    .fetch_optional(pool)
    .await?;

    if let Some(row) = row {
        let lng: Option<f64> = row.try_get("lng")?;
        let lat: Option<f64> = row.try_get("lat")?;
        if let (Some(lng), Some(lat)) = (lng, lat) {
            return Ok(Some((lng, lat, 5.8)));
        }
    }

    // 3) Largest city fallback
    let row = sqlx::query(
        r#"
        SELECT lng::float8 AS lng, lat::float8 AS lat
        FROM geo_places
        WHERE country_name = $1 OR country_name ILIKE $1
        ORDER BY population DESC NULLS LAST
        LIMIT 1
        "#,
    )
    .bind(country_key)
    .fetch_optional(pool)
    .await?;

    if let Some(row) = row {
        let lng: Option<f64> = row.try_get("lng")?;
        let lat: Option<f64> = row.try_get("lat")?;
        if let (Some(lng), Some(lat)) = (lng, lat) {
            return Ok(Some((lng, lat, 5.5)));
        }
    }
    Ok(None)
}

/// All stored national capitals for client-side accurate fly-to.
pub async fn list_capitals(pool: &PgPool) -> Result<Vec<(String, String, f64, f64, f64)>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT country_name, capital_name, lng::float8 AS lng, lat::float8 AS lat, zoom::float8 AS zoom
        FROM geo_capitals
        WHERE country_name NOT IN ('Taiwan')
        ORDER BY country_name
        "#,
    )
    .fetch_all(pool)
    .await?;

    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let country: String = row.try_get("country_name")?;
        let capital: String = row.try_get("capital_name")?;
        let lng: f64 = row.try_get("lng")?;
        let lat: f64 = row.try_get("lat")?;
        let zoom: f64 = row.try_get("zoom").unwrap_or(5.8);
        out.push((country, capital, lng, lat, zoom));
    }
    Ok(out)
}


fn clean_optional(value: Option<String>) -> Option<String> {
    value.and_then(|v| {
        let t = v.trim().to_string();
        if t.is_empty() {
            None
        } else {
            Some(t)
        }
    })
}

fn row_to_option(row: sqlx::postgres::PgRow) -> Result<GeoOption, sqlx::Error> {
    let value: String = row.try_get("value")?;
    let label: String = row.try_get("label")?;
    Ok(GeoOption { value, label })
}

fn row_to_match(row: sqlx::postgres::PgRow) -> Result<GeoMatch, sqlx::Error> {
    let country: String = row.try_get("country_name")?;
    let province: Option<String> = row.try_get("admin1_name")?;
    let city: Option<String> = row.try_get("place_name")?;
    let district: Option<String> = row.try_get("admin2_name")?;
    let lat: f64 = row.try_get("lat")?;
    let lng: f64 = row.try_get("lng")?;
    Ok(GeoMatch {
        country,
        province: clean_optional(province),
        city: clean_optional(city),
        district: clean_optional(district),
        spot_name: None,
        lat,
        lng,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn countries_query_runs() {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
        let pool = PgPool::connect(&database_url).await.expect("pool");
        let rows = countries(&pool).await.expect("countries");
        assert!(rows.is_empty() || rows.iter().any(|row| !row.value.is_empty()));
    }
}
