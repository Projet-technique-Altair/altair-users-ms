use serde::Deserialize;

use crate::error::AppError;

#[derive(Debug, Deserialize)]
pub struct UpdateMePayload {
    pub pseudo: Option<String>,
    pub email: Option<String>,
    pub role: Option<String>,
}

#[derive(Debug)]
pub struct UpdateMeInput {
    pub pseudo: Option<String>,
    pub email: Option<String>,
    pub role: Option<String>,
}

pub fn build_update_input(payload: UpdateMePayload) -> Result<UpdateMeInput, AppError> {
    // Trim and discard empty values so "  " is treated as not provided.
    let pseudo = payload
        .pseudo
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned);

    let email = payload
        .email
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned);

    let role = payload
        .role
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|raw| raw.to_ascii_lowercase());

    // Prevent no-op update requests.
    if pseudo.is_none() && email.is_none() && role.is_none() {
        return Err(AppError::Conflict(
            "No updatable field provided (pseudo, email, role)".to_string(),
        ));
    }

    // Keep role vocabulary explicit and bounded.
    if let Some(ref role_value) = role {
        let is_supported = matches!(role_value.as_str(), "admin" | "creator" | "learner");
        if !is_supported {
            return Err(AppError::Conflict(
                "Unsupported role. Allowed: admin, creator, learner".to_string(),
            ));
        }
    }

    Ok(UpdateMeInput {
        pseudo,
        email,
        role,
    })
}
