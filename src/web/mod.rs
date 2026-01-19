use axum::Router;
use crate::state::AppState;

pub(crate) fn web() -> Router<AppState> {
    Router::new()
        .route("/", axum::routing::get(get_root))
}

async fn get_root() -> &'static str {
    "Herodot WebApp - Self hosting climate date repository"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_root() {
        let result = get_root().await;
        assert_eq!(result, "Herodot WebApp - Self hosting climate date repository");
    }
}
