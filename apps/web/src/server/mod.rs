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
//! - `isr` is the on-disk incremental render cache and the allowlist keeping it bounded;
//! - `assets` embeds the files `portfolio_server::assets` serves;
//! - `compression`, `seo`, `legal` and `telemetry` are what their names say.
//!
//! The API, the asset routes, the cache and security policies and the middleware that finishes
//! every document live in `portfolio_server`, which needs no Dioxus.

mod assets;
mod compression;
mod config;
mod isr;
pub(crate) mod legal;
mod router;
mod seo;
mod serve;
mod telemetry;

pub use serve::serve;
