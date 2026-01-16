use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BME280 {
    // temperature in degrees Celsius
    temperature: f32,

    // Pressure in Pascals (Pa)
    pressure: f32,

    // Relative humidity in percent (%)
    humidity: f32,
}

impl BME280 {
    pub fn new(temperature: f32, pressure: f32, humidity: f32) -> Self {
        Self { temperature, pressure, humidity }
    }

    pub fn temperature(&self) -> f32 {
        self.temperature
    }

    pub fn pressure(&self) -> f32 {
        self.pressure
    }

    pub fn humidity(&self) -> f32 {
        self.humidity
    }
}

impl Display for BME280 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2} °C, {:.2} Pa, {:.2}%", self.temperature, self.pressure, self.humidity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reading() {
        let _reading = BME280 { temperature: 22.625, pressure: 101325.0, humidity: 35.0 };
    }

    #[test]
    fn test_new() {
        let _reading = BME280::new( 22.625, 101325.0, 35.0);
    }

    #[test]
    fn test_get_temperature() {
        let reading = BME280::new(22.625, 101325.0, 35.0);
        assert_eq!(reading.temperature(), 22.625);
    }

    #[test]
    fn test_get_pressure() {
        let reading = BME280::new(22.625, 101325.0, 35.0);
        assert_eq!(reading.pressure(), 101325.0);
    }

    #[test]
    fn test_get_humidity() {
        let reading = BME280::new(22.625, 101325.0, 35.0);
        assert_eq!(reading.humidity(), 35.0);
    }

    #[test]
    fn test_display() {
        let reading = BME280::new(22.625, 101325.0, 35.0);
        assert_eq!(format!("{}", reading), "22.62 °C, 101325.00 Pa, 35.00%");
    }
}
