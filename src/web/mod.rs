use crate::state::AppState;
use axum::extract::State;
use axum::response::{Html, IntoResponse};
use axum::routing::{get, post};
use axum::{Form, Router};

use crate::user::UserCredentials;
use lazy_static::lazy_static;
use serde::Deserialize;

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
        .route("/login", get(login))
        .route("/login", post(login_post))
        .route("/ds18b20", get(ds18b20))
}

async fn index() -> impl IntoResponse {
    let context = tera::Context::new();
    let output = Tera.render("index.html", &context).unwrap();
    Html(output)
}

async fn login() -> impl IntoResponse {
    let context = tera::Context::new();
    let output = Tera.render("login.html", &context).unwrap();
    Html(output)
}

#[derive(Debug, Clone, Deserialize)]
struct LoginFormData {
    username: String,
    password: String
}

impl From<LoginFormData> for UserCredentials {
    fn from(value: LoginFormData) -> Self {
        Self::new(value.username, value.password)
    }
}

async fn login_post(State(state): State<AppState>, Form(credentials): Form<LoginFormData>) -> impl IntoResponse {
    let username = credentials.username.clone();
    println!("Login attempt with username: {}", username);
    let user = state.repository.get_user_by_username(&credentials.username).await;
    match user {
        Ok(user) => {
            let credentials: UserCredentials = credentials.into();
            if user.credentials_is(credentials).unwrap() {
                Html(format!("Login successful for user: {}", username))
            } else {
                eprintln!("REJECTED login attempt by user {}: wrong credentials", username);
                Html(format!("Login failed for user: {}", username))
            }
        },
        Err(err) => {
            eprintln!("REJECTED login attempt by user {}, due to: {}", username, err);
            Html(format!("Login failed for user: {}", username))
        }
    }
}

async fn ds18b20(State(state): State<AppState>) -> impl IntoResponse {
    let mut context = tera::Context::new();
    let records = state.repository.get_all_ds18b20_records().await.unwrap();
    context.insert("records", &records);
    let output = Tera.render("ds18b20.html", &context).unwrap();
    Html(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_index() {
        let response = index().await.into_response();
        assert_eq!(response.status(), axum::http::StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_str.contains("<h1>Herodot WebApp - Self-hosting climate date repository</h1>"));
    }
}
