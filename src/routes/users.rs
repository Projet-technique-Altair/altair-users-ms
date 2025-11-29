use axum::{Router, routing::get, extract::Path, Json};
use crate::models::user::User;
use uuid::Uuid;

pub fn users_routes() -> Router {
    Router::new()
        // GET /users/:id
        .route("/:id", get(get_user_by_id))
}

pub async fn get_user_by_id(Path(id): Path<Uuid>) -> Json<User> {
    Json(User {
        user_id: id,
        name: "Mock User".into(),
        role: "teacher".into(),
    })
}
