mod contact_us_success_template;
mod contact_us_template;
mod form_template;
mod index_template;
pub(crate) use contact_us_success_template::ContactUsSuccessTemplate;
pub(crate) use contact_us_template::ContactUsTemplate;
pub(crate) use form_template::FormTemplate;
pub(crate) use index_template::IndexTemplate;

use askama::Template;

use crate::model::{Contact, ContactInquiryForm};
use sigma_theme::copyright_years;
use sigma_theme::nav::{Breadcrumb, SiteHeader};
use sigma_theme::site_nav::{AppSiteNav, render_app_site_nav};

fn page_header(brand: &str) -> SiteHeader {
    SiteHeader::new(brand)
}

fn site_nav(return_path: &str, show_contact_us: bool) -> Result<String, askama::Error> {
    render_app_site_nav(&AppSiteNav {
        identity_base: &crate::config::identity_public_base_url(),
        app_base: &crate::config::public_base_url(),
        contact_base: &crate::config::public_base_url(),
        cart_url: &crate::config::cart_public_base_url(),
        cart_count: 0,
        return_path,
        show_cart: true,
        show_contact_us,
        leading_html: "",
    })
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
pub fn render_contact_us_html(
    return_url: &str,
    form: Option<ContactInquiryForm>,
    error: Option<String>,
    human_check: &sigma_human_check::HumanCheck,
) -> Result<String, askama::Error> {
    let (display_name, email, phone, message) = match form {
        Some(form) => (form.display_name, form.email, form.phone, form.message),
        None => (String::new(), String::new(), String::new(), String::new()),
    };
    ContactUsTemplate {
        site_header: page_header("Sigma Contact")
            .with_breadcrumb(Breadcrumb::current("Contact us")),
        site_nav: site_nav("/contact", false)?,
        return_url: return_url.to_string(),
        display_name,
        email,
        phone,
        message,
        error,
        identity_base_url: crate::config::identity_public_base_url(),
        copyright_years: copyright_years(),
        human_check_enabled: human_check.is_enabled(),
        human_check_challenge_url: "/human-check/challenge".to_string(),
    }
    .render()
}

/// # Errors
///
/// Returns [`askama::Error`] when template rendering fails.
pub fn render_contact_us_success_html(return_url: &str) -> Result<String, askama::Error> {
    ContactUsSuccessTemplate {
        site_header: page_header("Sigma Contact")
            .with_breadcrumb(Breadcrumb::current("Message sent")),
        site_nav: site_nav("/contact/success", false)?,
        return_url: return_url.to_string(),
        copyright_years: copyright_years(),
    }
    .render()
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
        site_header: page_header("Sigma Contact"),
        site_nav: site_nav("/", false)?,
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
    let return_path = contact
        .as_ref()
        .map(|entry| format!("/contacts/{}/edit", entry.id))
        .unwrap_or_else(|| "/contacts/new".to_string());
    FormTemplate {
        site_header: page_header("Sigma Contact")
            .with_breadcrumb(Breadcrumb::link("/", "Contacts"))
            .with_breadcrumb(Breadcrumb::current(if contact.is_some() {
                "Edit contact"
            } else {
                "New contact"
            })),
        site_nav: site_nav(&return_path, false)?,
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
