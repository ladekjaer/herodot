use serde::{Deserialize, Serialize};
use rerec::record::Record;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordDisplay {
    pub id: String,
    pub timestamp: String,
    pub reading: String,
}

impl From<Record> for RecordDisplay {
    fn from(value: Record) -> Self {
        Self {
            id: value.id().to_string(),
            timestamp: value.timestamp().to_string(),
            reading: value.reading().to_string(),
        }
    }
}