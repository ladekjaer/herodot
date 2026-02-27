use crate::state::AppState;
use axum::extract::{FromRequestParts, State};
use axum::http::StatusCode;
use axum::routing::put;
use axum::{Json, Router};
use axum::http::request::Parts;
use rerec::record::Record;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub(crate) fn api() -> Router<AppState> {
    Router::new()
        .route("/record", put(put_record))
}

async fn put_record(
    auth_token: AuthTokenValue,
    State(state): State<AppState>,
    Json(record): Json<Record>,
) -> (StatusCode, Json<Value>) {
    match state.repository.get_api_key_by_token(auth_token.value).await {
        Ok(api_key) => {
            println!("API key found: {:?}", api_key);
        }
        Err(error) => {
            eprintln!("Error getting API key: {}", error);
            let response_message =
                json!({"error": "invalid token", "message": "token not found"});
            return (StatusCode::UNAUTHORIZED, Json(response_message));
        }
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
            let response_message =
                json!({"error": "database error", "message": error.to_string()});
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response_message))
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
}

impl<S: Send + Sync> FromRequestParts<S> for AuthTokenValue {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts.headers.remove(axum::http::header::AUTHORIZATION);
        if let Some(key) = parts.headers.get("access_key") {
            let auth_token = AuthTokenValue::new(key.to_str().unwrap().to_string());
            Ok(auth_token)
        } else {
            Err((StatusCode::UNAUTHORIZED, "no authorization header"))
        }
    }
}
