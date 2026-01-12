use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::error::AppError;

//
// =======================
// API ENUMS (exposés)
// =======================
//

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UserPost {
    Student,
    Creator,
    Admin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UserStatus {
    Active,
    Blackhole,
    Banned,
}

//
// =======================
// DB MODEL (SQLx)
// =======================
//

#[derive(Debug, Clone, FromRow)]
pub struct UserRow {
    pub user_id: Uuid,
    pub name: String,
    pub pseudo: String,
    pub mail: String,
    pub post: String,   // snake_case en DB
    pub status: String, // snake_case en DB
}

//
// =======================
// API MODEL (exposé)
// =======================
//

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    pub user_id: Uuid,
    pub name: String,
    pub pseudo: String,
    pub mail: String,
    pub post: UserPost,
    pub status: UserStatus,
}

//
// =======================
// DB → API MAPPING
// =======================
//

impl TryFrom<UserRow> for User {
    type Error = AppError;

    fn try_from(row: UserRow) -> Result<Self, Self::Error> {
        let post = match row.post.as_str() {
            "student" => UserPost::Student,
            "creator" => UserPost::Creator,
            "admin" => UserPost::Admin,
            other => {
                return Err(AppError::Internal(format!(
                    "Invalid user post in DB: {other}"
                )))
            }
        };

        let status = match row.status.as_str() {
            "active" => UserStatus::Active,
            "blackhole" => UserStatus::Blackhole,
            "banned" => UserStatus::Banned,
            other => {
                return Err(AppError::Internal(format!(
                    "Invalid user status in DB: {other}"
                )))
            }
        };

        Ok(User {
            user_id: row.user_id,
            name: row.name,
            pseudo: row.pseudo,
            mail: row.mail,
            post,
            status,
        })
    }
}
