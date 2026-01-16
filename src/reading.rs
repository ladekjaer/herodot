use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Reading {
    DS18B20(DS18B20)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DS18B20 {
    device_name: String,
    raw_reading: i32
}

impl DS18B20 {
    pub fn device_name(&self) -> &str {
        &self.device_name
    }

    pub fn raw_reading(&self) -> i32 {
        self.raw_reading
    }
}
