use crate::state::AppState;
use crate::authentication::user::UserCredentials;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect};
use axum::routing::{get, post};
use axum::{Form, Router};
use serde::Deserialize;
use tower_sessions::Session;
use crate::error::AppResult;

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

async fn register(
    State(state): State<AppState>,
    Form(credentials): Form<UserCreationFormData>
) -> impl IntoResponse {
    if credentials.password != credentials.password_confirmation {
        tracing::debug!("REJECTED user creation attempt: password confirmation does not match");
        return (StatusCode::BAD_REQUEST, "Bad Request: Password confirmation must match password!");
    }

    match state.repository.create_user(&credentials.username, &credentials.password).await {
        Ok(_user) => {
            (StatusCode::CREATED, "User created successfully")
        }
        Err(err) => {
            tracing::error!("REJECTED user creation attempt: {}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error: Could not create user!")
        }
    }
}

async fn login(
    session: Session,
    State(state): State<AppState>,
    Form(credentials): Form<LoginFormData>
) -> AppResult<impl IntoResponse> {
    let username = credentials.username.clone();
    tracing::debug!("login attempt, username: {}", username);
    match state.repository.get_user_by_username(&credentials.username).await {
        Ok(user) => {
            let credentials: UserCredentials = credentials.into();
            if user.credentials_is(credentials)? {
                match session.insert("user", user.clone()).await {
                    Ok(_) => {
                        tracing::debug!("user logged in, username: {}", username);
                        Ok(Redirect::to("/"))
                    },
                    Err(err) => {
                        tracing::error!("User login error. Failed to save session data for user {}: {}", username, err);
                        Ok(Redirect::to("/login?error=internal_error"))
                    }
                }

            } else {
                tracing::debug!("User login rejected, wrong credential, username: {}", username);
                Ok(Redirect::to("/login?error=wrong_credentials"))
            }
        },
        Err(err) => {
            tracing::debug!("User retrieval error, failed to look up user: {}, due to: {}", username, err);
            Ok(Redirect::to("/login?error=unable_to_lookup_user"))
        }
    }
}

async fn logout(session: Session) -> impl IntoResponse {
    session.delete().await.expect("Failed to delete session");
    Redirect::to("/")
}
