//! Sigma Contact: identity directory sync and external contact management.

mod allowlist;
mod api;
pub mod config;
mod human_check;
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

pub use model::{Contact, ContactSource, CreateContact, UpdateContact};

/// Shared contact store handle (`PgPool` is internally concurrent).
pub type SharedStore = Arc<store::ContactStore>;

/// Resolve listen address from **`PORT`** (default **8080**).
#[must_use]
pub fn listen_socket_addr_from_env() -> std::net::SocketAddr {
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8080);
    SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port)
}

fn with_store(
    store: SharedStore,
) -> impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone {
    warp::any().map(move || store.clone())
}

/// Site routes: web UI, JSON API, `/up`, theme static assets, error recovery.
pub fn routes(
    store: store::ContactStore,
) -> impl Filter<Extract = (impl Reply,), Error = Infallible> + Clone + Send + 'static {
    use warp::reply::with::header;

    let health_pool = Arc::new(store.pool().clone());
    let store = Arc::new(store);

    let content_security_policy = {
        let identity_origin = config::identity_public_origin();
        format!(
            "default-src 'self'; base-uri 'self'; object-src 'none'; frame-ancestors 'none'; \
             img-src 'self' data:; style-src 'self' 'unsafe-inline'; script-src 'self'; \
             font-src 'self'; connect-src 'self' {identity_origin}; form-action 'self'"
        )
    };

    let human_check = sigma_human_check::HumanCheck::from_env();

    warp::path("up")
        .and(warp::get())
        .map(|| warp::reply::with_status("up", warp::http::StatusCode::OK))
        .or(sigma_pg::health::warp::health_routes(
            "contact",
            Some(health_pool),
        ))
        .or(human_check::routes(human_check.clone()))
        .or(public_contact::routes(
            with_store(store.clone()),
            human_check,
        ))
        .or(web::routes(with_store(store.clone())))
        .or(api::routes(with_store(store)))
        .or(sigma_theme::warp::static_files())
        .or(sigma_theme::warp::favicon())
        .recover(sigma_theme::warp::handle_rejection)
        .with(header("content-security-policy", content_security_policy))
        .with(header("x-content-type-options", "nosniff"))
        .with(header("x-frame-options", "DENY"))
        .with(header("referrer-policy", "strict-origin-when-cross-origin"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use warp::http::StatusCode;

    async fn test_store() -> store::ContactStore {
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
