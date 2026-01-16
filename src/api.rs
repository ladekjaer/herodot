use axum::{Json, Router};
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::Error;
use uuid::Uuid;
use crate::state::AppState;

pub(crate) fn api() -> Router<AppState> {
    Router::new()
        .route("/record", post(post_ds18b20_record))
}

async fn post_ds18b20_record(State(state): State<AppState>, Json(record): Json<DS18B20Record>) -> (StatusCode, Json<Value>) {
    println!("Received: {:?}", record);

    match state.repository.commit_record(record).await {
        Ok(record_id) => {
            let reply = json!({
                "message": "record saved successfully",
                "record_id": record_id
            });
            (StatusCode::CREATED, Json(reply))
        }
        Err(error) => {
            let response_message =
                json!({"error": "database error", "description": error.to_string()});
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response_message))
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DS18B20Record {
    id: Uuid,
    device_name: String,
    raw_reading: i32,
    timestamp: DateTime<Utc>
}

impl DS18B20Record {
    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn device_name(&self) -> &str {
        &self.device_name
    }

    pub fn raw_reading(&self) -> i32 {
        self.raw_reading
    }

    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}
