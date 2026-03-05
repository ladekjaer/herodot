use axum::middleware;
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

    axum::Router::new()
        .merge(web::web())
        .nest("/status", status::status())
        .nest("/api_keys", api_key::router())
        .nest("/api", api::api())
        .with_state(state)
        .layer(session_layer)
        .layer(middleware::from_fn(http_security_headers::add_security_headers))
}
