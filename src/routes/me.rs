use axum::{Router, routing::get, Json};
use uuid::Uuid;

use crate::models::user::User;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/", get(get_me))
}

async fn get_me() -> Json<User> {
    Json(User {
        user_id: Uuid::new_v4(),
        name: "Current User".into(),
        pseudo: "current".into(),
        mail: "current@example.com".into(),
        post: "student".into(),
        status: "active".into(),
    })
}
