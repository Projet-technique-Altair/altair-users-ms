use serde::{Deserialize, Serialize};
use uuid::Uuid;
use sqlx::FromRow;

#[derive(Serialize, Deserialize, Clone, FromRow)]
pub struct User {
    pub user_id: Uuid,
    pub name: String,
    pub pseudo: String,
    pub mail: String,
    pub post: String,   // student | creator | admin
    pub status: String, // active | blackhole | banned
}
