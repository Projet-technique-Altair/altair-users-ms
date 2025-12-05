use uuid::Uuid;
use serde_json::{json, Value};

use crate::models::user::User;

#[derive(Clone)]
pub struct UsersService;

impl UsersService {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn find_user(&self, id: Uuid) -> Value {
        // TODO: replace with real DB
        json!(User {
            user_id: id,
            name: "John Doe".into(),
            pseudo: "jdoe".into(),
            mail: "jdoe@example.com".into(),
            post: "student".into(),
            status: "active".into(),
        })
    }
}
