use serde_json::json;
use serde_json::Value;

#[derive(Clone)]
pub struct UsersService;

impl UsersService {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn find_user(&self, id: i32) -> Value {
        json!({
            "id": id,
            "name": format!("User {}", id)
        })
    }
}
