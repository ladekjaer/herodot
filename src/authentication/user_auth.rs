use uuid::Uuid;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::{Extension, RequestPartsExt};
use tower_sessions::Session;
use crate::authentication::user::User;
use crate::error::AppError;

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
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let Extension(session) = parts
            .extract::<Extension<Session>>()
            .await
            .map_err(|_| AppError::InternalServerError("Session not found"))?;

        match session.get::<User>("user").await? {
            Some(user) => Ok(AuthUser::new(user)),
            None => Err(AppError::Unauthorized("Unauthorized")),
        }
    }
}