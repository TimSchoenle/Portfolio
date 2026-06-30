//! `Cache-Control` TTLs for the served assets.
//!
//! Trunk content-hashes the emitted JS/WASM (and the favicon), so those URLs
//! can be cached forever; `index.html` must always be revalidated to pick up a
//! fresh deploy. Web fonts get a moderate TTL, and the resume PDFs and
//! generated metadata (robots.txt, sitemap.xml, web manifest) a short one.
//!
//! Implemented as a response middleware that only sets `Cache-Control` when a
//! handler has not already done so, so the API's own headers win.

use axum::{
    extract::Request,
    http::header::{self, HeaderValue},
    middleware::Next,
    response::Response,
};

/// Immutable, content-hashed assets: cache for a year and never revalidate.
const IMMUTABLE_ONE_YEAR: &str = "public, max-age=31536000, immutable";
/// Stable-but-unhashed media (web fonts): thirty days.
const THIRTY_DAYS: &str = "public, max-age=2592000";
/// Generated metadata and the resume PDFs: one hour.
const ONE_HOUR: &str = "public, max-age=3600";
/// HTML / SPA fallback: cacheable but always revalidated against the origin so
/// a new deploy is served the moment it lands.
const REVALIDATE: &str = "no-cache";

/// Picks the `Cache-Control` value for a static-asset request path.
fn cache_control_for(path: &str) -> &'static str {
    if is_content_hashed(path) {
        return IMMUTABLE_ONE_YEAR;
    }
    if path.ends_with(".woff2") {
        return THIRTY_DAYS;
    }
    if path.starts_with("/resume/")
        || path.ends_with(".pdf")
        || path.ends_with(".webmanifest")
        || path.ends_with("robots.txt")
        || path.ends_with("sitemap.xml")
        || path.ends_with(".svg")
        || path.ends_with(".css")
    {
        return ONE_HOUR;
    }
    // index.html, the SPA fallback, and anything else: revalidate.
    REVALIDATE
}

/// Whether the path's filename carries a Trunk content hash: a 16-hex segment
/// in the stem, e.g. `frontend-87d64e6150ebfbc8.js`.
fn is_content_hashed(path: &str) -> bool {
    let file = path.rsplit('/').next().unwrap_or(path);
    let stem = file.split('.').next().unwrap_or(file);
    if !stem.contains('-') {
        return false;
    }
    let last = stem.rsplit('-').next().unwrap_or(stem);
    // wasm-bindgen appends `_bg` to the hash in the WASM filename.
    let hash = last.strip_suffix("_bg").unwrap_or(last);
    hash.len() == 16 && hash.bytes().all(|b| b.is_ascii_hexdigit())
}

/// Response middleware that sets a `Cache-Control` TTL for the request path
/// unless the handler already provided one.
pub async fn set_cache_control(request: Request, next: Next) -> Response {
    let path = request.uri().path().to_owned();
    let mut response = next.run(request).await;

    if !response.headers().contains_key(header::CACHE_CONTROL) {
        let value = cache_control_for(&path);
        response
            .headers_mut()
            .insert(header::CACHE_CONTROL, HeaderValue::from_static(value));
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashed_js_and_wasm_are_immutable_for_a_year() {
        assert_eq!(
            cache_control_for("/frontend-87d64e6150ebfbc8.js"),
            IMMUTABLE_ONE_YEAR
        );
        assert_eq!(
            cache_control_for("/frontend-87d64e6150ebfbc8_bg.wasm"),
            IMMUTABLE_ONE_YEAR
        );
        assert_eq!(
            cache_control_for("/favicon-4203b67ddd500b42.svg"),
            IMMUTABLE_ONE_YEAR
        );
    }

    #[test]
    fn fonts_get_a_moderate_ttl() {
        assert_eq!(
            cache_control_for("/fonts/jetbrains-mono-latin-400-normal.woff2"),
            THIRTY_DAYS
        );
    }

    #[test]
    fn generated_metadata_and_resumes_get_a_short_ttl() {
        assert_eq!(cache_control_for("/resume/cv.pdf"), ONE_HOUR);
        assert_eq!(cache_control_for("/site.webmanifest"), ONE_HOUR);
        assert_eq!(cache_control_for("/robots.txt"), ONE_HOUR);
        assert_eq!(cache_control_for("/sitemap.xml"), ONE_HOUR);
        assert_eq!(cache_control_for("/favicon.svg"), ONE_HOUR);
    }

    #[test]
    fn html_and_unknown_paths_revalidate() {
        assert_eq!(cache_control_for("/"), REVALIDATE);
        assert_eq!(cache_control_for("/index.html"), REVALIDATE);
        assert_eq!(cache_control_for("/projects"), REVALIDATE);
    }

    #[test]
    fn unhashed_names_are_not_treated_as_immutable() {
        assert!(!is_content_hashed("/favicon.svg"));
        assert!(!is_content_hashed(
            "/fonts/jetbrains-mono-latin-400-normal.woff2"
        ));
        // 16 chars but not all hex.
        assert!(!is_content_hashed("/frontend-zzzzzzzzzzzzzzzz.js"));
    }
}
