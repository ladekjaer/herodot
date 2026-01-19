mod api;
mod repository;
mod state;
mod status;
mod web;

pub async fn app(db_pool: sqlx::PgPool) -> axum::Router {
    let state = state::AppState::new(db_pool);
    axum::Router::new()
        .merge(web::web())
        .nest("/status", status::status())
        .nest("/api", api::api())
        .with_state(state)
}
