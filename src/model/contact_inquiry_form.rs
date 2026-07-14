//! [`ContactInquiryForm`].

#[allow(unused_imports)]
use super::*;
use serde::Deserialize;

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
    /// Check required fields and lengths for a public inquiry.
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
