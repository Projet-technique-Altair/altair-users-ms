use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct User {
    pub user_id: Uuid,
    pub name: String,
    pub role: String,
}
