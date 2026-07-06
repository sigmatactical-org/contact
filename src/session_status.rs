pub use sigma_pg::clients::session::IdentityStatus;

/// Resolve the signed-in user from the identity BFF using browser session cookies.
pub(crate) async fn fetch_identity_status(cookie_header: Option<&str>) -> Option<IdentityStatus> {
    sigma_pg::clients::session::fetch_identity_status(
        &crate::config::identity_public_base_url(),
        cookie_header,
    )
    .await
    .ok()
    .flatten()
}
