use askama::Template;

use crate::model::Contact;
use sigma_theme::copyright_years;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    identity_contacts: Vec<Contact>,
    external_contacts: Vec<Contact>,
    identity_sync_configured: bool,
    message: Option<String>,
    copyright_years: String,
}

#[derive(Template)]
#[template(path = "form.html")]
struct FormTemplate {
    contact: Option<Contact>,
    display_name: String,
    email: String,
    phone: String,
    notes: String,
    error: Option<String>,
    copyright_years: String,
}

fn partition_contacts(contacts: Vec<Contact>) -> (Vec<Contact>, Vec<Contact>) {
    let mut identity_contacts = Vec::new();
    let mut external_contacts = Vec::new();
    for contact in contacts {
        if contact.source == crate::model::ContactSource::Identity {
            identity_contacts.push(contact);
        } else {
            external_contacts.push(contact);
        }
    }
    (identity_contacts, external_contacts)
}

/// # Errors
///
/// Returns [`askama::Error`] when template rendering fails.
pub fn render_index_html(
    contacts: Vec<Contact>,
    identity_sync_configured: bool,
    message: Option<String>,
) -> Result<String, askama::Error> {
    let (identity_contacts, external_contacts) = partition_contacts(contacts);
    IndexTemplate {
        identity_contacts,
        external_contacts,
        identity_sync_configured,
        message,
        copyright_years: copyright_years(),
    }
    .render()
}

/// # Errors
///
/// Returns [`askama::Error`] when template rendering fails.
pub fn render_form_html(
    contacts: Vec<Contact>,
    contact: Option<Contact>,
    error: Option<String>,
    _identity_sync_configured: bool,
) -> Result<String, askama::Error> {
    let _ = contacts;
    let (display_name, email, phone, notes) = match contact.as_ref() {
        Some(c) => (
            c.display_name.clone(),
            c.email.clone().unwrap_or_default(),
            c.phone.clone().unwrap_or_default(),
            c.notes.clone().unwrap_or_default(),
        ),
        None => (String::new(), String::new(), String::new(), String::new()),
    };
    FormTemplate {
        contact,
        display_name,
        email,
        phone,
        notes,
        error,
        copyright_years: copyright_years(),
    }
    .render()
}
