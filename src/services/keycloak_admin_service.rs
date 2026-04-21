use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Clone)]
pub struct KeycloakAdminService {
    base_url: reqwest::Url,
    realm: String,
    admin_realm: String,
    admin_client_id: String,
    admin_username: String,
    admin_password: String,
    sync_username: bool,
    http: reqwest::Client,
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

enum KeycloakEndpoint<'a> {
    Token { realm: &'a str },
    AdminUser { realm: &'a str, user_id: &'a str },
    RealmRoleMappings { realm: &'a str, user_id: &'a str },
    RealmRole { realm: &'a str, role: &'a str },
    ResetPassword { realm: &'a str, user_id: &'a str },
}

impl<'a> KeycloakEndpoint<'a> {
    fn segments(&self) -> Result<Vec<&'a str>, AppError> {
        match self {
            Self::Token { realm } => {
                validate_keycloak_segment(realm, "realm")?;
                Ok(vec!["realms", realm, "protocol", "openid-connect", "token"])
            }
            Self::AdminUser { realm, user_id } => {
                validate_keycloak_segment(realm, "realm")?;
                validate_keycloak_segment(user_id, "user_id")?;
                Ok(vec!["admin", "realms", realm, "users", user_id])
            }
            Self::RealmRoleMappings { realm, user_id } => {
                validate_keycloak_segment(realm, "realm")?;
                validate_keycloak_segment(user_id, "user_id")?;
                Ok(vec![
                    "admin",
                    "realms",
                    realm,
                    "users",
                    user_id,
                    "role-mappings",
                    "realm",
                ])
            }
            Self::RealmRole { realm, role } => {
                validate_keycloak_segment(realm, "realm")?;
                validate_keycloak_segment(role, "role")?;
                Ok(vec!["admin", "realms", realm, "roles", role])
            }
            Self::ResetPassword { realm, user_id } => {
                validate_keycloak_segment(realm, "realm")?;
                validate_keycloak_segment(user_id, "user_id")?;
                Ok(vec![
                    "admin",
                    "realms",
                    realm,
                    "users",
                    user_id,
                    "reset-password",
                ])
            }
        }
    }
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
        let base_url = parse_keycloak_base_url(&base_url)?;

        Some(Self {
            base_url,
            realm,
            admin_realm,
            admin_client_id,
            admin_username,
            admin_password,
            sync_username,
            http: reqwest::Client::new(),
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
        let token = self.fetch_admin_token().await?;

        if pseudo.is_some() || email.is_some() {
            self.update_user_identity(&token, keycloak_id, pseudo, email)
                .await?;
        }

        if let Some(role_name) = role {
            self.sync_realm_role(&token, keycloak_id, role_name).await?;
        }

        Ok(())
    }

    async fn fetch_admin_token(&self) -> Result<String, AppError> {
        let params = [
            ("grant_type", "password"),
            ("client_id", self.admin_client_id.as_str()),
            ("username", self.admin_username.as_str()),
            ("password", self.admin_password.as_str()),
        ];

        let response = self
            .post(KeycloakEndpoint::Token {
                realm: self.admin_realm.as_str(),
            })?
            .form(&params)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Token request failed: {e}")))?;

        let status = response.status();

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!(
                "Token request failed: {} - {}",
                status, body
            )));
        }

        let payload: TokenResponse = response
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("Decode error: {e}")))?;

        Ok(payload.access_token)
    }

    async fn update_user_identity(
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

        self.exec_json_request(
            reqwest::Method::PUT,
            KeycloakEndpoint::AdminUser {
                realm: self.realm.as_str(),
                user_id: keycloak_id,
            },
            token,
            &serde_json::Value::Object(body),
        )
        .await?;
        Ok(())
    }

    async fn sync_realm_role(
        &self,
        token: &str,
        keycloak_id: &str,
        target_role: &str,
    ) -> Result<(), AppError> {
        let current_roles: Vec<RoleRepresentation> = self
            .exec_json_get(
                KeycloakEndpoint::RealmRoleMappings {
                    realm: self.realm.as_str(),
                    user_id: keycloak_id,
                },
                token,
                "Keycloak roles fetch failed",
            )
            .await?;

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
                reqwest::Method::DELETE,
                KeycloakEndpoint::RealmRoleMappings {
                    realm: self.realm.as_str(),
                    user_id: keycloak_id,
                },
                token,
                &serde_json::to_value(to_remove).map_err(|e| AppError::Internal(e.to_string()))?,
            )
            .await?;
        }

        let target_representation: RoleRepresentation = self
            .exec_json_get(
                KeycloakEndpoint::RealmRole {
                    realm: self.realm.as_str(),
                    role: target_role,
                },
                token,
                "Keycloak role lookup failed",
            )
            .await?;

        self.exec_json_request(
            reqwest::Method::POST,
            KeycloakEndpoint::RealmRoleMappings {
                realm: self.realm.as_str(),
                user_id: keycloak_id,
            },
            token,
            &serde_json::to_value(vec![target_representation])
                .map_err(|e| AppError::Internal(e.to_string()))?,
        )
        .await?;

        Ok(())
    }

    async fn exec_json_get<T: for<'de> Deserialize<'de>>(
        &self,
        endpoint: KeycloakEndpoint<'_>,
        token: &str,
        error_prefix: &str,
    ) -> Result<T, AppError> {
        let response = self
            .get(endpoint)?
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("{error_prefix}: {e}")))?;

        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|e| AppError::Internal(format!("{error_prefix}: {e}")))?;

        if !status.is_success() {
            return Err(AppError::Internal(format!(
                "{error_prefix}: {} - {}",
                status, text
            )));
        }

        serde_json::from_str::<T>(&text)
            .map_err(|e| AppError::Internal(format!("{error_prefix}: {e}")))
    }

    async fn exec_json_request(
        &self,
        method: reqwest::Method,
        endpoint: KeycloakEndpoint<'_>,
        token: &str,
        body: &serde_json::Value,
    ) -> Result<(), AppError> {
        let response = self
            .request_with_body(method, endpoint)?
            .bearer_auth(token)
            .json(body)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Request failed: {e}")))?;

        let status = response.status();
        let text = response.text().await.unwrap_or_default();

        if !status.is_success() {
            return Err(AppError::Internal(format!(
                "Keycloak request failed: {} - {}",
                status, text
            )));
        }

        Ok(())
    }

    fn get(&self, endpoint: KeycloakEndpoint<'_>) -> Result<reqwest::RequestBuilder, AppError> {
        let url = self.build_endpoint_url(endpoint)?;
        Ok(self.http.get(url))
    }

    fn post(&self, endpoint: KeycloakEndpoint<'_>) -> Result<reqwest::RequestBuilder, AppError> {
        let url = self.build_endpoint_url(endpoint)?;
        Ok(self.http.post(url))
    }

    fn request_with_body(
        &self,
        method: reqwest::Method,
        endpoint: KeycloakEndpoint<'_>,
    ) -> Result<reqwest::RequestBuilder, AppError> {
        let url = self.build_endpoint_url(endpoint)?;
        Ok(self.http.request(method, url))
    }

    fn build_endpoint_url(&self, endpoint: KeycloakEndpoint<'_>) -> Result<reqwest::Url, AppError> {
        let path_segments = endpoint.segments()?;
        let mut url = self.base_url.clone();
        let mut segments = url
            .path_segments_mut()
            .map_err(|_| AppError::Internal("Invalid Keycloak base URL".into()))?;
        segments.pop_if_empty();

        for segment in path_segments {
            segments.push(segment);
        }

        drop(segments);

        Ok(url)
    }

    pub async fn toggle_realm_role(
        &self,
        keycloak_id: &str,
        new_role: &str,
    ) -> Result<(), AppError> {
        let token = self.fetch_admin_token().await?;
        self.sync_realm_role(&token, keycloak_id, new_role).await?;

        Ok(())
    }

    pub async fn update_password(
        &self,
        keycloak_id: &str,
        new_password: &str,
    ) -> Result<(), AppError> {
        let token = self.fetch_admin_token().await?;

        let body = serde_json::json!({
            "type": "password",
            "value": new_password,
            "temporary": false
        });

        self.exec_json_request(
            reqwest::Method::PUT,
            KeycloakEndpoint::ResetPassword {
                realm: self.realm.as_str(),
                user_id: keycloak_id,
            },
            &token,
            &body,
        )
        .await?;

        Ok(())
    }
}

fn validate_keycloak_segment(value: &str, label: &str) -> Result<(), AppError> {
    let trimmed = value.trim();

    if trimmed.is_empty()
        || trimmed != value
        || trimmed.contains('/')
        || trimmed.contains('?')
        || trimmed.contains('#')
        || trimmed.chars().any(char::is_control)
    {
        return Err(AppError::Internal(format!(
            "Invalid Keycloak path segment: {label}"
        )));
    }

    Ok(())
}

fn parse_keycloak_base_url(raw: &str) -> Option<reqwest::Url> {
    let mut url = reqwest::Url::parse(raw).ok()?;

    match url.scheme() {
        "http" | "https" => {}
        _ => return None,
    }

    if url.host_str().is_none() || !url.username().is_empty() || url.password().is_some() {
        return None;
    }

    url.set_query(None);
    url.set_fragment(None);

    let normalized_path = match url.path().trim_end_matches('/') {
        "" => "/".to_string(),
        path => format!("{path}/"),
    };
    url.set_path(&normalized_path);

    Some(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn service_with_base_url(raw: &str) -> KeycloakAdminService {
        KeycloakAdminService {
            base_url: parse_keycloak_base_url(raw).unwrap(),
            realm: "altair".to_string(),
            admin_realm: "master".to_string(),
            admin_client_id: "admin-cli".to_string(),
            admin_username: "admin".to_string(),
            admin_password: "secret".to_string(),
            sync_username: false,
            http: reqwest::Client::new(),
        }
    }

    #[test]
    fn parse_keycloak_base_url_keeps_only_safe_http_base() {
        let url = parse_keycloak_base_url("https://keycloak.local/auth?next=http://evil.test#frag")
            .unwrap();

        assert_eq!(url.as_str(), "https://keycloak.local/auth/");
    }

    #[test]
    fn parse_keycloak_base_url_rejects_unsafe_inputs() {
        assert!(parse_keycloak_base_url("file:///etc/passwd").is_none());
        assert!(parse_keycloak_base_url("https://user:keycloak@example.test").is_none());
        assert!(parse_keycloak_base_url("https://").is_none());
    }

    #[test]
    fn build_endpoint_url_encodes_static_path_segments() {
        let service = service_with_base_url("https://keycloak.local/auth");
        let url = service
            .build_endpoint_url(KeycloakEndpoint::RealmRoleMappings {
                realm: "altair",
                user_id: "00000000-0000-4000-8000-000000000000",
            })
            .unwrap();

        assert_eq!(
            url.as_str(),
            "https://keycloak.local/auth/admin/realms/altair/users/00000000-0000-4000-8000-000000000000/role-mappings/realm"
        );
    }

    #[test]
    fn build_endpoint_url_rejects_path_injection_segments() {
        let service = service_with_base_url("https://keycloak.local/auth");

        assert!(service
            .build_endpoint_url(KeycloakEndpoint::AdminUser {
                realm: "altair",
                user_id: "a/b",
            })
            .is_err());
        assert!(service
            .build_endpoint_url(KeycloakEndpoint::RealmRole {
                realm: "altair",
                role: "admin?redirect=http://evil.test",
            })
            .is_err());
    }
}
