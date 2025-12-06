use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    pub user_id: Uuid,
    pub name: String,
    pub pseudo: String,
    pub mail: String,
    pub post: String,   // student | teacher | admin
    pub status: String, // active | banned | blackhole
}
