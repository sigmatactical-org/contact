//! [`ContactNavTemplate`].

use askama::Template;

#[derive(Template)]
#[template(path = "contact_nav.html")]
pub(crate) struct ContactNavTemplate<'a> {
    pub(crate) contact_us_url: &'a str,
}
