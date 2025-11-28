pub mod me;
pub mod users;

use axum::Router;

pub fn me_routes() -> Router {
    Router::new()
        .route("/", axum::routing::get(me::get_me))
}

pub fn users_routes() -> Router {
    Router::new()
        .route("/:id", axum::routing::get(users::get_user_by_id))
}
