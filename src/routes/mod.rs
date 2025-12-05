use axum::Router;
use crate::state::AppState;

pub mod users;
pub mod health;
pub mod me;
pub mod metrics;

pub fn init_routes() -> Router<AppState> {
    Router::new()
        .nest("/users", users::routes())
        .nest("/health", health::routes())
        .nest("/me", me::routes())
        .nest("/metrics", metrics::routes())
}
