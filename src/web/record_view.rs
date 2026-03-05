use rerec::Reading;
use serde::{Deserialize, Serialize};
use rerec::record::Record;
use sqlx::types::chrono;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordView {
    pub id: String,
    pub timestamp: String,
    pub reading: String,
}

impl From<Record> for RecordView {
    fn from(value: Record) -> Self {
        Self {
            id: value.id().to_string(),
            timestamp: value.timestamp().to_string(),
            reading: value.reading().to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct Bme280RecordView {
    id: Uuid,
    temperature: f32,
    pressure: f32,
    humidity: f32,
    timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct Ds18b20RecordView {
    id: Uuid,
    device_name: String,
    raw_reading: i32,
    timestamp: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<Record> for Bme280RecordView {
    type Error = ();

    fn try_from(value: Record) -> Result<Self, Self::Error> {
        let reading = value.reading().clone();
        match reading {
            Reading::BME280(reading) => {
                Ok(Bme280RecordView {
                    id: value.id(),
                    temperature: reading.temperature(),
                    pressure: reading.pressure(),
                    humidity: reading.humidity(),
                    timestamp: value.timestamp(),
                })
            }
            _ => Err(())
        }
    }
}

impl TryFrom<Record> for Ds18b20RecordView {
    type Error = ();

    fn try_from(value: Record) -> Result<Self, Self::Error> {
        match value.reading() {
            Reading::DS18B20(reading) => {
                Ok(Ds18b20RecordView {
                    id: value.id(),
                    device_name: reading.device_name().to_string(),
                    raw_reading: reading.raw_reading(),
                    timestamp: value.timestamp(),
                })
            }
            _ => Err(())
        }
    }
}
