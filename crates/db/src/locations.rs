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
