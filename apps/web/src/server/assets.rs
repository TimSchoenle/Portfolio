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
use portfolio_data::RESUME_FILES;

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
        .route("/resume/{file}", get(resume))
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

/// Build-generated resume PDFs, embedded at compile time by `build.rs` under
/// stable ASCII names. Empty when the resume generator has not run (dev builds),
/// in which case the route replies 404 — mirroring the empty fingerprint
/// manifest. Embedding keeps the SSR server a single self-contained binary and
/// sidesteps the non-ASCII file names (e.g. `Tim-Schönle-Lebenslauf.pdf`) that
/// Dioxus's on-disk `public/` static serving failed to resolve.
const RESUME_EN: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/resume-en.pdf"));
const RESUME_DE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/resume-de.pdf"));

/// The embedded PDF bytes for a resume language code.
fn resume_bytes(lang: &str) -> &'static [u8] {
    match lang {
        "de" => RESUME_DE,
        _ => RESUME_EN,
    }
}

/// Serves an embedded resume PDF by its exact published file name.
///
/// Accepting only the known `RESUME_FILES` names keeps the handler total and
/// rules out path traversal. A language whose PDF was not generated (empty embed)
/// yields 404.
async fn resume(Path(file): Path<String>) -> impl IntoResponse {
    let Some((lang, _)) = RESUME_FILES.iter().find(|(_, name)| *name == file) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let bytes = resume_bytes(lang);
    if bytes.is_empty() {
        return StatusCode::NOT_FOUND.into_response();
    }
    ([(header::CONTENT_TYPE, "application/pdf")], bytes).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    async fn status(path: &str) -> StatusCode {
        router()
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap()
            .status()
    }

    #[tokio::test]
    async fn favicon_is_served() {
        assert_eq!(status("/favicon.svg").await, StatusCode::OK);
    }

    #[tokio::test]
    async fn unknown_font_is_not_found() {
        assert_eq!(status("/fonts/not-a-font.woff2").await, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn resume_rejects_names_outside_the_allowlist() {
        // Only the published `RESUME_FILES` names are accepted; anything else —
        // including path-traversal attempts — is refused before any disk access.
        assert_eq!(status("/resume/anything-else.pdf").await, StatusCode::NOT_FOUND);
        assert_eq!(status("/resume/..%2f..%2fCargo.toml").await, StatusCode::NOT_FOUND);
    }
}
