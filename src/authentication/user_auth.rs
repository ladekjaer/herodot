use uuid::Uuid;
use axum::extract::FromRequestParts;
use axum::http::StatusCode;
use axum::http::request::Parts;
use axum::{Extension, RequestPartsExt};
use tower_sessions::Session;
use crate::authentication::user::User;

pub(crate) struct AuthUser {
    id: Uuid,
    username: String,
}

impl AuthUser {
    pub fn new(user: User) -> Self {
        AuthUser {
            id: *user.id(),
            username: user.username().to_string(),
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn username(&self) -> &str {
        &self.username
    }
}

impl<S: Send + Sync> FromRequestParts<S> for AuthUser {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let Extension(session) = parts.extract::<Extension<Session>>()
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Session not found"))?;

        match session.get::<User>("user").await {
            Ok(user) => {
                if let Some(user) = user {
                    Ok(AuthUser::new(user))
                } else {
                    Err((StatusCode::UNAUTHORIZED, "Unauthorized"))
                }
            }
            Err(error) => {
                eprintln!("Error getting user from session: {}", error);
                Err((StatusCode::INTERNAL_SERVER_ERROR, "Session error"))
            }
        }
    }
}