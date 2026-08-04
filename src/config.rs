//! Environment-driven configuration for the contact service.
//!
//! Required values are declared in the [`sigma_config::service!`] block and
//! checked by [`validate_with`] at startup; optional integrations return
//! `None` when they are not configured for this environment.

sigma_config::service! {
    prefix = "CONTACT";
    role = "contact";
    urls {
        /// Public base URL of the identity BFF.
        identity_public_base_url = "IDENTITY_PUBLIC_URL" => "http://127.0.0.1:3000/";
        /// Public base URL of this contact service.
        public_base_url = "PUBLIC_BASE_URL" => "http://127.0.0.1:8083/";
        /// Public base URL of the cart service for navbar links.
        cart_public_base_url = "CART_PUBLIC_URL" => "http://127.0.0.1:8084/";
    }
}

/// OIDC issuer URL for the identity provider (Keycloak realm URL).
#[must_use]
pub fn identity_issuer_url() -> Option<String> {
    SERVICE.opt_str("IDENTITY_ISSUER_URL")
}

/// Service-account client id for Keycloak Admin API access.
#[must_use]
pub fn identity_client_id() -> Option<String> {
    SERVICE.opt_str("IDENTITY_CLIENT_ID")
}

/// Service-account client secret for Keycloak Admin API access.
#[must_use]
pub fn identity_client_secret() -> Option<String> {
    SERVICE.opt_str("IDENTITY_CLIENT_SECRET")
}

/// Whether identity sync is configured.
#[must_use]
pub fn identity_sync_configured() -> bool {
    identity_issuer_url().is_some()
        && identity_client_id().is_some()
        && identity_client_secret().is_some()
}

/// Browser origin of the identity BFF for CSP `connect-src` (no trailing slash).
#[must_use]
pub fn identity_public_origin() -> String {
    sigma_config::origin_of(&identity_public_base_url())
}

/// Base URL for server-to-server calls to the identity BFF (e.g. session
/// status checks on contact form submit). Must be reachable from this pod,
/// unlike `identity_public_base_url`, which is the browser-facing ingress
/// host and does not resolve back to identity from inside the cluster
/// network. Falls back to the public URL for non-cluster local dev.
#[must_use]
pub fn identity_internal_base_url() -> String {
    SERVICE
        .opt_url("IDENTITY_INTERNAL_URL")
        .unwrap_or_else(identity_public_base_url)
}

/// Allowed `return_url` values for the public `/contact` form.
#[must_use]
pub fn return_uris() -> Vec<String> {
    SERVICE
        .opt_str("RETURN_URIS")
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|entry| !entry.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

/// PostgreSQL connection URL (shared Sigma database).
#[must_use]
pub fn database_url() -> String {
    SERVICE.database_url()
}
