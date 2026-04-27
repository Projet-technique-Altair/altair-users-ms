use crate::state::AppState;
use axum::{
    routing::{get, post},
    Router,
};

use crate::routes::health::health;
use crate::routes::me::{toggle_my_role, update_password};
use crate::routes::users::{
    create_admin_user_sanction, get_admin_user_detail, list_users_admin, search_users,
    update_admin_account_status,
};

pub mod health;
pub mod me;
pub mod users;

pub fn init_routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/admin/users", get(list_users_admin))
        .route("/admin/users/:id/detail", get(get_admin_user_detail))
        .route(
            "/admin/users/:id/sanctions",
            post(create_admin_user_sanction),
        )
        .route(
            "/admin/users/:id/account-status",
            axum::routing::patch(update_admin_account_status),
        )
        .route("/me", get(me::me).patch(me::update_me))
        .route("/search", get(search_users))
        .route("/me/", get(me::me).patch(me::update_me))
        .route("/me/toggle-role", post(toggle_my_role))
        .route("/me/password", post(update_password))
        .nest("/users", users::routes())
}
