use axum::Router;
use crate::state::AppState;

pub mod users;

pub fn init_routes() -> Router<AppState> {
    Router::new()
        .nest("/users", users::routes())
        .route("/health", axum::routing::get(|| async { "ok" }))
}
