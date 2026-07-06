use sqlx::PgPool;
use thiserror::Error;

use crate::model::{Contact, ContactSource, CreateContact, UpdateContact};

const SCHEMA: &str = "contact";

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("contact not found")]
    NotFound,
    #[error("identity contacts cannot be modified")]
    IdentityReadOnly,
    #[error("database error: {0}")]
    Database(#[from] anyhow::Error),
    #[error("{0}")]
    InvalidInput(String),
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
struct Database {
    contacts: Vec<Contact>,
}

#[derive(Debug, Clone)]
pub struct ContactStore {
    pool: PgPool,
    db: Database,
}

impl ContactStore {
    /// Connect to PostgreSQL and load the contact snapshot.
    pub async fn connect() -> Result<Self, StoreError> {
        let pool = sigma_pg::connect().await?;
        let db: Database = sigma_pg::load_document(&pool, SCHEMA).await?;
        Ok(Self { pool, db })
    }

    /// Reset the contact snapshot (tests only).
    #[cfg(test)]
    pub async fn connect_empty() -> Result<Self, StoreError> {
        let pool = sigma_pg::connect().await?;
        let db = Database::default();
        sigma_pg::save_document(&pool, SCHEMA, &db).await?;
        Ok(Self { pool, db })
    }

    async fn persist(&self) -> Result<(), StoreError> {
        sigma_pg::save_document(&self.pool, SCHEMA, &self.db).await?;
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

    pub async fn create_external(&mut self, input: CreateContact) -> Result<Contact, StoreError> {
        if input.display_name.trim().is_empty() {
            return Err(StoreError::InvalidInput(
                "display_name is required".to_string(),
            ));
        }
        let contact = Contact::new_external(input);
        self.db.contacts.push(contact.clone());
        self.persist().await?;
        Ok(contact)
    }

    pub async fn update_external(
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
            return Err(StoreError::InvalidInput(
                "display_name is required".to_string(),
            ));
        }
        contact.apply_update(input);
        let updated = contact.clone();
        self.persist().await?;
        Ok(updated)
    }

    pub async fn delete_external(&mut self, id: &str) -> Result<(), StoreError> {
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
        self.persist().await
    }

    /// Merge identity-sourced contacts; external entries are preserved.
    pub async fn merge_identity(
        &mut self,
        identity_contacts: Vec<Contact>,
    ) -> Result<usize, StoreError> {
        self.db
            .contacts
            .retain(|c| c.source != ContactSource::Identity);

        let count = identity_contacts.len();
        self.db.contacts.extend(identity_contacts);
        self.persist().await?;
        Ok(count)
    }
}
