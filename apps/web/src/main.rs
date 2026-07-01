//! Dioxus fullstack entrypoint.
//!
//! The same [`app::App`] component tree is compiled twice: once to wasm (the
//! `web` feature) where it hydrates the server-rendered HTML, and once to a
//! native binary (the `server` feature) that runs the Axum SSR server. With
//! neither feature (a bare `cargo check`), `main` is empty and only the
//! renderer-agnostic component code is type-checked.

mod app;

fn main() {
    #[cfg(feature = "server")]
    backend::serve();

    #[cfg(all(feature = "web", not(feature = "server")))]
    dioxus::launch(app::App);
}

/// Native SSR server. Kept in a `cfg`-gated module so its Axum/`dioxus::server`
/// dependencies never enter the wasm client build.
#[cfg(feature = "server")]
mod backend {
    pub fn serve() {
        // Binds to `IP`/`PORT` (default 127.0.0.1:8080). Custom routes,
        // security headers and the cache-control layer are added in Phase 2.
        dioxus::serve(|| async move { Ok(dioxus::server::router(crate::app::App)) });
    }
}
