use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::model::{Contact, ContactSource, CreateContact, UpdateContact};

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("contact not found")]
    NotFound,
    #[error("identity contacts cannot be modified")]
    IdentityReadOnly,
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
struct Database {
    contacts: Vec<Contact>,
}

#[derive(Debug, Clone)]
pub struct ContactStore {
    path: PathBuf,
    db: Database,
}

impl ContactStore {
    /// Load or initialize the contact database at `path`.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let path = path.as_ref().to_path_buf();
        let db = if path.exists() {
            let bytes = std::fs::read(&path)?;
            serde_json::from_slice(&bytes)?
        } else {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            Database::default()
        };
        Ok(Self { path, db })
    }

    fn save(&self) -> Result<(), StoreError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let bytes = serde_json::to_vec_pretty(&self.db)?;
        std::fs::write(&self.path, bytes)?;
        Ok(())
    }

    #[must_use]
    pub fn list(&self) -> Vec<Contact> {
        let mut contacts = self.db.contacts.clone();
        contacts.sort_by(|a, b| {
            a.display_name
                .to_lowercase()
                .cmp(&b.display_name.to_lowercase())
        });
        contacts
    }

    #[must_use]
    pub fn get(&self, id: &str) -> Option<Contact> {
        self.db.contacts.iter().find(|c| c.id == id).cloned()
    }

    pub fn create_external(&mut self, input: CreateContact) -> Result<Contact, StoreError> {
        if input.display_name.trim().is_empty() {
            return Err(StoreError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "display_name is required",
            )));
        }
        let contact = Contact::new_external(input);
        self.db.contacts.push(contact.clone());
        self.save()?;
        Ok(contact)
    }

    pub fn update_external(
        &mut self,
        id: &str,
        input: UpdateContact,
    ) -> Result<Contact, StoreError> {
        let contact = self
            .db
            .contacts
            .iter_mut()
            .find(|c| c.id == id)
            .ok_or(StoreError::NotFound)?;
        if contact.source != ContactSource::External {
            return Err(StoreError::IdentityReadOnly);
        }
        if input.display_name.trim().is_empty() {
            return Err(StoreError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "display_name is required",
            )));
        }
        contact.apply_update(input);
        let updated = contact.clone();
        self.save()?;
        Ok(updated)
    }

    pub fn delete_external(&mut self, id: &str) -> Result<(), StoreError> {
        let index = self
            .db
            .contacts
            .iter()
            .position(|c| c.id == id)
            .ok_or(StoreError::NotFound)?;
        if self.db.contacts[index].source != ContactSource::External {
            return Err(StoreError::IdentityReadOnly);
        }
        self.db.contacts.remove(index);
        self.save()
    }

    /// Merge identity-sourced contacts; external entries are preserved.
    pub fn merge_identity(&mut self, identity_contacts: Vec<Contact>) -> Result<usize, StoreError> {
        self.db
            .contacts
            .retain(|c| c.source != ContactSource::Identity);

        let count = identity_contacts.len();
        self.db.contacts.extend(identity_contacts);
        self.save()?;
        Ok(count)
    }
}
