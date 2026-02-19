use tower_sessions::SessionManagerLayer;
use tower_sessions_sqlx_store::PostgresStore;

mod api;
mod repository;
mod state;
mod status;
mod user;
mod user_api;
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
        .nest("/api", api::api())
        .with_state(state)
        .layer(session_layer)
}
