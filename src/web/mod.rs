use axum::extract::State;
use axum::response::{Html, IntoResponse};
use axum::Router;
use crate::state::AppState;

use lazy_static::lazy_static;

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
        .route("/", axum::routing::get(index))
        .route("/ds18b20", axum::routing::get(ds18b20))
}

async fn index() -> impl IntoResponse {
    let context = tera::Context::new();
    let output = Tera.render("index.html", &context).unwrap();
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
    async fn test_index() {
        let response = index().await.into_response();
        assert_eq!(response.status(), axum::http::StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_str.contains("<h1>Herodot WebApp - Self-hosting climate date repository</h1>"));
    }
}
