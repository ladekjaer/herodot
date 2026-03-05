use std::env;
use std::net::SocketAddr;

fn init_tracing() {
    use tracing_subscriber::{fmt, EnvFilter};

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,tower_http=info,sqlx=warn"));

    fmt().with_env_filter(env_filter).compact().init();
}

#[tokio::main]
async fn main() {
    init_tracing();

    let database_url =
        env::var("DATABASE_URL").expect("Environment variable 'DATABASE_URL' must be set");
    let db_pool = sqlx::PgPool::connect(&database_url).await.unwrap();
    let app = herodot::app(db_pool).await;

    let port = env::var("PORT").expect("Environment variable 'PORT' must be set");
    let addr = SocketAddr::from((std::net::Ipv4Addr::UNSPECIFIED, port.parse().unwrap()));

    tracing::info!(%addr, "Starting web server on");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
