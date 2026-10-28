//! The renderer-independent half of the Portfolio web server.
//!
//! `apps/web` renders pages with Dioxus and builds the router. This crate holds what that
//! router mounts and does not need Dioxus for:
//!
//! - [`api`]: the JSON profile API and the health probes.
//! - [`assets`]: the favicon, fonts, resume PDFs and icons embedded in the binary.
//! - [`cache`]: the `Cache-Control` policy per path.
//! - [`csp`]: the `Content-Security-Policy` for documents and subresources.
//! - [`page`]: the middleware that finishes every document, and the page memo behind it.
//! - [`locale`] and [`isr_marker`]: the language negotiation and cache key [`page`] relies on.

pub mod api;
pub mod assets;
pub mod cache;
pub mod csp;
pub mod isr_marker;
pub mod locale;
pub mod page;
