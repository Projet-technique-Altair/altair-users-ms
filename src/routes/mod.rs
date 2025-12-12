use axum::Router;
use crate::state::AppState;

pub mod health;
pub mod me;
pub mod users;

pub fn init_routes() -> Router<AppState> {
    Router::new()
        .route("/health", axum::routing::get(health::health))
        .nest("/me", me::routes())
        .nest("/users", users::routes())
}
