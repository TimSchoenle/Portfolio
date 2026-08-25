//! The components the pages are assembled from.
//!
//! Every one of them renders the same markup on the server and on the client, and reaches for
//! the browser only inside a `#[cfg(feature = "web")]` effect that runs after hydration. A
//! component whose server render differs from its first client render tears the hydration, so
//! the animated ones start at their finished state and step backwards once the client is live.
//!
//! Routing and the document head are not here; those are `crate::routes` and `crate::app`.

pub mod about;
pub mod canonical;
pub mod chapter_rail;
pub mod contact;
pub mod experience;
pub mod footer;
pub mod hero;
pub mod masthead;
pub mod palette;
pub mod projects;
pub mod radar;
pub mod reveal;
pub mod section_header;
pub mod skills;
