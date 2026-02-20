use crate::state::AppState;
use axum::extract::{FromRequestParts, State};
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::{Extension, RequestPartsExt, Router};
use lazy_static::lazy_static;
use tower_sessions::Session;
use uuid::Uuid;
use crate::user::User;
use crate::user_api;

lazy_static! {
    pub static ref Tera: tera::Tera = match tera::Tera::new("templates/**/*") {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Tera initialization error: {}", e);
            std::process::exit(1);
        }
    };
}

struct AuthUser {
    id: Uuid,
    username: String,
}

impl AuthUser {
    pub fn new(user: User) -> Self {
        AuthUser {
            id: user.id().clone(),
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

pub(crate) fn web() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .route("/me", get(me))
        .route("/login", get(login))
        .route("/register", get(register))
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
