use axum::Router;
use tower_http::cors::{Any, CorsLayer};

mod routes;
mod services;
mod state;
mod models;
mod error;

use crate::routes::init_routes;
use crate::state::AppState;

#[tokio::main]
async fn main() {
    let state = AppState::new();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = init_routes()
        .with_state(state)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001")
        .await
        .unwrap();

    println!("Users MS running on http://localhost:3001");

    axum::serve(listener, app)
        .await
        .unwrap();
}
