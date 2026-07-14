use std::convert::Infallible;

use warp::http::StatusCode;
use warp::{Filter, Rejection, Reply};

use crate::SharedStore;
use crate::allowlist::UriAllowlist;
use crate::human_check;
use crate::model::{ContactInquiryForm, CreateContact};
use crate::session_status;
use crate::store::StoreError;
use crate::templates;

static RETURN_URL_ALLOWLIST: std::sync::OnceLock<UriAllowlist> = std::sync::OnceLock::new();

fn return_url_allowlist() -> &'static UriAllowlist {
    RETURN_URL_ALLOWLIST.get_or_init(|| UriAllowlist::new(crate::config::return_uris()))
}

/// Build this module's routes.
pub fn routes(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
    human_check: sigma_human_check::HumanCheck,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    contact_form(store.clone(), human_check.clone()).or(contact_success())
}

fn contact_form(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
    human_check: sigma_human_check::HumanCheck,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    let path = warp::path("contact").and(warp::path::end());

    let get_form = path
        .and(warp::get())
        .and(warp::query::<ContactQuery>())
        .and(human_check::with_check(human_check.clone()))
        .and_then(
            |query: ContactQuery, human_check: sigma_human_check::HumanCheck| async move {
                if !return_url_is_allowed(&query.return_url) {
                    return Ok(warp::reply::with_status(
                        "Invalid return_url",
                        StatusCode::BAD_REQUEST,
                    )
                    .into_response());
                }
                templates::render_contact_us_html(&query.return_url, None, None, &human_check)
                    .map(|html| warp::reply::html(html).into_response())
                    .map_err(|_| warp::reject::not_found())
            },
        );

    let post_form = path
        .and(warp::post())
        .and(warp::body::form())
        .and(warp::header::optional::<String>("cookie"))
        .and(store)
        .and(human_check::with_check(human_check))
        .and_then(
            |form: ContactInquiryForm,
             cookie: Option<String>,
             store: SharedStore,
             human_check: sigma_human_check::HumanCheck| async move {
                let mut form = form;
                if let Some(status) = session_status::fetch_identity_status(cookie.as_deref()).await
                    && status.authenticated
                {
                    if let Some(display_name) = status.username {
                        form.display_name = display_name;
                    }
                    if let Some(email) = status.email {
                        form.email = email;
                    }
                }

                if !return_url_is_allowed(&form.return_url) {
                    return Ok(warp::reply::with_status(
                        "Invalid return_url",
                        StatusCode::BAD_REQUEST,
                    )
                    .into_response());
                }
                if let Err(message) = form.validate() {
                    let return_url = form.return_url.clone();
                    return templates::render_contact_us_html(
                        &return_url,
                        Some(form),
                        Some(message),
                        &human_check,
                    )
                    .map(|html| {
                        warp::reply::with_status(warp::reply::html(html), StatusCode::BAD_REQUEST)
                            .into_response()
                    })
                    .map_err(|_| warp::reject::not_found());
                }

                if let Err(err) = human_check::verify_field(&human_check, &form.altcha) {
                    let return_url = form.return_url.clone();
                    return templates::render_contact_us_html(
                        &return_url,
                        Some(form),
                        Some(human_check::rejection_message(&err)),
                        &human_check,
                    )
                    .map(|html| {
                        warp::reply::with_status(warp::reply::html(html), StatusCode::BAD_REQUEST)
                            .into_response()
                    })
                    .map_err(|_| warp::reject::not_found());
                }

                let notes = format!("Website inquiry:\n\n{}", form.message.trim());
                let input = CreateContact {
                    display_name: form.display_name.trim().to_string(),
                    email: Some(form.email.trim().to_string()),
                    phone: empty_to_none(form.phone.clone()),
                    notes: Some(notes),
                };

                match store.create_external(input).await {
                    Ok(_) => {
                        let location: warp::http::Uri = success_location(&form.return_url)
                            .parse()
                            .expect("valid redirect location");
                        Ok(warp::redirect::found(location).into_response())
                    }
                    Err(StoreError::InvalidInput(message)) => {
                        let return_url = form.return_url.clone();
                        templates::render_contact_us_html(
                            &return_url,
                            Some(form),
                            Some(message),
                            &human_check,
                        )
                        .map(|html| {
                            warp::reply::with_status(
                                warp::reply::html(html),
                                StatusCode::BAD_REQUEST,
                            )
                            .into_response()
                        })
                        .map_err(|_| warp::reject::not_found())
                    }
                    Err(_) => Err(warp::reject::not_found()),
                }
            },
        );

    get_form.or(post_form)
}

fn contact_success()
-> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path!("contact" / "success")
        .and(warp::path::end())
        .and(warp::get())
        .and(warp::query::<ContactQuery>())
        .and_then(|query: ContactQuery| async move {
            if !query.return_url.is_empty() && !return_url_is_allowed(&query.return_url) {
                return Ok(
                    warp::reply::with_status("Invalid return_url", StatusCode::BAD_REQUEST)
                        .into_response(),
                );
            }
            templates::render_contact_us_success_html(&query.return_url)
                .map(|html| warp::reply::html(html).into_response())
                .map_err(|_| warp::reject::not_found())
        })
}

#[derive(Debug, serde::Deserialize)]
struct ContactQuery {
    #[serde(default)]
    return_url: String,
}

fn return_url_is_allowed(return_url: &str) -> bool {
    return_url.is_empty() || return_url_allowlist().is_allowed(return_url)
}

fn success_location(return_url: &str) -> String {
    if return_url.is_empty() {
        return "/contact/success".to_string();
    }
    format!("/contact/success?return_url={}", percent_encode(return_url))
}

fn percent_encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            b'/' => out.push_str("%2F"),
            b':' => out.push_str("%3A"),
            b'?' => out.push_str("%3F"),
            b'#' => out.push_str("%23"),
            b'[' => out.push_str("%5B"),
            b']' => out.push_str("%5D"),
            b'@' => out.push_str("%40"),
            b'!' => out.push_str("%21"),
            b'$' => out.push_str("%24"),
            b'&' => out.push_str("%26"),
            b'\'' => out.push_str("%27"),
            b'(' => out.push_str("%28"),
            b')' => out.push_str("%29"),
            b'*' => out.push_str("%2A"),
            b'+' => out.push_str("%2B"),
            b',' => out.push_str("%2C"),
            b';' => out.push_str("%3B"),
            b'=' => out.push_str("%3D"),
            b' ' => out.push_str("%20"),
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

fn empty_to_none(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}
