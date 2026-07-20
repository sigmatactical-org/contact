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

fn with_store(
    store: SharedStore,
) -> impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone {
    warp::any().map(move || store.clone())
}

/// Site routes: web UI, JSON API, `/up`, theme static assets, error recovery,
/// and the shared security header set (CSP `connect-src` extended with the
/// identity BFF origin).
pub fn routes(
    store: store::ContactStore,
) -> impl Filter<Extract = (impl Reply,), Error = Infallible> + Clone + Send + 'static {
    let health_pool = Arc::new(store.pool().clone());
    let store = Arc::new(store);
    let human_check = sigma_human_check::HumanCheck::from_env();

    let index = sigma_human_check::warp::routes(human_check.clone())
        .or(public_contact::routes(
            with_store(store.clone()),
            human_check,
        ))
        .or(web::routes(with_store(store.clone())))
        .or(api::routes(with_store(store)));

    // `security_headers` captures the `&str` lifetime in its `'static` return
    // type, so the origin has to outlive the process; cache it once.
    static IDENTITY_ORIGIN: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    let identity_origin = IDENTITY_ORIGIN.get_or_init(config::identity_public_origin);

    sigma_theme::warp::security_headers(
        sigma_theme::warp::site_routes(
            index,
            sigma_pg::health::warp::health_routes("contact", Some(health_pool)),
        ),
        identity_origin,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use warp::http::StatusCode;

    async fn test_store() -> store::ContactStore {
        sigma_pg::clients::internal::ensure_test_internal_token();
        store::ContactStore::connect_empty()
            .await
            .expect("PostgreSQL required for tests")
    }

    #[tokio::test]
    async fn up_returns_ok() {
        let res = warp::test::request()
            .method("GET")
            .path("/up")
            .reply(&routes(test_store().await))
            .await;
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn index_lists_contacts() {
        let res = warp::test::request()
            .method("GET")
            .path("/")
            .reply(&routes(test_store().await))
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
            .reply(&routes(test_store().await))
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
            .reply(&routes(test_store().await))
            .await;
        assert_eq!(res.status(), StatusCode::CREATED);
        let contact: Contact = serde_json::from_slice(res.body()).unwrap();
        assert_eq!(contact.display_name, "Ada Lovelace");
        assert_eq!(contact.source, ContactSource::External);
    }
}
