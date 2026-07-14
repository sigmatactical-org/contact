//! Reusable Sigma "Contact us" navbar button linking to the contact service.
//! Shared across Sigma web services so the contact affordance looks identical
//! everywhere.

#![forbid(unsafe_code)]

use askama::Template;

#[derive(Template)]
#[template(path = "contact_nav.html")]
struct ContactNavTemplate<'a> {
    contact_us_url: &'a str,
}

/// Build the contact form URL that returns the user to `return_path` on the
/// calling app.
///
/// - `contact_base`: public base URL of the contact service.
/// - `app_base`: public base URL of the calling app (used to build `return_url`).
/// - `return_path`: path on the calling app to return to after contact.
#[must_use]
pub fn contact_us_url(contact_base: &str, app_base: &str, return_path: &str) -> String {
    let app_uri = join_url(app_base, return_path);
    let contact_root = contact_base.trim_end_matches('/');
    format!(
        "{contact_root}/contact?return_url={}",
        percent_encode(&app_uri)
    )
}

/// Render the Contact us navbar button linking to `contact_us_url`.
///
/// # Errors
///
/// Returns [`askama::Error`] when template rendering fails.
pub fn render_contact_nav(contact_us_url: &str) -> Result<String, askama::Error> {
    ContactNavTemplate { contact_us_url }.render()
}

fn join_url(base: &str, path: &str) -> String {
    let base = base.trim_end_matches('/');
    if path == "/" || path.is_empty() {
        return format!("{base}/");
    }
    let path = path.trim_start_matches('/');
    format!("{base}/{path}")
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
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_contact_url_with_return_path() {
        let url = contact_us_url(
            "http://contact.example/",
            "http://store.example/",
            "/products/SIGMA-RACER",
        );
        assert!(url.contains("/contact?return_url="));
        assert!(url.contains("return_url=http%3A%2F%2Fstore.example%2Fproducts%2FSIGMA-RACER"));
    }

    #[test]
    fn root_return_path_keeps_trailing_slash() {
        let url = contact_us_url("http://contact.example", "http://store.example", "/");
        assert!(url.contains("return_url=http%3A%2F%2Fstore.example%2F"));
    }

    #[test]
    fn renders_contact_link() {
        let html = render_contact_nav(
            "http://contact.example/contact?return_url=http%3A%2F%2Fstore.example%2F",
        )
        .expect("render");
        assert!(html.contains(
            "href=\"http://contact.example/contact?return_url=http%3A%2F%2Fstore.example%2F\""
        ));
        assert!(html.contains("aria-label=\"Contact us\""));
        assert!(html.contains(">Contact us</a>"));
    }
}
