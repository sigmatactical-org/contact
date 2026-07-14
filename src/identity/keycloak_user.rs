//! [`KeycloakUser`].

#[allow(unused_imports)]
use super::*;

#[derive(serde::Deserialize)]
pub(crate) struct KeycloakUser {
    pub(crate) id: String,
    pub(crate) username: Option<String>,
    pub(crate) email: Option<String>,
    pub(crate) first_name: Option<String>,
    pub(crate) last_name: Option<String>,
    pub(crate) enabled: Option<bool>,
}
