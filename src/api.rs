use crate::record::Record;
use crate::state::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::put;
use axum::{Json, Router};
use serde_json::{json, Value};

pub(crate) fn api() -> Router<AppState> {
    Router::new()
        .route("/record", put(put_record))
}

async fn put_record(
    State(state): State<AppState>,
    Json(record): Json<Record>,
) -> (StatusCode, Json<Value>) {
    println!("RECORD PUT request: {:?}", record);

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
