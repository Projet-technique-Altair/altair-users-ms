use axum::{routing::get, Router};
use crate::state::AppState;

use crate::routes::health::health;


pub mod health;
pub mod me;
pub mod users;

pub fn init_routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .nest("/me", me::routes())
        .nest("/users", users::routes())
}
