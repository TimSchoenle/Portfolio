//! Stable-path static assets embedded in the server binary.
//!
//! The favicon and the vendored woff2 fonts are referenced by absolute URLs
//! (`/favicon.svg`, `/fonts/…`) from `fonts.css`, the masthead logo and the web
//! manifest, so they must live at fixed paths rather than the content-hashed
//! paths `dx`/manganis produces. Embedding them keeps the runtime a single
//! self-contained binary (no asset directory to ship alongside it). Their
//! `Cache-Control` is filled in by the cache middleware (see `cache.rs`).

use std::path::PathBuf;

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

/// Directory holding the build-generated resume PDFs, resolved next to the
/// server binary. The bundle ships `public/` as a sibling of `server`, so this
/// resolves correctly regardless of the process's working directory.
fn resume_dir() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    Some(exe.parent()?.join("public").join("resume"))
}

/// Serves a build-generated resume PDF by its exact published file name.
///
/// The resume file names are non-ASCII (e.g. `Tim-Schönle-Lebenslauf.pdf`), and
/// Dioxus's built-in `public/` static serving fails to resolve them — requests
/// fall through to the SSR 404 page even though the file is present. We serve
/// them from an explicit route instead. Accepting only the known `RESUME_FILES`
/// names keeps the handler total and rules out path traversal. Missing files
/// (dev builds where the generator has not run) yield 404, mirroring the empty
/// fingerprint manifest.
async fn resume(Path(file): Path<String>) -> impl IntoResponse {
    if !RESUME_FILES.iter().any(|(_, name)| *name == file) {
        return StatusCode::NOT_FOUND.into_response();
    }
    let Some(path) = resume_dir().map(|dir| dir.join(&file)) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    match tokio::fs::read(&path).await {
        Ok(bytes) => ([(header::CONTENT_TYPE, "application/pdf")], bytes).into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
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
