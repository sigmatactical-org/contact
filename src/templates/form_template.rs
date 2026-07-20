//! [`FormTemplate`].

use askama::Template;
use sigma_theme::nav::SiteHeader;

use crate::model::Contact;

#[derive(Template)]
#[template(path = "form.html")]
pub(crate) struct FormTemplate {
    pub(crate) site_header: SiteHeader,
    pub(crate) site_nav: String,
    pub(crate) contact: Option<Contact>,
    pub(crate) display_name: String,
    pub(crate) email: String,
    pub(crate) phone: String,
    pub(crate) notes: String,
    pub(crate) error: Option<String>,
    pub(crate) copyright_years: String,
}
