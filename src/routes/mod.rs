use axum::Router;
use crate::state::AppState;
use axum::routing::get;

pub mod health;
pub mod me;
pub mod users;

pub fn init_routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health::health))
        .nest("/me", me::routes())
        .nest("/users", users::routes())
}
