//! # SW Information Repository 구현

use crate::domain::sw_information::entities::SwInformation;
use crate::domain::sw_information::SwInformationRepository;
use async_trait::async_trait;
use sqlx::PgPool;

pub struct SwInformationRepositoryImpl {
    pool: PgPool,
}

impl SwInformationRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SwInformationRepository for SwInformationRepositoryImpl {
    async fn find_all(&self) -> Result<Vec<SwInformation>, sqlx::Error> {
        sqlx::query_as::<_, SwInformation>(
            r#"
            SELECT id, product_item, model_name, sw_version, manufacturer, address,
                   manufacturing_permit_number, manufacturing_year_month, serial_number, udi,
                   created_at, updated_at
            FROM sw_information
            ORDER BY id
            "#,
        )
        .fetch_all(&self.pool)
        .await
    }

    async fn find_by_id(&self, id: i32) -> Result<Option<SwInformation>, sqlx::Error> {
        sqlx::query_as::<_, SwInformation>(
            r#"
            SELECT id, product_item, model_name, sw_version, manufacturer, address,
                   manufacturing_permit_number, manufacturing_year_month, serial_number, udi,
                   created_at, updated_at
            FROM sw_information
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }
}
