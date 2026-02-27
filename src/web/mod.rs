use crate::state::AppState;
use axum::extract::{State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::{Router};
use lazy_static::lazy_static;
use rerec::record::Record;
use serde::{Deserialize, Serialize};
use crate::authentication::user_api;
use crate::authentication::user_auth::AuthUser;

lazy_static! {
    pub static ref Tera: tera::Tera = match tera::Tera::new("templates/**/*") {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Tera initialization error: {}", e);
            std::process::exit(1);
        }
    };
}

pub(crate) fn web() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .route("/me", get(me))
        .route("/login", get(login))
        .route("/register", get(register))
        .route("/api_keys", get(api_keys))
        .route("/records", get(records))
        .route("/bme280", get(bme280))
        .route("/ds18b20", get(ds18b20))
        .nest("/users",user_api::user_router())
}

async fn index(user: Result<AuthUser, impl IntoResponse>) -> impl IntoResponse {
    let mut context = tera::Context::new();
    if let Ok(user) = user {
        context.insert("username", user.username());
    }

    let output = Tera.render("index.html", &context).unwrap();
    Html(output).into_response()
}

async fn me(user: AuthUser) -> impl IntoResponse {
    let mut context = tera::Context::new();
    context.insert("username", user.username());
    context.insert("user_id", &user.id());

    let template = "me.html";

    match Tera.render(template, &context) {
        Ok(output) => {
            Html(output).into_response()
        },
        Err(error) => {
            eprintln!("Error rendering template ({}): {}", template, error);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn login() -> impl IntoResponse {
    let context = tera::Context::new();
    let output = Tera.render("login.html", &context).unwrap();
    Html(output)
}

async fn register() -> impl IntoResponse {
    let context = tera::Context::new();
    let output = Tera.render("register.html", &context).unwrap();
    Html(output)
}

async fn api_keys(user: AuthUser, State(state): State<AppState>) -> impl IntoResponse {
    let mut context = tera::Context::new();
    context.insert("username", user.username());

    let api_keys = state.repository.list_api_keys().await.unwrap();
    context.insert("api_keys", &api_keys);

    let output = Tera.render("api_keys.html", &context).unwrap();
    Html(output)
}

async fn records(user: AuthUser, State(state): State<AppState>) -> impl IntoResponse {
    let mut context = tera::Context::new();
    context.insert("username", user.username());

    let records = state.repository.get_records().await.unwrap();
    let records: Vec<RecordDisplay> = records.into_iter().map(|r| r.into()).collect();
    context.insert("records", &records);
    let output = Tera.render("records.html", &context).unwrap();
    Html(output).into_response()
}

async fn bme280(user: AuthUser, State(state): State<AppState>) -> impl IntoResponse {
    let mut context = tera::Context::new();
    context.insert("username", user.username());

    let records = state.repository.get_all_bme280_records().await.unwrap();
    context.insert("records", &records);
    let output = Tera.render("bme280.html", &context).unwrap();
    Html(output).into_response()
}

async fn ds18b20(user: AuthUser, State(state): State<AppState>) -> impl IntoResponse {
    let mut context = tera::Context::new();
    context.insert("username", user.username());

    let records = state.repository.get_all_ds18b20_records().await.unwrap();
    context.insert("records", &records);
    let output = Tera.render("ds18b20.html", &context).unwrap();
    Html(output).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_render_index() {
        let page = Tera.render("index.html", &tera::Context::new()).unwrap();
        assert!(page.contains("<h1>Herodot WebApp - Self-hosting climate date repository</h1>"));
    }

    #[tokio::test]
    async fn test_login() {
        let response = login().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_str.contains("<h1>Login</h1>"));
    }

    #[tokio::test]
    async fn test_register() {
        let response = register().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_str.contains("<h1>Create a new user account</h1>"));
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RecordDisplay {
    pub id: String,
    pub timestamp: String,
    pub reading: String,
}

impl From<Record> for RecordDisplay {
    fn from(value: Record) -> Self {
        Self {
            id: value.id().to_string(),
            timestamp: value.timestamp().to_string(),
            reading: value.reading().to_string(),
        }
    }
}
