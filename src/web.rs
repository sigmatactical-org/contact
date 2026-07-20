use std::convert::Infallible;

use warp::http::StatusCode;
use warp::{Filter, Rejection, Reply};

use crate::SharedStore;
use crate::identity;
use crate::model::{Contact, ContactForm, ContactSource};
use crate::store::StoreError;
use crate::templates;

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
        .and(store)
        .and_then(|store: SharedStore| async move {
            let contacts = store.list().await.map_err(|_| warp::reject::not_found())?;
            templates::render_index_html(contacts, crate::config::identity_sync_configured(), None)
                .map(warp::reply::html)
                .map_err(|_| warp::reject::not_found())
        })
}

fn new_contact_page()
-> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path("contacts")
        .and(warp::path("new"))
        .and(warp::path::end())
        .and(warp::get())
        .and_then(|| async move {
            templates::render_form_html(None, None)
                .map(warp::reply::html)
                .map_err(|_| warp::reject::not_found())
        })
}

fn create_contact_form(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path("contacts")
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::form())
        .and(store)
        .and_then(|form: ContactForm, store: SharedStore| async move {
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
            Ok::<_, Rejection>(response)
        })
}

fn edit_contact_page(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path!("contacts" / String / "edit")
        .and(warp::get())
        .and(store)
        .and_then(|id: String, store: SharedStore| async move {
            let Some(contact) = store
                .get(&id)
                .await
                .map_err(|_| warp::reject::not_found())?
            else {
                return Err(warp::reject::not_found());
            };
            if contact.source != ContactSource::External {
                return Err(warp::reject::not_found());
            }
            templates::render_form_html(Some(contact), None)
                .map(warp::reply::html)
                .map_err(|_| warp::reject::not_found())
        })
}

fn update_contact_form(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path!("contacts" / String / "edit")
        .and(warp::post())
        .and(warp::body::form())
        .and(store)
        .and_then(
            |id: String, form: ContactForm, store: SharedStore| async move {
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
        .and(store)
        .and_then(|id: String, store: SharedStore| async move {
            match store.delete_external(&id).await {
                Ok(()) => Ok(warp::redirect::redirect(warp::http::Uri::from_static("/"))),
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
        .and(store)
        .and_then(|store: SharedStore| async move {
            let sync_result = identity::fetch_identity_contacts().await;
            let message = match sync_result {
                Ok(identity_contacts) => match store.merge_identity(identity_contacts).await {
                    Ok(count) => Some(format!("Synced {count} identity contact(s).")),
                    Err(e) => Some(format!("Sync failed: {e}")),
                },
                Err(e) => Some(format!("Sync failed: {e}")),
            };
            let contacts = store.list().await.map_err(|_| warp::reject::not_found())?;
            templates::render_index_html(
                contacts,
                crate::config::identity_sync_configured(),
                message,
            )
            .map(warp::reply::html)
            .map_err(|_| warp::reject::not_found())
        })
}
