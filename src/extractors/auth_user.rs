use axum::{
    async_trait,
    extract::FromRequestParts,
    http::request::Parts,
};

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub keycloak_id: String,
    pub name: String,
    pub email: String,
    pub roles: Vec<String>,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let keycloak_id = parts
            .headers
            .get("x-altair-keycloak-id")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("Missing keycloak id".into()))?
            .to_string();

        let name = parts
            .headers
            .get("x-altair-name")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown")
            .to_string();

        let email = parts
            .headers
            .get("x-altair-email")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown@altair.local")
            .to_string();

        let roles = parts
            .headers
            .get("x-altair-roles")
            .and_then(|v| v.to_str().ok())
            .map(|s| {
                s.split(',')
                    .map(|r| r.trim().to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        Ok(AuthUser {
            keycloak_id,
            name,
            email,
            roles,
        })
    }
}
