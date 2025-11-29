use axum::Router;
use tokio::net::TcpListener;

mod routes;
mod models;

use routes::{me_routes, users_routes, health_routes, metrics_routes};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .nest("/me", me_routes())
        .nest("/users", users_routes())
        .nest("/health", health_routes())
        .nest("/metrics/basic", metrics_routes());

    let addr = "0.0.0.0:3001";
    println!("Users-MS running on {}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
