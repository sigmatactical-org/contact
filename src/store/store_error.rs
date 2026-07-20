//! [`StoreError`].

use thiserror::Error;

/// Contact store error. Unlike the shared `sigma_pg::api::StoreError` it has
/// an [`StoreError::IdentityReadOnly`] variant (403) for writes against
/// identity-sourced contacts.
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
