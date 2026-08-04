//! Sigma Contact: identity directory sync and external contact management.

#![forbid(unsafe_code)]

mod allowlist;
mod api;
pub mod config;
mod identity;
mod model;
mod public_contact;
mod session_status;
pub mod store;
mod templates;
mod web;

use std::convert::Infallible;
use std::sync::Arc;

use warp::Filter;
use warp::Reply;

pub use model::{Contact, ContactSource};

/// Shared contact store handle (`PgPool` is internally concurrent).
pub type SharedStore = Arc<store::ContactStore>;

/// Site routes: web UI, JSON API, `/up`, theme static assets, error recovery,
/// and the shared security header set (CSP `connect-src` extended with the
/// identity BFF origin).
///
/// `human_check` guards the public contact form and comes from the caller so
/// that a missing secret fails startup in `main` rather than here, where the
/// only options would be to panic or to accept unverified submissions.
pub fn routes(
    store: store::ContactStore,
    human_check: sigma_human_check::HumanCheck,
) -> impl Filter<Extract = (impl Reply,), Error = Infallible> + Clone + Send + 'static {
    let health_pool = Arc::new(store.pool().clone());
    let store = Arc::new(store);

    let index = sigma_human_check::warp::routes(human_check.clone())
        .or(public_contact::routes(
            sigma_theme::warp::with_state(store.clone()),
            human_check,
        ))
        .or(web::routes(sigma_theme::warp::with_state(store.clone())))
        .or(api::routes(sigma_theme::warp::with_state(store)));

    sigma_theme::warp::security_headers(
        sigma_theme::warp::site_routes(
            index,
            sigma_pg::health::warp::health_routes("contact", Some(health_pool)),
        ),
        config::identity_public_origin(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use warp::http::StatusCode;

    async fn test_store() -> store::ContactStore {
        sigma_pg::test_helpers::ready_store(store::ContactStore::connect_empty()).await
    }

    /// Routes over an empty store, with the bot check off so the tests exercise
    /// pages and the API rather than proof-of-work.
    async fn test_routes()
    -> impl Filter<Extract = (impl Reply,), Error = Infallible> + Clone + Send + 'static {
        routes(
            test_store().await,
            sigma_human_check::HumanCheck::disabled(),
        )
    }

    #[tokio::test]
    async fn up_returns_ok() {
        let res = warp::test::request()
            .method("GET")
            .path("/up")
            .reply(&test_routes().await)
            .await;
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn index_lists_contacts() {
        let res = warp::test::request()
            .method("GET")
            .path("/")
            .reply(&test_routes().await)
            .await;
        assert_eq!(res.status(), StatusCode::OK);
        let body = std::str::from_utf8(res.body()).unwrap();
        assert!(body.contains("Contacts"));
    }

    #[tokio::test]
    async fn api_lists_empty_contacts() {
        let res = warp::test::request()
            .method("GET")
            .path("/contacts")
            .header("accept", "application/json")
            .header(
                "x-sigma-internal-token",
                sigma_pg::clients::internal::TEST_INTERNAL_TOKEN,
            )
            .reply(&test_routes().await)
            .await;
        assert_eq!(res.status(), StatusCode::OK);
        let body: Vec<Contact> = serde_json::from_slice(res.body()).unwrap();
        assert!(body.is_empty());
    }

    #[tokio::test]
    async fn api_create_external_contact() {
        let res = warp::test::request()
            .method("POST")
            .path("/contacts")
            .header("content-type", "application/json")
            .header("x-sigma-internal-token", sigma_pg::clients::internal::TEST_INTERNAL_TOKEN)
            .body(
                r#"{"display_name":"Ada Lovelace","email":"ada@example.com","phone":null,"notes":null}"#,
            )
            .reply(&test_routes().await)
            .await;
        assert_eq!(res.status(), StatusCode::CREATED);
        let contact: Contact = serde_json::from_slice(res.body()).unwrap();
        assert_eq!(contact.display_name, "Ada Lovelace");
        assert_eq!(contact.source, ContactSource::External);
    }
}
