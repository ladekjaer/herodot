use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let database_url = "postgresql://webapp:password@localhost:5432/herodot";
    let db_pool = sqlx::PgPool::connect(database_url).await.unwrap();
    let app = herodot::app(db_pool).await;

    let addr = SocketAddr::from((std::net::Ipv4Addr::UNSPECIFIED, 8080));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
