use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

//
// =======================
// DB MODEL (SQLx)
// =======================
//

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct UserRow {
    pub user_id: Uuid,
    pub keycloak_id: String,
    pub role: String,

    pub name: String,
    pub pseudo: String,
    pub email: String,

    pub avatar: Option<String>,
    pub last_login: Option<chrono::NaiveDateTime>,
    pub created_at: chrono::NaiveDateTime,
}

//
// =======================
// API MODEL (exposé)
// =======================
//

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub user_id: Uuid,
    pub role: String,

    pub name: String,
    pub pseudo: String,
    pub email: String,

    pub avatar: Option<String>,
    pub last_login: Option<chrono::NaiveDateTime>,
    pub created_at: chrono::NaiveDateTime,
}

//
// =======================
// DB → API MAPPING
// =======================
//

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        Self {
            user_id: row.user_id,
            role: row.role,

            name: row.name,
            pseudo: row.pseudo,
            email: row.email,

            avatar: row.avatar,
            last_login: row.last_login,
            created_at: row.created_at,
        }
    }
}
