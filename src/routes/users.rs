use axum::{extract::Path, Json};
use crate::models::user::User;
use uuid::Uuid;

pub async fn get_user_by_id(Path(id): Path<Uuid>) -> Json<User> {
    Json(User {
        user_id: id,
        name: "Mock User".into(),
        role: "teacher".into(),
    })
}
