use std::env;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let database_url = env::var("DATABASE_URL").expect("Environment variable 'DATABASE_URL' must be set");
    let db_pool = sqlx::PgPool::connect(&database_url).await.unwrap();
    let app = herodot::app(db_pool).await;

    let port = env::var("PORT").expect("Environment variable 'PORT' must be set");
    let addr = SocketAddr::from((std::net::Ipv4Addr::UNSPECIFIED, port.parse().unwrap()));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
