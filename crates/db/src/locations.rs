use instant_domain::locations::LocationNode;
use sqlx::{PgPool, Row};

pub async fn provinces(pool: &PgPool) -> Result<Vec<LocationNode>, sqlx::Error> {
    let rows = sqlx::query("SELECT DISTINCT province FROM locations ORDER BY province")
        .fetch_all(pool)
        .await?;

    rows.into_iter()
        .map(|row| {
            Ok(LocationNode {
                province: row.try_get("province")?,
                city: None,
                district: None,
                spot_name: None,
            })
        })
        .collect()
}

pub async fn cities(pool: &PgPool, province: String) -> Result<Vec<String>, sqlx::Error> {
    distinct_location_values(
        pool,
        "SELECT DISTINCT city AS value FROM locations WHERE province = $1 AND city IS NOT NULL ORDER BY city",
        vec![province],
    )
    .await
}

pub async fn districts(
    pool: &PgPool,
    province: String,
    city: String,
) -> Result<Vec<String>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT DISTINCT district AS value FROM locations WHERE province = $1 AND city = $2 AND district IS NOT NULL ORDER BY district",
    )
    .bind(province)
    .bind(city)
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(|row| row.try_get("value")).collect()
}

pub async fn spots(
    pool: &PgPool,
    province: String,
    city: String,
    district: Option<String>,
) -> Result<Vec<String>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT DISTINCT spot_name AS value
        FROM locations
        WHERE province = $1
          AND city = $2
          AND ($3::text IS NULL OR district = $3)
          AND spot_name IS NOT NULL
        ORDER BY spot_name
        "#,
    )
    .bind(province)
    .bind(city)
    .bind(district)
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(|row| row.try_get("value")).collect()
}

async fn distinct_location_values(
    pool: &PgPool,
    sql: &str,
    values: Vec<String>,
) -> Result<Vec<String>, sqlx::Error> {
    let mut query = sqlx::query(sql);
    for value in values {
        query = query.bind(value);
    }
    let rows = query.fetch_all(pool).await?;
    rows.into_iter().map(|row| row.try_get("value")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn guide_hierarchy_returns_seeded_values() {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
        let pool = sqlx::PgPool::connect(&database_url).await.expect("pool");

        let province_items = provinces(&pool).await.expect("provinces");
        assert!(province_items.iter().any(|item| item.province == "上海市"));

        let spot_items = spots(
            &pool,
            "上海市".to_string(),
            "上海市".to_string(),
            Some("黄浦区".to_string()),
        )
        .await
        .expect("spots");
        assert!(spot_items.iter().any(|spot| spot == "外滩"));
    }
}
