use std::fmt;
use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DS18B20 {
    device_name: String,
    raw_reading: i32
}

impl DS18B20 {
    pub fn new(device_name: String, raw_reading: i32) -> Self {
        Self { device_name, raw_reading }
    }

    pub fn device_name(&self) -> &str {
        &self.device_name
    }

    pub fn raw_reading(&self) -> i32 {
        self.raw_reading
    }

    pub fn temperature(&self) -> f32 {
        self.raw_reading as f32 / 1000.0
    }
}

impl Display for DS18B20 {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let name = self.device_name();
        let temperature = self.temperature();
        write!(f, "{}: {} °C", name, temperature)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reading() {
        let _reading = DS18B20 { device_name: "28-000000000000".into(), raw_reading: 22625 };
    }

    #[test]
    fn test_new() {
        let _reading = DS18B20::new("28-000000000000".into(), 22625);
    }

    #[test]
    fn test_debug() {
        let reading = DS18B20::new("28-000000000000".into(), 22625);
        let debug_output = format!("{:?}", reading);
        assert!(debug_output.contains("DS18B20"));
        assert!(debug_output.contains("28-000000000000"));
        assert!(debug_output.contains("22625"));
    }

    #[test]
    fn test_partial_eq() {
        let reading1 = DS18B20::new("28-000000000000".into(), 22625);
        let reading1_again = DS18B20::new("28-000000000000".into(), 22625);
        let reading2 = DS18B20::new("28-000000000000".into(), 23000);

        assert_eq!(reading1, reading1_again);
        assert_ne!(reading1, reading2);
    }

    #[test]
    fn test_get_temperature() {
        let reading = DS18B20::new("28-000000000000".into(), 22625);
        assert_eq!(reading.temperature(), 22.625);

        let reading = DS18B20::new("28-000000000000".into(), -22625);
        assert_eq!(reading.temperature(), -22.625);
    }

    #[test]
    fn test_get_device_name() {
        let reading = DS18B20::new("28-000000000000".into(), 22625);
        let actual = reading.device_name();
        assert_eq!(actual, "28-000000000000");
    }

    #[test]
    fn test_display() {
        let reading = DS18B20::new("28-000000000000".into(), 22625);
        assert_eq!(format!("{}", reading), "28-000000000000: 22.625 °C");

        let reading = DS18B20::new("28-000000000000".into(), -22625);
        assert_eq!(format!("{}", reading), "28-000000000000: -22.625 °C");
    }

    #[test]
    fn test_get_raw_reading() {
        let reading = DS18B20::new("28-000000000000".into(), 22625);
        assert_eq!(reading.raw_reading(), 22625);
    }
}