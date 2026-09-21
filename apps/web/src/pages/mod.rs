//! Routed page components. Their names match the `Route` variants.

mod home;
mod legal;
mod licenses;
mod not_found;

pub use home::Home;
pub use legal::LegalDocument;
pub use licenses::Licenses;
pub use not_found::NotFound;
