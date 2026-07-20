//! Identity directory sync via the shared sigma-pg identity client.

pub use sigma_pg::clients::identity::IdentityError;

use crate::config;
use crate::model::Contact;

/// Pull enabled realm users from the identity provider as [`Contact`]s.
pub async fn fetch_identity_contacts() -> Result<Vec<Contact>, IdentityError> {
    let users = sigma_pg::clients::identity::fetch_users(
        config::identity_issuer_url().as_deref(),
        config::identity_client_id().as_deref(),
        config::identity_client_secret().as_deref(),
    )
    .await?;
    Ok(users
        .into_iter()
        .map(|user| Contact::from_identity(user.id, user.display_name, user.email))
        .collect())
}
