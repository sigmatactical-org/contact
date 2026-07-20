//! [`ContactQuery`].

/// Query string for the public contact form and success pages.
#[derive(Debug, serde::Deserialize)]
pub(crate) struct ContactQuery {
    #[serde(default)]
    pub(crate) return_url: String,
}
