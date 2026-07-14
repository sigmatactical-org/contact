//! [`UpdateContact`].

#[allow(unused_imports)]
use super::*;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateContact {
    pub display_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub notes: Option<String>,
}
