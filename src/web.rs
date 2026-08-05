use std::convert::Infallible;

use sigma_theme::warp::internal_rejection;
use warp::http::StatusCode;
use warp::reply::Response;
use warp::{Filter, Rejection, Reply};

use crate::SharedStore;
use crate::config;
use crate::identity;
use crate::model::{Contact, ContactForm, ContactSource};
use crate::session_status;
use crate::store::StoreError;
use crate::templates;

/// Outcome of the admin session gate for HTML admin routes.
enum AdminGate {
    Allow,
    SignIn(Response),
    /// Signed in but not an admin — hide the admin surface.
    Deny,
}

/// Require an admin identity session. Anonymous users are sent to sign-in;
/// signed-in non-admins get a 404 so the directory and API docs stay private.
async fn require_admin(cookie: Option<&str>, return_path: &str) -> AdminGate {
    match session_status::fetch_identity_status(cookie).await {
        Some(status) if status.is_admin => AdminGate::Allow,
        Some(_) => AdminGate::Deny,
        None => AdminGate::SignIn(sign_in_redirect(return_path)),
    }
}

fn sign_in_redirect(return_path: &str) -> Response {
    let links = sigma_identity_nav::auth_links(
        &config::identity_public_base_url(),
        &config::public_base_url(),
        return_path,
    );
    match links.sign_in_url.parse::<warp::http::Uri>() {
        Ok(uri) => warp::redirect::see_other(uri).into_response(),
        Err(_) => warp::reply::with_status(
            warp::reply::html(sigma_theme::errors::internal_server_error_html()),
            StatusCode::INTERNAL_SERVER_ERROR,
        )
        .into_response(),
    }
}

fn cookie_filter() -> impl Filter<Extract = (Option<String>,), Error = Rejection> + Clone {
    warp::header::optional::<String>("cookie")
}

/// Build this module's routes.
pub fn routes(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    index_page(store.clone())
        .or(new_contact_page())
        .or(create_contact_form(store.clone()))
        .or(edit_contact_page(store.clone()))
        .or(update_contact_form(store.clone()))
        .or(delete_contact_form(store.clone()))
        .or(sync_contacts_form(store))
}

fn index_page(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path::end()
        .and(warp::get())
        .and(cookie_filter())
        .and(store)
        .and_then(|cookie: Option<String>, store: SharedStore| async move {
            match require_admin(cookie.as_deref(), "/").await {
                AdminGate::Allow => {}
                AdminGate::SignIn(resp) => return Ok::<_, Rejection>(resp),
                AdminGate::Deny => return Err(warp::reject::not_found()),
            }
            let contacts = store
                .list()
                .await
                .map_err(|e| internal_rejection("list contacts", e))?;
            templates::render_index_html(contacts, crate::config::identity_sync_configured(), None)
                .map(|html| warp::reply::html(html).into_response())
                .map_err(|e| internal_rejection("render contact index", e))
        })
}

fn new_contact_page()
-> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path("contacts")
        .and(warp::path("new"))
        .and(warp::path::end())
        .and(warp::get())
        .and(cookie_filter())
        .and_then(|cookie: Option<String>| async move {
            match require_admin(cookie.as_deref(), "/contacts/new").await {
                AdminGate::Allow => {}
                AdminGate::SignIn(resp) => return Ok::<_, Rejection>(resp),
                AdminGate::Deny => return Err(warp::reject::not_found()),
            }
            templates::render_form_html(None, None)
                .map(|html| warp::reply::html(html).into_response())
                .map_err(|e| internal_rejection("render contact form", e))
        })
}

fn create_contact_form(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path("contacts")
        .and(warp::path::end())
        .and(warp::post())
        .and(cookie_filter())
        .and(warp::body::form())
        .and(store)
        .and_then(
            |cookie: Option<String>, form: ContactForm, store: SharedStore| async move {
                match require_admin(cookie.as_deref(), "/contacts/new").await {
                    AdminGate::Allow => {}
                    AdminGate::SignIn(resp) => return Ok::<_, Rejection>(resp),
                    AdminGate::Deny => return Err(warp::reject::not_found()),
                }
                let response = match store.create_external(form.into_create()).await {
                    Ok(_) => {
                        warp::redirect::redirect(warp::http::Uri::from_static("/")).into_response()
                    }
                    Err(StoreError::InvalidInput(_)) => {
                        match templates::render_form_html(
                            None,
                            Some("Display name is required.".to_string()),
                        ) {
                            Ok(html) => warp::reply::with_status(
                                warp::reply::html(html),
                                StatusCode::BAD_REQUEST,
                            )
                            .into_response(),
                            Err(_) => return Err(warp::reject::not_found()),
                        }
                    }
                    Err(_) => return Err(warp::reject::not_found()),
                };
                Ok(response)
            },
        )
}

fn edit_contact_page(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path!("contacts" / String / "edit")
        .and(warp::get())
        .and(cookie_filter())
        .and(store)
        .and_then(|id: String, cookie: Option<String>, store: SharedStore| async move {
            let return_path = format!("/contacts/{id}/edit");
            match require_admin(cookie.as_deref(), &return_path).await {
                AdminGate::Allow => {}
                AdminGate::SignIn(resp) => return Ok::<_, Rejection>(resp),
                AdminGate::Deny => return Err(warp::reject::not_found()),
            }
            let Some(contact) = store
                .get(&id)
                .await
                .map_err(|e| internal_rejection("read contact", e))?
            else {
                return Err(warp::reject::not_found());
            };
            if contact.source != ContactSource::External {
                return Err(warp::reject::not_found());
            }
            templates::render_form_html(Some(contact), None)
                .map(|html| warp::reply::html(html).into_response())
                .map_err(|e| internal_rejection("render contact form", e))
        })
}

fn update_contact_form(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path!("contacts" / String / "edit")
        .and(warp::post())
        .and(cookie_filter())
        .and(warp::body::form())
        .and(store)
        .and_then(
            |id: String, cookie: Option<String>, form: ContactForm, store: SharedStore| async move {
                let return_path = format!("/contacts/{id}/edit");
                match require_admin(cookie.as_deref(), &return_path).await {
                    AdminGate::Allow => {}
                    AdminGate::SignIn(resp) => return Ok::<_, Rejection>(resp),
                    AdminGate::Deny => return Err(warp::reject::not_found()),
                }
                let response = match store.update_external(&id, form.clone().into_update()).await {
                    Ok(_) => {
                        warp::redirect::redirect(warp::http::Uri::from_static("/")).into_response()
                    }
                    Err(StoreError::NotFound) => return Err(warp::reject::not_found()),
                    Err(StoreError::IdentityReadOnly) => return Err(warp::reject::not_found()),
                    Err(StoreError::InvalidInput(_)) => {
                        // Re-render the edit form from the submitted values
                        // (`InvalidInput` implies the contact exists and is
                        // external) instead of re-querying the store.
                        let mut contact = Contact::new_external(form.into_create());
                        contact.id = id;
                        match templates::render_form_html(
                            Some(contact),
                            Some("Display name is required.".to_string()),
                        ) {
                            Ok(html) => warp::reply::with_status(
                                warp::reply::html(html),
                                StatusCode::BAD_REQUEST,
                            )
                            .into_response(),
                            Err(_) => return Err(warp::reject::not_found()),
                        }
                    }
                    Err(_) => return Err(warp::reject::not_found()),
                };
                Ok(response)
            },
        )
}

fn delete_contact_form(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path!("contacts" / String / "delete")
        .and(warp::post())
        .and(cookie_filter())
        .and(store)
        .and_then(|id: String, cookie: Option<String>, store: SharedStore| async move {
            match require_admin(cookie.as_deref(), "/").await {
                AdminGate::Allow => {}
                AdminGate::SignIn(resp) => return Ok::<_, Rejection>(resp),
                AdminGate::Deny => return Err(warp::reject::not_found()),
            }
            match store.delete_external(&id).await {
                Ok(()) => Ok(warp::redirect::redirect(warp::http::Uri::from_static("/"))
                    .into_response()),
                Err(StoreError::NotFound) | Err(StoreError::IdentityReadOnly) => {
                    Err(warp::reject::not_found())
                }
                Err(_) => Err(warp::reject::not_found()),
            }
        })
}

fn sync_contacts_form(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path("contacts")
        .and(warp::path("sync"))
        .and(warp::path::end())
        .and(warp::post())
        .and(cookie_filter())
        .and(store)
        .and_then(|cookie: Option<String>, store: SharedStore| async move {
            match require_admin(cookie.as_deref(), "/").await {
                AdminGate::Allow => {}
                AdminGate::SignIn(resp) => return Ok::<_, Rejection>(resp),
                AdminGate::Deny => return Err(warp::reject::not_found()),
            }
            let sync_result = identity::fetch_identity_contacts().await;
            let message = match sync_result {
                Ok(identity_contacts) => match store.merge_identity(identity_contacts).await {
                    Ok(count) => Some(format!("Synced {count} identity contact(s).")),
                    Err(e) => Some(format!("Sync failed: {e}")),
                },
                Err(e) => Some(format!("Sync failed: {e}")),
            };
            let contacts = store
                .list()
                .await
                .map_err(|e| internal_rejection("list contacts", e))?;
            templates::render_index_html(
                contacts,
                crate::config::identity_sync_configured(),
                message,
            )
            .map(|html| warp::reply::html(html).into_response())
            .map_err(|e| internal_rejection("render contact index", e))
        })
}
