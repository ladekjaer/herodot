use crate::state::AppState;
use crate::user::UserCredentials;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect};
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
                        println!("USER LOGIN by user {}", username);
                        Redirect::to("/")
                    },
                    Err(err) => {
                        eprintln!("USER LOGIN ERROR: Failed to save session data for user {}: {}", username, err);
                        Redirect::to("/login?error=internal_error")
                    }
                }

            } else {
                eprintln!("USER LOGIN REJECTED: Wrong credentials for by user: {}", username);
                Redirect::to("/login?error=wrong_credentials")
            }
        },
        Err(err) => {
            eprintln!("USER LOGIN ERROR: Failed to look up user: {}, due to: {}", username, err);
            Redirect::to("/login?error=unable_to_lookup_user")
        }
    }
}

async fn logout(session: Session) -> impl IntoResponse {
    session.delete().await.expect("Failed to delete session");
    Redirect::to("/")
}
