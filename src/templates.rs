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

#[derive(Template)]
#[template(path = "contact_us.html")]
struct ContactUsTemplate {
    site_header: SiteHeader,
    site_nav: String,
    return_url: String,
    display_name: String,
    email: String,
    phone: String,
    message: String,
    error: Option<String>,
    identity_base_url: String,
    copyright_years: String,
    human_check_enabled: bool,
    human_check_challenge_url: String,
}

#[derive(Template)]
#[template(path = "contact_us_success.html")]
struct ContactUsSuccessTemplate {
    site_header: SiteHeader,
    site_nav: String,
    return_url: String,
    copyright_years: String,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    site_header: SiteHeader,
    site_nav: String,
    identity_contacts: Vec<Contact>,
    external_contacts: Vec<Contact>,
    identity_sync_configured: bool,
    message: Option<String>,
    copyright_years: String,
}

#[derive(Template)]
#[template(path = "form.html")]
struct FormTemplate {
    site_header: SiteHeader,
    site_nav: String,
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
        site_nav: site_nav("/contact/success", true)?,
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
        site_nav: site_nav("/", true)?,
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
        site_nav: site_nav(&return_path, true)?,
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
