use crate::state::AppState;
use crate::user::UserCredentials;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect};
use axum::routing::{get, post};
use axum::{Form, Router};
use serde::Deserialize;
use tower_sessions::Session;

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

#[derive(Debug, Clone, Deserialize)]
struct UserCreationFormData {
    pub(crate) username: String,
    pub(crate) password: String,
    #[serde(rename = "password-confirm")]
    pub(crate) password_confirmation: String
}

pub(crate) fn user_router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", get(logout))
}

async fn register(State(state): State<AppState>, Form(credentials): Form<UserCreationFormData>) -> impl IntoResponse {
    if credentials.password != credentials.password_confirmation {
        eprintln!("REJECTED user creation attempt: password confirmation does not match");
        return (StatusCode::BAD_REQUEST, "Bad Request: Password confirmation must match password!");
    }

    match state.repository.create_user(&credentials.username, &credentials.password).await {
        Ok(_user) => {
            (StatusCode::CREATED, "User created successfully")
        }
        Err(err) => {
            eprintln!("REJECTED user creation attempt: {}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error: Could not create user!")
        }
    }
}

async fn login(session: Session, State(state): State<AppState>, Form(credentials): Form<LoginFormData>) -> impl IntoResponse {
    let username = credentials.username.clone();
    println!("Login attempt with username: {}", username);
    let user = state.repository.get_user_by_username(&credentials.username).await;
    match user {
        Ok(user) => {
            let credentials: UserCredentials = credentials.into();
            if user.credentials_is(credentials).unwrap() {
                match session.insert("user", user.clone()).await {
                    Ok(_) => {
                        println!("ACCEPTED login attempt by user {}", username);
                        Html(format!("Login successful for user: {}", username))
                    },
                    Err(err) => {
                        eprintln!("INTERNAL ERROR REJECTED login attempt by user {}: {}", username, err);
                        Html(format!("INTERNAL ERROR Login failed for user: {}", username))
                    }
                }

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

async fn logout(session: Session) -> impl IntoResponse {
    session.delete().await.expect("Failed to delete session");
    Redirect::to("/")
}
