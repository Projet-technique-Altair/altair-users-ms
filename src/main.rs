use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::EnvFilter;

mod error;
mod models;
mod routes;
mod services;
mod state;

use crate::routes::init_routes;
use crate::state::AppState;

#[tokio::main]
async fn main() {
    // Logger
    println!("MAIN START");

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    tracing::info!("🚀 Users MS booting");

    dotenvy::dotenv().ok();

    // AppState NE retourne PAS Result
    let state = AppState::new().await;
    tracing::info!("✅ AppState initialized");

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = init_routes().with_state(state).layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001")
        .await
        .expect("Failed to bind port 3001");

    tracing::info!("🌐 Users MS running on 0.0.0.0:3001");

    axum::serve(listener, app).await.unwrap();

    println!("MAIN END");
}
