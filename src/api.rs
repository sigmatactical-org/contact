use std::convert::Infallible;

use warp::http::StatusCode;
use warp::reply::Response;
use warp::{Filter, Rejection, Reply};

use crate::SharedStore;
use crate::identity;
use crate::model::{CreateContact, UpdateContact};
use crate::store::StoreError;

#[derive(serde::Serialize)]
struct SyncResponse {
    synced: usize,
}

#[derive(serde::Serialize)]
struct ErrorBody {
    error: String,
}

fn json_error(status: StatusCode, message: impl Into<String>) -> Response {
    warp::reply::with_status(
        warp::reply::json(&ErrorBody {
            error: message.into(),
        }),
        status,
    )
    .into_response()
}

fn store_error_status(err: &StoreError) -> StatusCode {
    match err {
        StoreError::NotFound => StatusCode::NOT_FOUND,
        StoreError::IdentityReadOnly => StatusCode::FORBIDDEN,
        StoreError::InvalidInput(_) => StatusCode::BAD_REQUEST,
        StoreError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub fn routes(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    list_contacts(store.clone())
        .or(get_contact(store.clone()))
        .or(create_contact(store.clone()))
        .or(update_contact(store.clone()))
        .or(delete_contact(store.clone()))
        .or(sync_contacts(store))
}

fn list_contacts(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path("contacts")
        .and(warp::path::end())
        .and(warp::get())
        .and(store)
        .and_then(|store: SharedStore| async move {
            let store = store.lock().await;
            let contacts = store.list().await.map_err(|_| warp::reject::not_found())?;
            Ok::<_, Rejection>(warp::reply::json(&contacts))
        })
}

fn get_contact(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path!("contacts" / String)
        .and(warp::path::end())
        .and(warp::get())
        .and(store)
        .and_then(|id: String, store: SharedStore| async move {
            let store = store.lock().await;
            match store
                .get(&id)
                .await
                .map_err(|_| warp::reject::not_found())?
            {
                Some(contact) => Ok(warp::reply::json(&contact)),
                None => Err(warp::reject::not_found()),
            }
        })
}

fn create_contact(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path("contacts")
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .and(store)
        .and_then(|input: CreateContact, store: SharedStore| async move {
            let mut store = store.lock().await;
            let response = match store.create_external(input).await {
                Ok(contact) => {
                    warp::reply::with_status(warp::reply::json(&contact), StatusCode::CREATED)
                        .into_response()
                }
                Err(e) => json_error(store_error_status(&e), e.to_string()),
            };
            Ok::<_, Rejection>(response)
        })
}

fn update_contact(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path!("contacts" / String)
        .and(warp::path::end())
        .and(warp::put())
        .and(warp::body::json())
        .and(store)
        .and_then(
            |id: String, input: UpdateContact, store: SharedStore| async move {
                let mut store = store.lock().await;
                let response = match store.update_external(&id, input).await {
                    Ok(contact) => warp::reply::json(&contact).into_response(),
                    Err(StoreError::NotFound) => return Err(warp::reject::not_found()),
                    Err(e) => json_error(store_error_status(&e), e.to_string()),
                };
                Ok(response)
            },
        )
}

fn delete_contact(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path!("contacts" / String)
        .and(warp::path::end())
        .and(warp::delete())
        .and(store)
        .and_then(|id: String, store: SharedStore| async move {
            let mut store = store.lock().await;
            let response = match store.delete_external(&id).await {
                Ok(()) => {
                    warp::reply::with_status(warp::reply(), StatusCode::NO_CONTENT).into_response()
                }
                Err(StoreError::NotFound) => return Err(warp::reject::not_found()),
                Err(e) => json_error(store_error_status(&e), e.to_string()),
            };
            Ok(response)
        })
}

fn sync_contacts(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path!("contacts" / "sync")
        .and(warp::path::end())
        .and(warp::post())
        .and(store)
        .and_then(|store: SharedStore| async move {
            match identity::fetch_identity_contacts().await {
                Ok(identity_contacts) => {
                    let mut store = store.lock().await;
                    let response = match store.merge_identity(identity_contacts).await {
                        Ok(synced) => warp::reply::json(&SyncResponse { synced }).into_response(),
                        Err(e) => json_error(store_error_status(&e), e.to_string()),
                    };
                    Ok::<_, Rejection>(response)
                }
                Err(e) => Ok(json_error(StatusCode::BAD_GATEWAY, e.to_string())),
            }
        })
}
