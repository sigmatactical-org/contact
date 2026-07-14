//! [`IndexTemplate`].

#[allow(unused_imports)]
use super::*;
use crate::model::Contact;
use askama::Template;
use sigma_theme::nav::SiteHeader;

#[derive(Template)]
#[template(path = "index.html")]
pub(crate) struct IndexTemplate {
    pub(crate) site_header: SiteHeader,
    pub(crate) site_nav: String,
    pub(crate) identity_contacts: Vec<Contact>,
    pub(crate) external_contacts: Vec<Contact>,
    pub(crate) identity_sync_configured: bool,
    pub(crate) message: Option<String>,
    pub(crate) copyright_years: String,
}
