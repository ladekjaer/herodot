use rerec::Reading;
use rerec::record::Record;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub(crate) struct Repository {
    db_pool: PgPool,
}

impl Repository {
    pub(crate) fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    pub(crate) async fn commit_record(&self, record: Record) -> Result<Uuid, sqlx::Error> {
        let record_id = record.id();
        let timestamp = record.timestamp();
        let reading = record.reading();

        match reading {
            Reading::BME280(reading) => {
                let temperature = reading.temperature();
                let pressure = reading.pressure();
                let humidity = reading.humidity();

                sqlx::query(r#"INSERT INTO records.bme280 (id, temperature, pressure, humidity, timestamp) VALUES ($1, $2, $3, $4, $5)"#)
                    .bind(record_id)
                    .bind(temperature)
                    .bind(pressure)
                    .bind(humidity)
                    .bind(timestamp)
                    .execute(&self.db_pool)
                    .await?;

                Ok(record_id)
            }
            Reading::DS18B20(reading) => {
                let device_name = reading.device_name();
                let raw_reading = reading.raw_reading();

                sqlx::query(r#"INSERT INTO records.ds18b20 (id, device_name, raw_reading, timestamp) VALUES ($1, $2, $3, $4)"#)
                    .bind(record_id)
                    .bind(device_name)
                    .bind(raw_reading)
                    .bind(timestamp)
                    .execute(&self.db_pool)
                    .await?;

                Ok(record_id)
            }
        }
    }
}
