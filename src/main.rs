use axum::{Router};
use tokio::net::TcpListener;

mod state;
mod error;
mod models;
mod services;
mod routes;

use state::AppState;
use routes::init_routes;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let state = AppState::init();

    let app = init_routes().with_state(state);

    let listener = TcpListener::bind("0.0.0.0:3001")
        .await
        .expect("Failed to bind port 3001");

    println!("Users-MS running on http://localhost:3001");

    axum::serve(listener, app).await.unwrap();
}
