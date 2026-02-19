use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::EnvFilter;

mod error;
mod models;
mod routes;
mod services;
mod state;
mod extractors;

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

    let port = std::env::var("PORT").unwrap_or("3001".to_string());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap_or_else(|_| panic!("Failed to bind port {}", port));

    tracing::info!("🌐 Users MS running on 0.0.0.0:{}", port);
    println!("Users MS running on http://localhost:{}", port);

    axum::serve(listener, app).await.unwrap();

    println!("MAIN END");
}
