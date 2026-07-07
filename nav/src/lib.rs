//! Reusable Sigma "Contact us" navbar button linking to the contact service.
//! Shared across Sigma web services so the contact affordance looks identical
//! everywhere.

use askama::Template;

#[derive(Template)]
#[template(path = "contact_nav.html")]
struct ContactNavTemplate<'a> {
    contact_us_url: &'a str,
}

/// Render the Contact us navbar button linking to `contact_us_url`.
///
/// # Errors
///
/// Returns [`askama::Error`] when template rendering fails.
pub fn render_contact_nav(contact_us_url: &str) -> Result<String, askama::Error> {
    ContactNavTemplate { contact_us_url }.render()
}

#[cfg(test)]
mod tests {
    use super::*;

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
