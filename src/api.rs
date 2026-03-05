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

    let record = state.repository.get_record_by_id(record_id).await?;

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

    let records = state.repository.get_record_by_filter(filter).await?;

    Ok((StatusCode::OK, Json(json!({"records": records}))))
}

async fn get_bme280(
    auth_token: AuthTokenValue,
    Query(filter): Query<RecordFilter>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    auth_token.validate(&state).await?;

    let records = state.repository.get_bme280_by_filter(filter).await?;

    Ok((StatusCode::OK, Json(json!({"records": records}))))
}

async fn get_ds18b20(
    auth_token: AuthTokenValue,
    Query(filter): Query<RecordFilter>,
    State(state): State<AppState>
) -> Result<impl IntoResponse, AppError> {
    auth_token.validate(&state).await?;

    let records = state.repository.get_ds18b20_by_filter(filter).await?;

    Ok((StatusCode::OK, Json(json!({"records": records}))))
}

async fn put_record(
    auth_token: AuthTokenValue,
    State(state): State<AppState>,
    Json(record): Json<Record>,
) -> Result<impl IntoResponse, AppError> {
    auth_token.validate(&state).await?;

    let record_id = state
        .repository
        .commit_record(record)
        .await
        .map_err(AppError::from_commit_record_error)?;

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
        let value = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .ok_or(AppError::Unauthorized(
                "missing Authorization header (expected: Authorization: Bearer <token>)"
            ))?;

        let raw = value
            .to_str()
            .map_err(|_| AppError::BadRequest("API key token must be valid ASCII"))?;

        let (scheme, token) = raw
            .split_once(' ')
            .ok_or(AppError::BadRequest(
                "Authorization header must be in the format 'Bearer <token>'"
            ))?;

        if !scheme.eq_ignore_ascii_case("bearer") {
            return Err(AppError::BadRequest(
                "Authorization scheme must be 'Bearer'"
            ));
        }

        let token = token.trim();
        if token.is_empty() {
            return Err(AppError::BadRequest(
                "Bearer token must not be empty"
            ));
        }

        Ok(AuthTokenValue::new(token.to_string()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RecordFilter {
    pub from: Option<DateTime<Utc>>,
    pub limit: Option<u32>,
}
