use axum::{Json, Router};
use axum::routing::get;
use serde_json::{json, Value};

pub(crate) fn status() -> Router {
    Router::new()
        .route("/", get(get_status))
}

async fn get_status() -> Json<Value> {
    let status = json!({
        "status": "ok",
        "message": "server is running",
        "version": env!("CARGO_PKG_VERSION")
    });
    Json(status)
}
