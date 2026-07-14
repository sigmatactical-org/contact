mod contact;
mod contact_form;
mod contact_inquiry_form;
mod contact_source;
mod create_contact;
mod update_contact;
pub use contact::Contact;
pub use contact_form::ContactForm;
pub use contact_inquiry_form::ContactInquiryForm;
pub use contact_source::ContactSource;
pub use create_contact::CreateContact;
pub use update_contact::UpdateContact;

fn empty_to_none(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}
