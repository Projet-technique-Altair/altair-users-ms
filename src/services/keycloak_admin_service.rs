use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Clone)]
pub struct KeycloakAdminService {
    base_url: String,
    realm: String,
    admin_realm: String,
    admin_client_id: String,
    admin_username: String,
    admin_password: String,
    sync_username: bool,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RoleRepresentation {
    id: Option<String>,
    name: Option<String>,
}

impl KeycloakAdminService {
    pub fn from_env() -> Option<Self> {
        // Service is disabled if mandatory admin env vars are missing.
        let base_url = std::env::var("KEYCLOAK_URL").ok()?;
        let realm = std::env::var("KEYCLOAK_REALM").ok()?;
        let admin_username = std::env::var("KEYCLOAK_ADMIN_USERNAME").ok()?;
        let admin_password = std::env::var("KEYCLOAK_ADMIN_PASSWORD").ok()?;

        let admin_realm =
            std::env::var("KEYCLOAK_ADMIN_REALM").unwrap_or_else(|_| "master".to_string());
        let admin_client_id =
            std::env::var("KEYCLOAK_ADMIN_CLIENT_ID").unwrap_or_else(|_| "admin-cli".to_string());
        let sync_username = std::env::var("KEYCLOAK_SYNC_USERNAME")
            .map(|v| v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

        Some(Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            realm,
            admin_realm,
            admin_client_id,
            admin_username,
            admin_password,
            sync_username,
        })
    }

    pub async fn sync_profile(
        &self,
        keycloak_id: &str,
        pseudo: Option<&str>,
        email: Option<&str>,
        role: Option<&str>,
    ) -> Result<(), AppError> {
        // One admin token is reused for all Keycloak calls in this request.
        let token = self.fetch_admin_token()?;

        if pseudo.is_some() || email.is_some() {
            self.update_user_identity(&token, keycloak_id, pseudo, email)?;
        }

        if let Some(role_name) = role {
            self.sync_realm_role(&token, keycloak_id, role_name)?;
        }

        Ok(())
    }

    fn fetch_admin_token(&self) -> Result<String, AppError> {
        let token_url = format!(
            "{}/realms/{}/protocol/openid-connect/token",
            self.base_url, self.admin_realm
        );

        let output = Command::new("curl")
            .args([
                "-sS",
                "-f",
                "-X",
                "POST",
                &token_url,
                "-H",
                "Content-Type: application/x-www-form-urlencoded",
                "-d",
                &format!(
                    "grant_type=password&client_id={}&username={}&password={}",
                    self.admin_client_id, self.admin_username, self.admin_password
                ),
            ])
            .output()
            .map_err(|e| AppError::Internal(format!("Failed to execute curl for token: {e}")))?;

        if !output.status.success() {
            return Err(AppError::Internal(format!(
                "Keycloak token request failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let payload: TokenResponse = serde_json::from_slice(&output.stdout).map_err(|e| {
            AppError::Internal(format!(
                "Failed to decode Keycloak token response: {e}; body={}",
                String::from_utf8_lossy(&output.stdout)
            ))
        })?;

        Ok(payload.access_token)
    }

    fn update_user_identity(
        &self,
        token: &str,
        keycloak_id: &str,
        pseudo: Option<&str>,
        email: Option<&str>,
    ) -> Result<(), AppError> {
        let mut body = serde_json::Map::new();
        // Username sync is optional because many realms keep username immutable.
        if self.sync_username {
            if let Some(p) = pseudo {
                body.insert(
                    "username".to_string(),
                    serde_json::Value::String(p.to_string()),
                );
            }
        }
        if let Some(e) = email {
            body.insert(
                "email".to_string(),
                serde_json::Value::String(e.to_string()),
            );
            body.insert("emailVerified".to_string(), serde_json::Value::Bool(true));
        }

        let url = format!(
            "{}/admin/realms/{}/users/{}",
            self.base_url, self.realm, keycloak_id
        );
        self.exec_json_request("PUT", &url, token, &serde_json::Value::Object(body))?;
        Ok(())
    }

    fn sync_realm_role(
        &self,
        token: &str,
        keycloak_id: &str,
        target_role: &str,
    ) -> Result<(), AppError> {
        let current_url = format!(
            "{}/admin/realms/{}/users/{}/role-mappings/realm",
            self.base_url, self.realm, keycloak_id
        );

        let current_roles: Vec<RoleRepresentation> =
            self.exec_json_get(&current_url, token, "Keycloak roles fetch failed")?;

        // Replace managed roles atomically (admin/creator/learner) with target role.
        let managed = ["admin", "creator", "learner"];
        let to_remove: Vec<RoleRepresentation> = current_roles
            .into_iter()
            .filter(|r| {
                r.name
                    .as_deref()
                    .map(|n| managed.contains(&n))
                    .unwrap_or(false)
            })
            .collect();

        if !to_remove.is_empty() {
            self.exec_json_request(
                "DELETE",
                &current_url,
                token,
                &serde_json::to_value(to_remove).map_err(|e| AppError::Internal(e.to_string()))?,
            )?;
        }

        let role_url = format!(
            "{}/admin/realms/{}/roles/{}",
            self.base_url, self.realm, target_role
        );
        let target_representation: RoleRepresentation =
            self.exec_json_get(&role_url, token, "Keycloak role lookup failed")?;

        self.exec_json_request(
            "POST",
            &current_url,
            token,
            &serde_json::to_value(vec![target_representation])
                .map_err(|e| AppError::Internal(e.to_string()))?,
        )?;

        Ok(())
    }

    fn exec_json_get<T: for<'de> Deserialize<'de>>(
        &self,
        url: &str,
        token: &str,
        error_prefix: &str,
    ) -> Result<T, AppError> {
        let output = Command::new("curl")
            .args([
                "-sS",
                "-f",
                "-X",
                "GET",
                url,
                "-H",
                &format!("Authorization: Bearer {token}"),
            ])
            .output()
            .map_err(|e| AppError::Internal(format!("{error_prefix}: {e}")))?;

        if !output.status.success() {
            return Err(AppError::Internal(format!(
                "{error_prefix}: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        serde_json::from_slice(&output.stdout).map_err(|e| {
            AppError::Internal(format!(
                "{error_prefix}: decode error {e}, body={}",
                String::from_utf8_lossy(&output.stdout)
            ))
        })
    }

    fn exec_json_request(
        &self,
        method: &str,
        url: &str,
        token: &str,
        body: &serde_json::Value,
    ) -> Result<(), AppError> {
        let payload = serde_json::to_string(body).map_err(|e| AppError::Internal(e.to_string()))?;
        let output = Command::new("curl")
            .args([
                "-sS",
                "-f",
                "-X",
                method,
                url,
                "-H",
                &format!("Authorization: Bearer {token}"),
                "-H",
                "Content-Type: application/json",
                "-d",
                &payload,
            ])
            .output()
            .map_err(|e| AppError::Internal(format!("Keycloak request failed: {e}")))?;

        if !output.status.success() {
            return Err(AppError::Internal(format!(
                "Keycloak request failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    pub fn toggle_realm_role(
        &self,
        keycloak_id: &str,
        new_role: &str,
    ) -> Result<(), AppError> {
        let token = self.fetch_admin_token()?;

        self.sync_realm_role(&token, keycloak_id, new_role)?;

        Ok(())
    }

    pub fn update_password(
        &self,
        keycloak_id: &str,
        new_password: &str,
    ) -> Result<(), AppError> {

        let token = self.fetch_admin_token()?;

        let url = format!(
            "{}/admin/realms/{}/users/{}/reset-password",
            self.base_url, self.realm, keycloak_id
        );

        let body = serde_json::json!({
            "type": "password",
            "value": new_password,
            "temporary": false
        });

        self.exec_json_request("PUT", &url, &token, &body)?;

        Ok(())
    }
}
