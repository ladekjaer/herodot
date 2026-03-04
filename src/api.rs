use crate::state::AppState;
use axum::extract::{FromRequestParts, Path, Query, State};
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, put};
use axum::{Json, Router};
use rerec::record::Record;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::error::ErrorKind;
use sqlx::Error;
use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

pub(crate) fn api() -> Router<AppState> {
    Router::new()
        .route("/records", get(get_records_by_filter))
        .route("/records", put(put_record))
        .route("/records/bme280", get(get_bme280))
        .route("/records/ds18b20", get(get_ds18b20))
        .route("/records/{record_id}", get(get_record_by_id))
}

async fn get_record_by_id(auth_token: AuthTokenValue, State(state): State<AppState>, Path(record_id): Path<Uuid>) -> impl IntoResponse {
    if let Err(error) = auth_token.validate(&state).await {
        return error;
    }

    match state.repository.get_record_by_id(record_id).await {
        Ok(record) => {
            match record {
                None => {(StatusCode::NOT_FOUND, Json(json!({"error": "record not found", "message": "No record with the supplied id was found"})))}
                Some(record) => {(StatusCode::OK, Json(json!({"record": record})))}
            }
        }
        Err(error) => {
            eprintln!("Error getting record: {}", error);
            let response_message = json!({"error": "database error", "message": "retrieval failed"});
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response_message))
        }
    }
}

async fn get_records_by_filter(auth_token: AuthTokenValue, Query(filter): Query<RecordFilter>, State(state): State<AppState>) -> impl IntoResponse {
    if let Err(error) = auth_token.validate(&state).await {
        return error;
    }

    match state.repository.get_record_by_filter(filter).await {
        Ok(records) => {
            let response_message = json!({"records": records});
            (StatusCode::OK, Json(response_message))
        }
        Err(error) => {
            eprintln!("Error getting records: {}", error);
            let response_message = json!({"error": "database error", "message": "retrieval failed"});
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response_message))
        }
    }
}

async fn get_bme280(auth_token: AuthTokenValue, Query(filter): Query<RecordFilter>, State(state): State<AppState>) -> impl IntoResponse {
    if let Err(error) = auth_token.validate(&state).await {
        return error;
    }

    match state.repository.get_bme280_by_filter(filter).await {
        Ok(records) => {
            let response_message = json!({"records": records});
            (StatusCode::OK, Json(response_message))
        }
        Err(error) => {
            eprintln!("Error getting records: {}", error);
            let response_message = json!({"error": "database error", "message": "retrieval failed"});
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response_message))
        }
    }
}

async fn get_ds18b20(auth_token: AuthTokenValue, Query(filter): Query<RecordFilter>, State(state): State<AppState>) -> impl IntoResponse {
    if let Err(error) = auth_token.validate(&state).await {
        return error;
    }

    match state.repository.get_ds18b20_by_filter(filter).await {
        Ok(records) => {
            let response_message = json!({"records": records});
            (StatusCode::OK, Json(response_message))
        }
        Err(error) => {
            eprintln!("Error getting records: {}", error);
            let response_message = json!({"error": "database error", "message": "retrieval failed"});
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response_message))
        }
    }
}

async fn put_record(
    auth_token: AuthTokenValue,
    State(state): State<AppState>,
    Json(record): Json<Record>,
) -> (StatusCode, Json<Value>) {
    if let Err(error) = auth_token.validate(&state).await {
        return error;
    }

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
            eprintln!("Error saving record: {:?}", error);
            match error {
                Error::Database(error) => {
                    match error.kind() {
                        ErrorKind::UniqueViolation => {
                            let response_message = json!({"error": "duplicate record", "message": "a record with an id of a record that already exists"});
                            (StatusCode::CONFLICT, Json(response_message))
                        }
                        ErrorKind::ForeignKeyViolation => {
                            let response_message = json!({"error": "invalid record", "message": "record references non-existing record"});
                            (StatusCode::BAD_REQUEST, Json(response_message))
                        }
                        ErrorKind::NotNullViolation => {
                            let response_message = json!({"error": "invalid record", "message": "record is missing required fields"});
                            (StatusCode::BAD_REQUEST, Json(response_message))
                        }
                        ErrorKind::CheckViolation => {
                            let response_message = json!({"error": "invalid record", "message": "record is invalid"});
                            (StatusCode::BAD_REQUEST, Json(response_message))
                        }
                        ErrorKind::Other => {
                            let response_message = json!({"error": "database error", "message": "error saving record"});
                            (StatusCode::INTERNAL_SERVER_ERROR, Json(response_message))
                        }
                        _ => {
                            let response_message = json!({"error": "database error", "message": "unable to save record"});
                            (StatusCode::INTERNAL_SERVER_ERROR, Json(response_message))
                        }
                    }
                }
                _ => {
                    let response_message = json!({"error": "database error", "message": error.to_string()});
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(response_message))
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AuthTokenValue {
    value: String,
}

impl AuthTokenValue {
    pub fn new(token: String) -> Self {
        Self { value: token }
    }

    pub async fn validate(&self, state: &AppState) -> Result<(), (StatusCode, Json<Value>)> {
        if let Err(error) = state.repository.get_api_key_by_token(self.value.clone()).await {
            eprintln!("Error getting API key: {}", error);
            let response_message =
                json!({"error": "invalid token", "message": "token not found"});
            return Err((StatusCode::UNAUTHORIZED, Json(response_message)));
        }
        Ok(())
    }
}

impl<S: Send + Sync> FromRequestParts<S> for AuthTokenValue {
    type Rejection = (StatusCode, Json<Value>);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts.headers.remove(axum::http::header::AUTHORIZATION);
        if let Some(key) = parts.headers.get("access_key") {
            let auth_token = AuthTokenValue::new(key.to_str().unwrap().to_string());
            Ok(auth_token)
        } else {
            let response_message = json!({"message": "no authorization header"});
            Err((StatusCode::UNAUTHORIZED, Json(response_message)))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RecordFilter {
    pub from: Option<DateTime<Utc>>,
    pub limit: Option<u32>,
}
