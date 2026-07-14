//! [`IdentityError`].

#[allow(unused_imports)]
use super::*;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IdentityError {
    #[error(
        "identity sync is not configured (set CONTACT_IDENTITY_ISSUER_URL, CONTACT_IDENTITY_CLIENT_ID, CONTACT_IDENTITY_CLIENT_SECRET)"
    )]
    NotConfigured,
    #[error("invalid issuer URL: {0}")]
    InvalidIssuer(String),
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Keycloak token request failed: {0}")]
    Token(String),
    #[error("Keycloak user listing failed: {0}")]
    Users(String),
}
