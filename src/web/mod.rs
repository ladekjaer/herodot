use crate::state::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::Router;
use lazy_static::lazy_static;
use tower_sessions::Session;
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

pub(crate) fn web() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .route("/me", get(me))
        .route("/login", get(login))
        .route("/register", get(register))
        .route("/ds18b20", get(ds18b20))
        .nest("/users",user_api::user_router())
}

async fn index(session: Session) -> impl IntoResponse {
    let mut context = tera::Context::new();
    match session.get::<User>("user").await {
        Ok(user) => {
            if let Some(user) = user {
                context.insert("username", user.username());
            }
        }
        Err(error) => {
            eprintln!("Error getting user from session: {}", error);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }

    let output = Tera.render("index.html", &context).unwrap();
    Html(output).into_response()
}

async fn me(session: Session) -> impl IntoResponse {
    let mut context = tera::Context::new();

    let user = match session.get::<User>("user").await {
        Ok(user) => user,
        Err(error) => {
            eprintln!("Error getting user from session: {}", error);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    };

    let Some(user) = user else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    context.insert("username", user.username());

    let template = "me.html";

    match Tera.render("me.html", &context) {
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
