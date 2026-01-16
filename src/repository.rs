use sqlx::PgPool;
use uuid::Uuid;
use crate::api::DS18B20Record;

#[derive(Clone)]
pub(crate) struct Repository {
    db_pool: PgPool,
}

impl Repository {
    pub(crate) fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    pub(crate) async fn commit_record(&self, record: DS18B20Record) -> Result<Uuid, sqlx::Error> {
        let record_id = record.id();
        sqlx::query(r#"INSERT INTO records.ds18b20 (id, device_name, raw_reading, timestamp) VALUES ($1, $2, $3, $4)"#)
            .bind(record.id())
            .bind(record.device_name())
            .bind(record.raw_reading())
            .bind(record.timestamp())
            .execute(&self.db_pool)
            .await?;
        Ok(record_id)
    }
}
