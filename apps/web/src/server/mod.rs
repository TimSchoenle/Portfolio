//! Native SSR server.
//!
//! Extends the Dioxus fullstack router (`dioxus::server::router`, which serves the hydrating SSR
//! page, static assets and any `#[server]` endpoints) with the public JSON API, the SEO
//! documents, and the security-header, compression and cache-control layers.
//!
//! - `serve` starts the process: configuration, the legal catalog, telemetry, then the serve loop
//!   with its graceful shutdown;
//! - `config` loads and validates the configuration, and refuses to start without a usable one;
//! - `router` assembles the application router and its header, compression and cache layers;
//! - `page` finishes every document: language, cookie, policy, and the memo that answers a
//!   repeat request without rendering;
//! - `isr` is the on-disk incremental render cache and the allowlist keeping it bounded;
//! - `compression`, `csp`, `cache`, `assets`, `api`, `seo`, `legal` and `telemetry` are what
//!   their names say.

mod api;
mod assets;
mod cache;
mod compression;
mod config;
mod csp;
mod isr;
pub(crate) mod legal;
mod page;
mod router;
mod seo;
mod serve;
mod telemetry;

pub use serve::serve;
