use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContactSource {
    Identity,
    External,
}

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

#[derive(Debug, Clone, Deserialize)]
pub struct CreateContact {
    pub display_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateContact {
    pub display_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContactForm {
    pub display_name: String,
    pub email: String,
    pub phone: String,
    pub notes: String,
}

/// Public website contact inquiry (includes return URL for cross-site flows).
#[derive(Debug, Clone, Deserialize)]
pub struct ContactInquiryForm {
    pub return_url: String,
    pub display_name: String,
    pub email: String,
    pub phone: String,
    pub message: String,
    #[serde(default)]
    pub altcha: String,
}

impl ContactInquiryForm {
    pub fn validate(&self) -> Result<(), String> {
        if self.display_name.trim().is_empty() {
            return Err("Name is required.".to_string());
        }
        if self.email.trim().is_empty() || !self.email.contains('@') {
            return Err("A valid email address is required.".to_string());
        }
        if self.message.trim().is_empty() {
            return Err("Message is required.".to_string());
        }
        Ok(())
    }
}

impl ContactForm {
    #[must_use]
    pub fn into_create(self) -> CreateContact {
        CreateContact {
            display_name: self.display_name,
            email: empty_to_none(self.email),
            phone: empty_to_none(self.phone),
            notes: empty_to_none(self.notes),
        }
    }

    #[must_use]
    pub fn into_update(self) -> UpdateContact {
        UpdateContact {
            display_name: self.display_name,
            email: empty_to_none(self.email),
            phone: empty_to_none(self.phone),
            notes: empty_to_none(self.notes),
        }
    }
}

fn empty_to_none(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

impl Contact {
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

    pub fn apply_update(&mut self, input: UpdateContact) {
        self.display_name = input.display_name.trim().to_string();
        self.email = input.email.map(|s| s.trim().to_string());
        self.phone = input.phone.map(|s| s.trim().to_string());
        self.notes = input.notes.map(|s| s.trim().to_string());
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }
}
