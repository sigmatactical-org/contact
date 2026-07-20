//! [`ContactUsTemplate`].

use askama::Template;
use sigma_theme::nav::SiteHeader;

#[derive(Template)]
#[template(path = "contact_us.html")]
pub(crate) struct ContactUsTemplate {
    pub(crate) site_header: SiteHeader,
    pub(crate) site_nav: String,
    pub(crate) return_url: String,
    pub(crate) display_name: String,
    pub(crate) email: String,
    pub(crate) phone: String,
    pub(crate) message: String,
    pub(crate) error: Option<String>,
    pub(crate) identity_base_url: String,
    pub(crate) copyright_years: String,
    pub(crate) human_check_enabled: bool,
    pub(crate) human_check_challenge_url: String,
}
