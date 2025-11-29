use axum::{Router, routing::get, Json};
use crate::models::user::User;
use uuid::Uuid;

pub fn me_routes() -> Router {
    Router::new()
        // GET /me/
        .route("/", get(get_me))
}

pub async fn get_me() -> Json<User> {
    Json(User {
        user_id: Uuid::new_v4(),
        name: "John Doe".into(),
        role: "student".into(),
    })
}
