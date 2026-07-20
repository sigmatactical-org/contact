//! [`ContactStore`].

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};

use super::StoreError;
use crate::model::{Contact, ContactSource, CreateContact, UpdateContact};

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
        sigma_pg::assert_disposable_test_db(&store.pool).await;
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
        Ok(rows.into_iter().map(row_to_contact).collect())
    }

    pub async fn get(&self, id: &str) -> Result<Option<Contact>, StoreError> {
        let row = sqlx::query(
            "SELECT id, source, identity_id, display_name, email, phone, notes, updated_at \
             FROM contact.contacts WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(row_to_contact))
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
        .bind(contact.updated_at)
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

    /// Replace all identity-sourced contacts in one transaction (single
    /// `UNNEST` bulk insert instead of a round-trip per contact).
    pub async fn merge_identity(
        &self,
        identity_contacts: Vec<Contact>,
    ) -> Result<usize, StoreError> {
        let count = identity_contacts.len();
        let mut ids = Vec::with_capacity(count);
        let mut sources = Vec::with_capacity(count);
        let mut identity_ids = Vec::with_capacity(count);
        let mut display_names = Vec::with_capacity(count);
        let mut emails = Vec::with_capacity(count);
        let mut phones = Vec::with_capacity(count);
        let mut notes = Vec::with_capacity(count);
        let mut updated_ats = Vec::with_capacity(count);
        for contact in identity_contacts {
            ids.push(contact.id);
            sources.push(source_str(contact.source).to_string());
            identity_ids.push(contact.identity_id);
            display_names.push(contact.display_name);
            emails.push(contact.email);
            phones.push(contact.phone);
            notes.push(contact.notes);
            updated_ats.push(contact.updated_at);
        }

        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM contact.contacts WHERE source = 'identity'")
            .execute(&mut *tx)
            .await?;
        sqlx::query(
            "INSERT INTO contact.contacts \
             (id, source, identity_id, display_name, email, phone, notes, updated_at) \
             SELECT * FROM UNNEST($1::text[], $2::text[], $3::text[], $4::text[], \
             $5::text[], $6::text[], $7::text[], $8::timestamptz[])",
        )
        .bind(&ids)
        .bind(&sources)
        .bind(&identity_ids)
        .bind(&display_names)
        .bind(&emails)
        .bind(&phones)
        .bind(&notes)
        .bind(&updated_ats)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(count)
    }
}

async fn insert_contact<'e>(
    executor: impl sqlx::PgExecutor<'e>,
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
    .bind(contact.updated_at)
    .execute(executor)
    .await?;
    Ok(())
}

fn row_to_contact(row: sqlx::postgres::PgRow) -> Contact {
    let source_str: String = row.get("source");
    Contact {
        id: row.get("id"),
        source: parse_source(&source_str),
        identity_id: row.get("identity_id"),
        display_name: row.get("display_name"),
        email: row.get("email"),
        phone: row.get("phone"),
        notes: row.get("notes"),
        updated_at: row.get::<DateTime<Utc>, _>("updated_at"),
    }
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
