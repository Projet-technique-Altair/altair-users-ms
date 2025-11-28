use axum::{Router};
use tokio::net::TcpListener;

mod routes;
mod models;

use routes::{me_routes, users_routes};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .nest("/me", me_routes())
        .nest("/users", users_routes());

    let addr = "0.0.0.0:4001";
    println!("Users-MS running on {}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
