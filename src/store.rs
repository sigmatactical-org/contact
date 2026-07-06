use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use thiserror::Error;

use crate::model::{Contact, ContactSource, CreateContact, UpdateContact};

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

impl From<sqlx::Error> for StoreError {
    fn from(err: sqlx::Error) -> Self {
        Self::Database(err.into())
    }
}

#[derive(Debug, Clone)]
pub struct ContactStore {
    pool: PgPool,
}

impl ContactStore {
    pub async fn connect() -> Result<Self, StoreError> {
        let pool = sigma_pg::connect_as("contact").await?;
        Ok(Self { pool })
    }

    #[cfg(test)]
    pub async fn connect_empty() -> Result<Self, StoreError> {
        let store = Self::connect().await?;
        sqlx::query("TRUNCATE contact.contacts")
            .execute(&store.pool)
            .await?;
        Ok(store)
    }

    #[must_use]
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn list(&self) -> Result<Vec<Contact>, StoreError> {
        let rows = sqlx::query(
            "SELECT id, source, identity_id, display_name, email, phone, notes, updated_at \
             FROM contact.contacts ORDER BY lower(display_name)",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_contact).collect()
    }

    pub async fn get(&self, id: &str) -> Result<Option<Contact>, StoreError> {
        let row = sqlx::query(
            "SELECT id, source, identity_id, display_name, email, phone, notes, updated_at \
             FROM contact.contacts WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        row.map(row_to_contact).transpose()
    }

    pub async fn create_external(&self, input: CreateContact) -> Result<Contact, StoreError> {
        if input.display_name.trim().is_empty() {
            return Err(StoreError::InvalidInput(
                "display_name is required".to_string(),
            ));
        }
        let contact = Contact::new_external(input);
        insert_contact(&self.pool, &contact).await?;
        Ok(contact)
    }

    pub async fn update_external(
        &self,
        id: &str,
        input: UpdateContact,
    ) -> Result<Contact, StoreError> {
        let mut contact = self.get(id).await?.ok_or(StoreError::NotFound)?;
        if contact.source != ContactSource::External {
            return Err(StoreError::IdentityReadOnly);
        }
        if input.display_name.trim().is_empty() {
            return Err(StoreError::InvalidInput(
                "display_name is required".to_string(),
            ));
        }
        contact.apply_update(input);
        sqlx::query(
            "UPDATE contact.contacts SET display_name = $2, email = $3, phone = $4, notes = $5, \
             updated_at = $6 WHERE id = $1",
        )
        .bind(&contact.id)
        .bind(&contact.display_name)
        .bind(&contact.email)
        .bind(&contact.phone)
        .bind(&contact.notes)
        .bind(parse_ts(&contact.updated_at)?)
        .execute(&self.pool)
        .await?;
        Ok(contact)
    }

    pub async fn delete_external(&self, id: &str) -> Result<(), StoreError> {
        let contact = self.get(id).await?.ok_or(StoreError::NotFound)?;
        if contact.source != ContactSource::External {
            return Err(StoreError::IdentityReadOnly);
        }
        sqlx::query("DELETE FROM contact.contacts WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn merge_identity(
        &self,
        identity_contacts: Vec<Contact>,
    ) -> Result<usize, StoreError> {
        let count = identity_contacts.len();
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM contact.contacts WHERE source = 'identity'")
            .execute(&mut *tx)
            .await?;
        for contact in identity_contacts {
            insert_contact_tx(&mut tx, &contact).await?;
        }
        tx.commit().await?;
        Ok(count)
    }
}

async fn insert_contact(pool: &PgPool, contact: &Contact) -> Result<(), StoreError> {
    sqlx::query(
        "INSERT INTO contact.contacts \
         (id, source, identity_id, display_name, email, phone, notes, updated_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(&contact.id)
    .bind(source_str(contact.source))
    .bind(&contact.identity_id)
    .bind(&contact.display_name)
    .bind(&contact.email)
    .bind(&contact.phone)
    .bind(&contact.notes)
    .bind(parse_ts(&contact.updated_at)?)
    .execute(pool)
    .await?;
    Ok(())
}

async fn insert_contact_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    contact: &Contact,
) -> Result<(), StoreError> {
    sqlx::query(
        "INSERT INTO contact.contacts \
         (id, source, identity_id, display_name, email, phone, notes, updated_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(&contact.id)
    .bind(source_str(contact.source))
    .bind(&contact.identity_id)
    .bind(&contact.display_name)
    .bind(&contact.email)
    .bind(&contact.phone)
    .bind(&contact.notes)
    .bind(parse_ts(&contact.updated_at)?)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

fn row_to_contact(row: sqlx::postgres::PgRow) -> Result<Contact, StoreError> {
    let source_str: String = row.get("source");
    Ok(Contact {
        id: row.get("id"),
        source: parse_source(&source_str),
        identity_id: row.get("identity_id"),
        display_name: row.get("display_name"),
        email: row.get("email"),
        phone: row.get("phone"),
        notes: row.get("notes"),
        updated_at: row.get::<DateTime<Utc>, _>("updated_at").to_rfc3339(),
    })
}

fn source_str(source: ContactSource) -> &'static str {
    match source {
        ContactSource::Identity => "identity",
        ContactSource::External => "external",
    }
}

fn parse_source(value: &str) -> ContactSource {
    match value {
        "identity" => ContactSource::Identity,
        _ => ContactSource::External,
    }
}

fn parse_ts(value: &str) -> Result<DateTime<Utc>, StoreError> {
    value
        .parse::<DateTime<Utc>>()
        .map_err(|e| StoreError::InvalidInput(format!("invalid timestamp: {e}")))
}
