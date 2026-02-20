use axum::http::{
    header::{AUTHORIZATION, CONTENT_TYPE},
    HeaderName, HeaderValue, Method,
};
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing_subscriber::EnvFilter;

mod error;
mod extractors;
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
        .allow_origin(AllowOrigin::list([
            HeaderValue::from_static("http://localhost:5173"),
            HeaderValue::from_static("http://localhost:3000"),
        ]))
        .allow_methods([Method::GET, Method::OPTIONS])
        .allow_headers([
            AUTHORIZATION,
            CONTENT_TYPE,
            HeaderName::from_static("x-altair-keycloak-id"),
            HeaderName::from_static("x-altair-name"),
            HeaderName::from_static("x-altair-email"),
            HeaderName::from_static("x-altair-roles"),
            HeaderName::from_static("x-altair-user-id"),
        ]);

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
