use axum::middleware;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::TraceLayer;
use tower_sessions::SessionManagerLayer;
use tower_sessions_sqlx_store::PostgresStore;
use authentication::api_key;

mod api;
mod authentication;
mod error;
mod http_security_headers;
mod repository;
mod state;
mod status;
mod web;

pub async fn app(db_pool: sqlx::PgPool) -> axum::Router {
    let state = state::AppState::new(db_pool.clone());
    let session_store = PostgresStore::new(db_pool)
        .with_schema_name("auth")
        .unwrap()
        .with_table_name("sessions")
        .unwrap();
    session_store.migrate().await.unwrap();
    let session_layer = SessionManagerLayer::new(session_store);

    let x_request_id = axum::http::HeaderName::from_static("x-request-id");

    axum::Router::new()
        .merge(web::web())
        .nest("/status", status::status())
        .nest("/api_keys", api_key::router())
        .nest("/api", api::api())
        .with_state(state)
        // Request correlation (adds x-request-id header if missing, and propagates it to responses)
        .layer(PropagateRequestIdLayer::new(x_request_id.clone()))
        .layer(SetRequestIdLayer::new(x_request_id.clone(), MakeRequestUuid))
        // HTTP request/response tracing
        .layer(TraceLayer::new_for_http())
        .layer(session_layer)
        .layer(middleware::from_fn(http_security_headers::add_security_headers))
}
