use axum::{Router};
use tokio::net::TcpListener;

mod state;
mod routes;
mod error;
mod services;

use state::AppState;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let state = AppState::init();

    let app = routes::init_routes().with_state(state);

    let listener = TcpListener::bind("0.0.0.0:3001")
        .await
        .expect("❌ Failed to bind port 3001");

    println!("Users-MS running on http://localhost:3001");

    axum::serve(listener, app).await.unwrap();
}
