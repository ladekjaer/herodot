use crate::state::AppState;
use crate::web::TERA;
use axum::extract::State;
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{Html, IntoResponse};
use axum::routing::post;
use axum::{Form, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::authentication::token::Token;
use crate::authentication::user_auth::AuthUser;

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_api_key))
}

async fn create_api_key(
    user: AuthUser,
    State(state): State<AppState>,
    Form(api_key_form_data): Form<ApiKeyFormData>
) -> impl IntoResponse {
    if user.username() != api_key_form_data.owner {
        eprintln!("REJECTED token creation attempt: owner does not match");
        return (StatusCode::FORBIDDEN, "Creation of keys owner by other users is not allowed.").into_response();
    }
    let key = ApiKey::from(api_key_form_data);
    match state.repository.create_api_key(&key).await {
        Ok(_) => {
            let mut context = tera::Context::new();
            context.insert("username", user.username());
            context.insert("key", &key);
            let output = TERA.render("api_key.html", &context).unwrap();

            let mut res = (StatusCode::CREATED, Html(output)).into_response();

            res.headers_mut().insert(
                header::CACHE_CONTROL,
                HeaderValue::from_static("no-store")
            );

            res
        }
        Err(error) => {
            eprintln!("Failed to create token: {}", error);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error: Could not create token!").into_response()
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
    token: String,
}

impl ApiKey {
    pub fn new(name: &str, owner: &str) -> Self {
        let id = Uuid::new_v4();
        let name = name.to_string();
        let owner = owner.to_string();
        let token = Token::new().to_string();
        Self { id, name, owner, token }
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

    pub fn token(&self) -> String {
        self.token.to_string()
    }
}