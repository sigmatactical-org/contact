//! [`CreateContact`].

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct CreateContact {
    pub display_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub notes: Option<String>,
}
