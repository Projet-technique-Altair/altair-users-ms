use axum::http::{HeaderName, HeaderValue, Method};
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing_subscriber::EnvFilter;

mod error;
mod extractors;
mod features;
mod models;
mod routes;
mod services;
mod state;

use crate::routes::init_routes;
use crate::state::AppState;

const DEFAULT_ALLOWED_ORIGINS: &str = "http://localhost:5173,http://localhost:3000";
const DEFAULT_ALLOWED_METHODS: &str = "GET,POST,PATCH,OPTIONS";
const DEFAULT_ALLOWED_HEADERS: &str = "authorization,content-type,x-altair-keycloak-id,x-altair-name,x-altair-email,x-altair-roles,x-altair-user-id";

fn parse_allowed_origins() -> Vec<HeaderValue> {
    let raw =
        std::env::var("ALLOWED_ORIGINS").unwrap_or_else(|_| DEFAULT_ALLOWED_ORIGINS.to_string());
    let parsed: Vec<HeaderValue> = raw
        .split(',')
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .filter_map(|origin| HeaderValue::from_str(origin).ok())
        .collect();

    if parsed.is_empty() {
        DEFAULT_ALLOWED_ORIGINS
            .split(',')
            .filter_map(|origin| HeaderValue::from_str(origin).ok())
            .collect()
    } else {
        parsed
    }
}

fn parse_allowed_methods() -> Vec<Method> {
    let raw =
        std::env::var("ALLOWED_METHODS").unwrap_or_else(|_| DEFAULT_ALLOWED_METHODS.to_string());
    let parsed: Vec<Method> = raw
        .split(',')
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .filter_map(|method| Method::from_bytes(method.as_bytes()).ok())
        .collect();

    if parsed.is_empty() {
        DEFAULT_ALLOWED_METHODS
            .split(',')
            .filter_map(|method| Method::from_bytes(method.as_bytes()).ok())
            .collect()
    } else {
        parsed
    }
}

fn parse_allowed_headers() -> Vec<HeaderName> {
    let raw =
        std::env::var("ALLOWED_HEADERS").unwrap_or_else(|_| DEFAULT_ALLOWED_HEADERS.to_string());
    let parsed: Vec<HeaderName> = raw
        .split(',')
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .filter_map(|header| HeaderName::from_bytes(header.to_ascii_lowercase().as_bytes()).ok())
        .collect();

    if parsed.is_empty() {
        DEFAULT_ALLOWED_HEADERS
            .split(',')
            .filter_map(|header| HeaderName::from_bytes(header.as_bytes()).ok())
            .collect()
    } else {
        parsed
    }
}

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

    let allowed_origins = parse_allowed_origins();
    let allowed_methods = parse_allowed_methods();
    let allowed_headers = parse_allowed_headers();

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(allowed_origins))
        .allow_methods(allowed_methods)
        .allow_headers(allowed_headers);

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
