use crate::error::AppError;
use axum::http::HeaderMap;
use uuid::Uuid;

#[derive(Debug)]
pub struct Caller {
    pub user_id: Uuid,
    pub roles: Vec<String>,
}

fn normalize_role(raw: &str) -> Option<String> {
    let mut role = raw.trim().to_lowercase();
    if role.is_empty() {
        return None;
    }

    if let Some(stripped) = role.strip_prefix("role_") {
        role = stripped.to_string();
    }
    if let Some(stripped) = role.strip_prefix("realm:") {
        role = stripped.to_string();
    }
    if let Some(stripped) = role.strip_prefix("client:") {
        role = stripped.rsplit(':').next().unwrap_or(stripped).to_string();
    }

    Some(role)
}

pub fn parse_roles_csv(raw: &str) -> Vec<String> {
    raw.split(',').filter_map(normalize_role).collect()
}

pub fn extract_caller(headers: &HeaderMap) -> Result<Caller, AppError> {
    let user_id = headers
        .get("x-altair-user-id")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| AppError::Unauthorized("Missing caller identity".to_string()))?;

    let roles = headers
        .get("x-altair-roles")
        .and_then(|h| h.to_str().ok())
        .map(parse_roles_csv)
        .unwrap_or_default();

    Ok(Caller { user_id, roles })
}

#[cfg(test)]
mod tests {
    use super::parse_roles_csv;

    #[test]
    fn parse_roles_csv_normalizes_values() {
        let roles = parse_roles_csv(" ADMIN, role_creator, realm:learner, client:altair:admin ");
        assert_eq!(roles, vec!["admin", "creator", "learner", "admin"]);
    }
}
