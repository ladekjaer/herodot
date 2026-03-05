use crate::error::AppError;
use crate::state::AppState;
use axum::extract::{FromRequestParts, Path, Query, State};
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, put};
use axum::{Json, Router};
use rerec::record::Record;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::error::ErrorKind;
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

async fn get_record_by_id(
    auth_token: AuthTokenValue,
    State(state): State<AppState>,
    Path(record_id): Path<Uuid>
) -> Result<impl IntoResponse, AppError> {
    auth_token.validate(&state).await?;

    let record = state
        .repository
        .get_record_by_id(record_id)
        .await.map_err(|error| {
            eprintln!("Error getting record: {}", error);
            AppError::InternalServerError("retrieval from database failed")
        })?;

    match record {
        Some(record) => Ok((StatusCode::OK, Json(json!({"record": record})))),
        None => Err(AppError::NotFound("No record with the supplied id was found"))
    }
}

async fn get_records_by_filter(
    auth_token: AuthTokenValue,
    Query(filter): Query<RecordFilter>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    auth_token.validate(&state).await?;

    let records = state
        .repository
        .get_record_by_filter(filter)
        .await
        .map_err(|error| {
            eprintln!("Error getting records: {}", error);
            AppError::InternalServerError("retrieval from database failed")
        })?;

    Ok((StatusCode::OK, Json(json!({"records": records}))))
}

async fn get_bme280(
    auth_token: AuthTokenValue,
    Query(filter): Query<RecordFilter>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    auth_token.validate(&state).await?;

    let records = state
        .repository
        .get_bme280_by_filter(filter)
        .await
        .map_err(|error| {
            eprintln!("Error getting records: {}", error);
            AppError::InternalServerError("retrieval from database failed")
        })?;

    Ok((StatusCode::OK, Json(json!({"records": records}))))
}

async fn get_ds18b20(
    auth_token: AuthTokenValue,
    Query(filter): Query<RecordFilter>,
    State(state): State<AppState>
) -> Result<impl IntoResponse, AppError> {
    auth_token.validate(&state).await?;

    let records = state
        .repository
        .get_ds18b20_by_filter(filter)
        .await
        .map_err(|error| {
            eprintln!("Error getting records: {}", error);
            AppError::InternalServerError("retrieval from database failed")
        })?;

    Ok((StatusCode::OK, Json(json!({"records": records}))))
}

async fn put_record(
    auth_token: AuthTokenValue,
    State(state): State<AppState>,
    Json(record): Json<Record>,
) -> Result<impl IntoResponse, AppError> {
    auth_token.validate(&state).await?;

    let record_id = match state.repository.commit_record(record).await {
        Ok(record_id) => record_id,
        Err(error) => {
            eprintln!("Error saving record: {:?}", error);

            return match error {
                sqlx::Error::Database(error) => match error.kind() {
                    ErrorKind::UniqueViolation => {
                        Ok((
                            StatusCode::CONFLICT,
                            Json(json!({"error": "duplicate record", "message": "a record with an id of a record that already exists"}))
                        ))
                    }
                    ErrorKind::ForeignKeyViolation => {
                        Ok((
                            StatusCode::BAD_REQUEST,
                            Json(json!({"error": "invalid record", "message": "record references non-existing record"}))
                        ))
                    }
                    ErrorKind::NotNullViolation => {
                        Ok((
                            StatusCode::BAD_REQUEST,
                            Json(json!({
                                "error": "invalid record",
                                "message": "record is missing required fields"
                            })),
                        ))
                    }
                    ErrorKind::CheckViolation => {
                        Ok((
                            StatusCode::BAD_REQUEST,
                            Json(json!({
                                "error": "invalid record",
                                "message": "record is invalid"
                            })),
                        ))
                    }
                    ErrorKind::Other => {
                        Err(AppError::InternalServerError("error saving record"))
                    }
                    _ => {
                        Err(AppError::InternalServerError("unable to save record"))
                    }
                }
                _ => Err(AppError::SqlxError(error)),
            }
        }
    };

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "message": "record saved successfully",
            "record_id": record_id
        }))
    ))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AuthTokenValue {
    value: String,
}

impl AuthTokenValue {
    pub fn new(token: String) -> Self {
        Self { value: token }
    }

    pub async fn validate(&self, state: &AppState) -> Result<(), AppError> {
        match state
            .repository
            .get_api_key_by_token(self.value.clone())
            .await
        {
            Ok(_) => Ok(()),
            Err(sqlx::Error::RowNotFound) => Err(AppError::Unauthorized("unknown API key token")),
            Err(error) => Err(AppError::SqlxError(error)),
        }
    }
}

impl<S: Send + Sync> FromRequestParts<S> for AuthTokenValue {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if let Some(key) = parts.headers.get("access_key") {
            let token = key
                .to_str()
                .map_err(|_| AppError::BadRequest("API key token must be valid ASCII"))?
                .to_string();

            if token.is_empty() {
                return Err(AppError::BadRequest("API key token must not be empty"));
            }

            Ok(AuthTokenValue::new(token))
        } else {
            Err(AppError::Unauthorized("no API key token provided"))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RecordFilter {
    pub from: Option<DateTime<Utc>>,
    pub limit: Option<u32>,
}
