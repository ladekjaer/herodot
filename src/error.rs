use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use sqlx;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug)]
pub enum AppError {
    BadRequest(&'static str),
    Unauthorized(&'static str),
    Forbidden(&'static str),
    NotFound(&'static str),
    Conflict(&'static str),
    InternalServerError(&'static str),

    SqlxError(sqlx::Error),
    TeraError(tera::Error),
    SessionError(tower_sessions::session::Error),
    User(crate::authentication::user::UserError),
}

impl AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,

            AppError::SqlxError(error) => match error {
                sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            },
            AppError::TeraError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::SessionError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::User(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    
    fn public_message(&self) -> &'static str {
        match self {
            AppError::BadRequest(msg)
            | AppError::Unauthorized(msg)
            | AppError::Forbidden(msg)
            | AppError::NotFound(msg)
            | AppError::Conflict(msg)
            | AppError::InternalServerError(msg) => msg,
            
            AppError::SqlxError(_) => "database error",
            AppError::TeraError(_) => "template rendering error",
            AppError::SessionError(_) => "session error",
            AppError::User(_) => "user handling error",
        }
    }

    fn error_code(&self) -> &'static str {
        match self {
            AppError::BadRequest(_) => "BAD_REQUEST",
            AppError::Unauthorized(_) => "UNAUTHORIZED",
            AppError::Forbidden(_) => "FORBIDDEN",
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::Conflict(_) => "CONFLICT",
            AppError::InternalServerError(_) => "INTERNAL_SERVER_ERROR",
            AppError::SqlxError(_) => "DATABASE_ERROR",
            AppError::TeraError(_) => "TEMPLATE_ERROR",
            AppError::SessionError(_) => "SESSION_ERROR",
            AppError::User(_) => "USER_HANDLING_ERROR",
        }
    }

    pub fn from_commit_record_error(error: sqlx::Error) -> Self {
        use sqlx::error::ErrorKind;

        match error {
            sqlx::Error::Database(db_error) => match db_error.kind() {
                ErrorKind::UniqueViolation => AppError::Conflict("record already exist"),
                ErrorKind::ForeignKeyViolation => AppError::BadRequest("invalid record: references non-existing entity"),
                ErrorKind::NotNullViolation => AppError::BadRequest("invalid record: missing required field"),
                ErrorKind::CheckViolation => AppError::BadRequest("invalid record: violates constraints"),
                _ => AppError::SqlxError(sqlx::Error::Database(db_error)),
            },
            other => AppError::SqlxError(other),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match &self {
            AppError::SqlxError(error) => tracing::warn!("AppError(Sqlx): {error}"),
            AppError::TeraError(error) => tracing::warn!("AppError(Tera): {error}"),
            AppError::SessionError(error) => tracing::warn!("AppError(Session): {error}"),
            AppError::User(error) => tracing::warn!("AppError(User): {error}"),
            _ => {}
        }

        let status = self.status_code();
        let body = json!({
            "error": self.error_code(),
            "message": self.public_message(),
        });

        (status, Json(body)).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(value: sqlx::Error) -> Self {
        AppError::SqlxError(value)
    }
}

impl From<tera::Error> for AppError {
    fn from(value: tera::Error) -> Self {
        AppError::TeraError(value)
    }
}

impl From<tower_sessions::session::Error> for AppError {
    fn from(value: tower_sessions::session::Error) -> Self {
        AppError::SessionError(value)
    }
}

impl From<crate::authentication::user::UserError> for AppError {
    fn from(value: crate::authentication::user::UserError) -> Self {
        AppError::User(value)
    }
}
