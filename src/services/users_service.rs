use uuid::Uuid;
use serde_json::{json, Value};
use crate::models::User;

#[derive(Clone)]
pub struct UsersService;

impl UsersService {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_mock_user(&self) -> Value {
        json!({
            "user_id": Uuid::new_v4(),
            "name": "Current User",
            "pseudo": "current",
            "mail": "current@example.com",
            "post": "student",
            "status": "active"
        })
    }

    pub fn get_mock_user_by_id(&self, id: String) -> Value {
        json!({
            "user_id": id,
            "name": "John Doe",
            "pseudo": "jdoe",
            "mail": "jdoe@example.com",
            "post": "student",
            "status": "active"
        })
    }
}
