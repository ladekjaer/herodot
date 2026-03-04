use crate::authentication::api_key::ApiKey;
use crate::authentication::user::User;
use rerec::Reading;
use rerec::bme280::BME280;
use rerec::ds18b20::DS18B20;
use rerec::record::Record;
use sqlx::PgPool;
use sqlx::types::chrono;
use uuid::Uuid;

const DEFAULT_LIMIT: usize = 100;
const HARD_LIMIT: usize = 5000;

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

impl From<Ds18b20Record> for Record {
    fn from(value: Ds18b20Record) -> Self {
        Record::new(
            value.id,
            value.timestamp,
            Reading::DS18B20(DS18B20::new(value.device_name, value.raw_reading)),
        )
    }
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub(crate) struct Bme280Record {
    pub id: Uuid,
    pub temperature: f32,
    pub pressure: f32,
    pub humidity: f32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl From<Bme280Record> for Record {
    fn from(value: Bme280Record) -> Self {
        Record::new(
            value.id,
            value.timestamp,
            Reading::BME280(BME280::new(
                value.temperature,
                value.pressure,
                value.humidity,
            )),
        )
    }
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

    pub(crate) async fn get_record_by_id(
        &self,
        record_id: Uuid,
    ) -> Result<Option<Record>, sqlx::Error> {
        let bme280 = self.get_bme280_record_by_id(record_id).await?;
        if let Some(bme280) = bme280 {
            return Ok(Some(Record::from(bme280)));
        }

        let ds18b20 = self.get_ds18b20_record_by_id(record_id).await?;
        if let Some(ds18b20) = ds18b20 {
            return Ok(Some(Record::from(ds18b20)));
        }

        Ok(None)
    }

    pub(crate) async fn get_records(&self) -> Result<Vec<Record>, sqlx::Error> {
        let mut records: Vec<Record> = Vec::new();
        let bme280_records = self.get_all_bme280_records().await?;
        let ds18b20_records = self.get_all_ds18b20_records().await?;

        records.extend(bme280_records.into_iter().map(|r| {
            let bme280_reading = BME280::new(r.temperature, r.pressure, r.humidity);
            let reading = Reading::BME280(bme280_reading);
            Record::new(r.id, r.timestamp, reading)
        }));

        records.extend(ds18b20_records.into_iter().map(|r| {
            let ds18b20_reading = DS18B20::new(r.device_name, r.raw_reading);
            let reading = Reading::DS18B20(ds18b20_reading);
            Record::new(r.id, r.timestamp, reading)
        }));

        Ok(records)
    }

    pub(crate) async fn get_bme280_record_by_id(
        &self,
        record_id: Uuid,
    ) -> Result<Option<Bme280Record>, sqlx::Error> {
        let record = sqlx::query_as::<_, Bme280Record>(r#"SELECT id, temperature, pressure, humidity, timestamp FROM records.bme280 WHERE id = $1"#)
            .bind(record_id)
            .fetch_optional(&self.db_pool)
            .await?;

        Ok(record)
    }

    pub(crate) async fn get_all_bme280_records(&self) -> Result<Vec<Bme280Record>, sqlx::Error> {
        let records = sqlx::query_as::<_, Bme280Record>(
            r#"SELECT id, temperature, pressure, humidity, timestamp FROM records.bme280"#,
        )
        .fetch_all(&self.db_pool)
        .await?;
        Ok(records)
    }

    pub(crate) async fn get_ds18b20_record_by_id(
        &self,
        record_id: Uuid,
    ) -> Result<Option<Ds18b20Record>, sqlx::Error> {
        let record = sqlx::query_as::<_, Ds18b20Record>(
            r#"SELECT id, device_name, raw_reading, timestamp FROM records.ds18b20 WHERE id = $1"#,
        )
        .bind(record_id)
        .fetch_optional(&self.db_pool)
        .await?;

        Ok(record)
    }

    pub(crate) async fn get_all_ds18b20_records(&self) -> Result<Vec<Ds18b20Record>, sqlx::Error> {
        let records = sqlx::query_as::<_, Ds18b20Record>(
            r#"SELECT id, device_name, raw_reading, timestamp FROM records.ds18b20"#,
        )
        .fetch_all(&self.db_pool)
        .await?;
        Ok(records)
    }

    pub(crate) async fn get_user_by_username(&self, username: &str) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(
            r#"SELECT id, username, password FROM auth.users WHERE username = $1"#,
        )
        .bind(username)
        .fetch_one(&self.db_pool)
        .await?;

        Ok(user)
    }

    pub(crate) async fn create_user(
        &self,
        username: &str,
        password: &str,
    ) -> Result<User, sqlx::Error> {
        let user = User::new(username.into(), password.into()).unwrap();
        sqlx::query(r#"INSERT INTO auth.users (id, username, password) VALUES ($1, $2, $3);"#)
            .bind(user.id())
            .bind(user.username())
            .bind(user.hashed_password())
            .execute(&self.db_pool)
            .await?;

        Ok(user)
    }

    pub(crate) async fn create_api_key(&self, api_key: &ApiKey) -> Result<(), sqlx::Error> {
        let owner = self.get_user_by_username(api_key.owner()).await?;
        sqlx::query(
            r#"INSERT INTO auth.api_keys (id, name, owner_id, token) VALUES ($1, $2, $3, $4);"#,
        )
        .bind(api_key.id())
        .bind(api_key.name())
        .bind(owner.id())
        .bind(api_key.token())
        .execute(&self.db_pool)
        .await?;
        Ok(())
    }

    pub(crate) async fn get_api_key_by_token(
        &self,
        token_value: String,
    ) -> Result<ApiKey, sqlx::Error> {
        let token = sqlx::query_as::<_, ApiKey>(
            r#"
SELECT
    keys.id, keys.name, users.username as owner, token
FROM
    auth.api_keys keys
    JOIN auth.users users ON keys.owner_id = users.id
WHERE
    token = $1;
"#,
        )
        .bind(token_value)
        .fetch_one(&self.db_pool)
        .await?;
        Ok(token)
    }

    pub(crate) async fn list_api_keys(&self) -> Result<Vec<ApiKey>, sqlx::Error> {
        let records = sqlx::query_as::<_, ApiKey>(
            r#"
SELECT
    keys.id, name, username as owner, token
FROM
    auth.api_keys keys
    JOIN auth.users users ON keys.owner_id = users.id;
            "#,
        )
        .fetch_all(&self.db_pool)
        .await?;
        Ok(records)
    }

    fn limit(max_length: Option<usize>) -> usize {
        match max_length {
            None => DEFAULT_LIMIT,
            Some(max_length) => std::cmp::min(max_length, HARD_LIMIT),
        }
    }
}
