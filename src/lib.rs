mod api;
mod status;

pub async fn app() -> axum::Router {
    axum::Router::new()
        .route("/", axum::routing::get(get_root))
        .nest("/status", status::status())
        .nest("/api", api::api())
}

async fn get_root() -> &'static str {
    "Hello, world!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_root() {
        let result = get_root().await;
        assert_eq!(result, "Hello, world!");
    }
}
