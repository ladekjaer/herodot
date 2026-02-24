use axum::response::IntoResponse;
use axum::{Form, Router};
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::state::AppState;
use crate::web::AuthUser;

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_api_key))
}

async fn create_api_key(user: AuthUser, State(state): State<AppState>, Form(api_key_form_data): Form<ApiKeyFormData>) -> impl IntoResponse {
    if user.username() != api_key_form_data.owner {
        eprintln!("REJECTED token creation attempt: owner does not match");
        return (StatusCode::FORBIDDEN, "Creation of keys owner by other users is not allowed.");
    }
    let key = ApiKey::from(api_key_form_data);
    match state.repository.create_api_key(key).await {
        Ok(_) => {
            (StatusCode::CREATED, "Token created successfully")
        }
        Err(error) => {
            eprintln!("Failed to create token: {}", error);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error: Could not create token!")
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiKeyFormData {
    name: String,
    owner: String,
}

impl From<ApiKeyFormData> for ApiKey {
    fn from(form_data: ApiKeyFormData) -> Self {
        ApiKey::new(&form_data.name, &form_data.owner)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub(crate) struct ApiKey {
    id: Uuid,
    name: String,
    owner: String,
    value: String,
}

impl ApiKey {
    pub fn new(name: &str, owner: &str) -> Self {
        let id = Uuid::new_v4();
        let name = name.to_string();
        let owner = owner.to_string();
        let value = Uuid::new_v4().to_string(); // TODO
        Self { id, name, owner, value }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn owner(&self) -> &str {
        &self.owner
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}
