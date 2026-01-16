pub mod bme280;
pub mod ds18b20;

use crate::reading::bme280::BME280;
use crate::reading::ds18b20::DS18B20;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Reading {
    BME280(BME280),
    DS18B20(DS18B20),
}
