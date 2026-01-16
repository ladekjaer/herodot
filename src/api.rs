use axum::{Json, Router};
use axum::http::StatusCode;
use axum::routing::post;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

pub(crate) fn api() -> Router {
    Router::new()
        .route("/record", post(post_ds18b20_record))
}

async fn post_ds18b20_record(Json(record): Json<DS18B20Record>) -> (StatusCode, Json<Value>) {
    println!("Received: {:?}", record);
    let reply = json!({
        "message": "record saved successfully",
        "record_id": record.id
    });
    (StatusCode::CREATED, Json(reply))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DS18B20Record {
    id: Uuid,
    device_name: String,
    raw_reading: i32,
    timestamp: DateTime<Utc>
}
