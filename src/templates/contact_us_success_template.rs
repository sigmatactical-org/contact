//! [`ContactUsSuccessTemplate`].

#[allow(unused_imports)]
use super::*;
use askama::Template;
use sigma_theme::nav::SiteHeader;

#[derive(Template)]
#[template(path = "contact_us_success.html")]
pub(crate) struct ContactUsSuccessTemplate {
    pub(crate) site_header: SiteHeader,
    pub(crate) site_nav: String,
    pub(crate) return_url: String,
    pub(crate) copyright_years: String,
}
