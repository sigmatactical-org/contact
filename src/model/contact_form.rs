//! [`ContactForm`].

use serde::Deserialize;
use sigma_pg::form::empty_to_none;

use super::{CreateContact, UpdateContact};

#[derive(Debug, Clone, Deserialize)]
pub struct ContactForm {
    pub display_name: String,
    pub email: String,
    pub phone: String,
    pub notes: String,
}
impl ContactForm {
    #[must_use]
    pub fn into_create(self) -> CreateContact {
        CreateContact {
            display_name: self.display_name,
            email: empty_to_none(self.email),
            phone: empty_to_none(self.phone),
            notes: empty_to_none(self.notes),
        }
    }

    #[must_use]
    pub fn into_update(self) -> UpdateContact {
        UpdateContact {
            display_name: self.display_name,
            email: empty_to_none(self.email),
            phone: empty_to_none(self.phone),
            notes: empty_to_none(self.notes),
        }
    }
}
