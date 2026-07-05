use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct IdentityStatus {
    pub(crate) authenticated: bool,
    pub(crate) username: Option<String>,
    pub(crate) email: Option<String>,
}

/// Resolve the signed-in user from the identity BFF using browser session cookies.
pub(crate) async fn fetch_identity_status(cookie_header: Option<&str>) -> Option<IdentityStatus> {
    let cookie_header = cookie_header.filter(|value| !value.trim().is_empty())?;
    let url = format!(
        "{}auth/status",
        crate::config::identity_public_base_url().trim_end_matches('/')
    );

    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header("cookie", cookie_header)
        .send()
        .await
        .ok()?;
    if !response.status().is_success() {
        return None;
    }

    response.json::<IdentityStatus>().await.ok()
}
