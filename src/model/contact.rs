//! [`Contact`].

#[allow(unused_imports)]
use super::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Contact {
    pub id: String,
    pub source: ContactSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity_id: Option<String>,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    pub updated_at: String,
}
impl Contact {
    /// New externally-sourced contact (web inquiry).
    pub fn new_external(input: CreateContact) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            source: ContactSource::External,
            identity_id: None,
            display_name: input.display_name.trim().to_string(),
            email: input.email.map(|s| s.trim().to_string()),
            phone: input.phone.map(|s| s.trim().to_string()),
            notes: input.notes.map(|s| s.trim().to_string()),
            updated_at: now,
        }
    }

    /// Contact synced from an identity-service user.
    pub fn from_identity(identity_id: String, display_name: String, email: Option<String>) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: format!("identity:{identity_id}"),
            source: ContactSource::Identity,
            identity_id: Some(identity_id),
            display_name,
            email,
            phone: None,
            notes: None,
            updated_at: now,
        }
    }

    /// Apply a partial update in place.
    pub fn apply_update(&mut self, input: UpdateContact) {
        self.display_name = input.display_name.trim().to_string();
        self.email = input.email.map(|s| s.trim().to_string());
        self.phone = input.phone.map(|s| s.trim().to_string());
        self.notes = input.notes.map(|s| s.trim().to_string());
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }
}
