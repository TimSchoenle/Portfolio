//! Stable-path static assets embedded in the server binary.
//!
//! The favicon and the vendored woff2 fonts are referenced by absolute URLs
//! (`/favicon.svg`, `/fonts/…`) from `fonts.css`, the masthead logo and the web
//! manifest, so they must live at fixed paths rather than the content-hashed
//! paths `dx`/manganis produces. Embedding them keeps the runtime a single
//! self-contained binary (no asset directory to ship alongside it). Their
//! `Cache-Control` is filled in by the cache middleware (see `cache.rs`).

use axum::{
    Router,
    extract::Path,
    http::{StatusCode, header},
    response::IntoResponse,
    routing::get,
};

const FAVICON: &[u8] = include_bytes!("../../assets/favicon.svg");

/// (URL filename, bytes) for every vendored font. Kept in sync with `fonts.css`
/// and `scripts/vendor-fonts.mjs`.
const FONTS: &[(&str, &[u8])] = &[
    (
        "space-grotesk-latin-400-normal.woff2",
        include_bytes!("../../assets/fonts/space-grotesk-latin-400-normal.woff2"),
    ),
    (
        "space-grotesk-latin-500-normal.woff2",
        include_bytes!("../../assets/fonts/space-grotesk-latin-500-normal.woff2"),
    ),
    (
        "space-grotesk-latin-600-normal.woff2",
        include_bytes!("../../assets/fonts/space-grotesk-latin-600-normal.woff2"),
    ),
    (
        "space-grotesk-latin-700-normal.woff2",
        include_bytes!("../../assets/fonts/space-grotesk-latin-700-normal.woff2"),
    ),
    (
        "jetbrains-mono-latin-400-normal.woff2",
        include_bytes!("../../assets/fonts/jetbrains-mono-latin-400-normal.woff2"),
    ),
    (
        "jetbrains-mono-latin-500-normal.woff2",
        include_bytes!("../../assets/fonts/jetbrains-mono-latin-500-normal.woff2"),
    ),
    (
        "jetbrains-mono-latin-700-normal.woff2",
        include_bytes!("../../assets/fonts/jetbrains-mono-latin-700-normal.woff2"),
    ),
];

/// Routes for the fixed-path embedded assets.
pub fn router() -> Router {
    Router::new()
        .route("/favicon.svg", get(favicon))
        .route("/fonts/{file}", get(font))
}

async fn favicon() -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "image/svg+xml")], FAVICON)
}

async fn font(Path(file): Path<String>) -> impl IntoResponse {
    match FONTS.iter().find(|(name, _)| *name == file) {
        Some((_, bytes)) => ([(header::CONTENT_TYPE, "font/woff2")], *bytes).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
