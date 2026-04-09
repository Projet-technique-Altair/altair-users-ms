use crate::state::AppState;
use axum::{
    routing::{get, post},
    Router,
};

use crate::routes::health::health;
use crate::routes::me::{toggle_my_role, update_password};
use crate::routes::users::search_users;

pub mod health;
pub mod me;
pub mod users;

pub fn init_routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/me", get(me::me).patch(me::update_me))
        .route("/search", get(search_users))
        .route("/me/", get(me::me).patch(me::update_me))
        .route("/me/toggle-role", post(toggle_my_role))
        .route("/me/password", post(update_password))
        .nest("/users", users::routes())
}
