use crate::user::User;
use rerec::record::Record;
use rerec::Reading;
use sqlx::types::chrono;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub(crate) struct Repository {
    db_pool: PgPool,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub(crate) struct Ds18b20Record {
    pub id: Uuid,
    pub device_name: String,
    pub raw_reading: i32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
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

    pub(crate) async fn get_all_ds18b20_records(&self) -> Result<Vec<Ds18b20Record>, sqlx::Error> {
        let records = sqlx::query_as::<_, Ds18b20Record>(r#"SELECT id, device_name, raw_reading, timestamp FROM records.ds18b20"#).fetch_all(&self.db_pool).await?;
        Ok(records)
    }

    pub(crate) async fn get_user_by_username(&self, username: &str) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(r#"SELECT id, username, password FROM auth.users WHERE username = $1"#)
            .bind(username)
            .fetch_one(&self.db_pool)
            .await?;

        Ok(user)
    }

    pub(crate) async fn create_user(&self, username: &str, password: &str) -> Result<User, sqlx::Error> {
        let user = User::new(username.into(), password.into()).unwrap();
        sqlx::query(r#"INSERT INTO auth.users (id, username, password) VALUES ($1, $2, $3);"#)
            .bind(user.id())
            .bind(user.username())
            .bind(user.hashed_password())
            .execute(&self.db_pool)
            .await?;

        Ok(user)
    }
}
