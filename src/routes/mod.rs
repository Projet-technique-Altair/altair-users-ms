use crate::state::AppState;
use axum::{routing::get, Router};

use crate::routes::health::health;

pub mod health;
pub mod me;
pub mod users;

pub fn init_routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/me", get(me::me).patch(me::update_me))
        .route("/me/", get(me::me).patch(me::update_me))
        .nest("/users", users::routes())
}
