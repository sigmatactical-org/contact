/// PostgreSQL connection URL (shared Sigma database).
#[must_use]
pub fn database_url() -> String {
    std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| sigma_pg::DEFAULT_DATABASE_URL.to_string())
}

/// OIDC issuer URL for the identity provider (Keycloak realm URL).
#[must_use]
pub fn identity_issuer_url() -> Option<String> {
    std::env::var("CONTACT_IDENTITY_ISSUER_URL")
        .ok()
        .filter(|s| !s.trim().is_empty())
}

/// Service-account client id for Keycloak Admin API access.
#[must_use]
pub fn identity_client_id() -> Option<String> {
    std::env::var("CONTACT_IDENTITY_CLIENT_ID")
        .ok()
        .filter(|s| !s.trim().is_empty())
}

/// Service-account client secret for Keycloak Admin API access.
#[must_use]
pub fn identity_client_secret() -> Option<String> {
    std::env::var("CONTACT_IDENTITY_CLIENT_SECRET")
        .ok()
        .filter(|s| !s.trim().is_empty())
}

/// Whether identity sync is configured.
#[must_use]
pub fn identity_sync_configured() -> bool {
    identity_issuer_url().is_some()
        && identity_client_id().is_some()
        && identity_client_secret().is_some()
}
