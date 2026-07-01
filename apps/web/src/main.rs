//! Dioxus fullstack entrypoint.
//!
//! The same [`app::App`] component tree is compiled twice: once to wasm (the
//! `web` feature) where it hydrates the server-rendered HTML, and once to a
//! native binary (the `server` feature) that runs the Axum SSR server. With
//! neither feature (a bare `cargo check`), `main` is empty and only the
//! renderer-agnostic component code is type-checked.

mod app;
mod github;
mod i18n;
mod pages;
mod routes;
mod sections;
mod ui;
mod util;

/// Native SSR server (Axum router, public API, SEO routes, security headers).
/// `cfg`-gated so none of its dependencies enter the wasm client build.
#[cfg(feature = "server")]
mod server;

fn main() {
    #[cfg(feature = "server")]
    server::serve();

    #[cfg(all(feature = "web", not(feature = "server")))]
    dioxus::launch(app::App);
}
