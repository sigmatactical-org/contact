//! [`TokenResponse`].

#[allow(unused_imports)]
use super::*;

#[derive(serde::Deserialize)]
pub(crate) struct TokenResponse {
    pub(crate) access_token: String,
}
