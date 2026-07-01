//! Routed page components. Their names match the `Route` variants.

mod home;
mod imprint;
mod not_found;
mod privacy;

pub use home::Home;
pub use imprint::Imprint;
pub use not_found::NotFound;
pub use privacy::Privacy;
