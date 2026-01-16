use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let app = herodot::app().await;

    let addr = SocketAddr::from((std::net::Ipv4Addr::UNSPECIFIED, 8080));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
