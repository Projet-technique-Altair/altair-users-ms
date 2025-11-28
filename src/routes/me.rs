use axum::{Json};
use crate::models::user::User;
use uuid::Uuid;

pub async fn get_me() -> Json<User> {
    Json(User {
        user_id: Uuid::new_v4(),
        name: "John Doe".into(),
        role: "student".into(),
    })
}
