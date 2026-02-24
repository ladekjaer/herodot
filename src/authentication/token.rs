use std::fmt::Display;
use base64::Engine;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Token {
    value: String,
}

impl Token {
    pub fn new() -> Self {
        let mut rng = rand::rng();
        let mut bytes = [0u8; 32];
        rng.fill_bytes(&mut bytes);

        let base64_encoded = BASE64_URL_SAFE_NO_PAD.encode(&bytes);

        Self { value: base64_encoded }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl From<String> for Token {
    fn from(value: String) -> Self {
        Self { value }
    }
}

impl Into<String> for Token {
    fn into(self) -> String {
        self.value
    }
}
