use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::reading::Reading;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    id: Uuid,
    timestamp: DateTime<Utc>,
    reading: Reading
}

impl Record {
    pub fn new(id: Uuid, timestamp: DateTime<Utc>, reading: Reading) -> Self {
        Self { id, timestamp, reading }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
    
    pub fn reading(&self) -> &Reading {
        &self.reading
    }
}
