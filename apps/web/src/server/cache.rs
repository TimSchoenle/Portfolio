//! `Cache-Control` TTLs for served assets.
//!
//! Content-hashed JS/WASM/CSS can be cached forever; the SSR HTML must always be
//! revalidated to pick up a fresh deploy. Web fonts get a moderate TTL, and the
//! resume PDFs and generated metadata (robots.txt, sitemap.xml, web manifest) a
//! short one.
//!
//! Implemented as a response middleware that only sets `Cache-Control` when a
//! handler has not already done so, so the API's own headers win.
//!
//! NOTE: `is_content_hashed` recognises the `dx`/`manganis` hash format, in which
//! the asset filename stem ends in a `dxh`-prefixed 16-hex segment (e.g.
//! `tailwind-dxhd165a451a45ed030.css`). The wasm bundle (`/wasm/web.js`,
//! `/wasm/web_bg.wasm`) is not filename-hashed and so revalidates.

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
/// HTML / SSR fallback: cacheable but always revalidated against the origin so a
/// new deploy is served the moment it lands.
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
    // The SSR HTML, the fallback, and anything else: revalidate.
    REVALIDATE
}

/// Whether the path's filename carries a `dx`/manganis content hash: a
/// `dxh`-prefixed 16-hex segment at the end of the stem, e.g.
/// `tailwind-dxhd165a451a45ed030.css`.
fn is_content_hashed(path: &str) -> bool {
    let file = path.rsplit('/').next().unwrap_or(path);
    let stem = file.split('.').next().unwrap_or(file);
    let last = stem.rsplit('-').next().unwrap_or(stem);
    // manganis (dx) prefixes the 16-hex asset hash with `dxh`.
    let Some(hash) = last.strip_prefix("dxh") else {
        return false;
    };
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
    fn hashed_assets_are_immutable_for_a_year() {
        assert_eq!(
            cache_control_for("/assets/tailwind-dxhd165a451a45ed030.css"),
            IMMUTABLE_ONE_YEAR
        );
        assert_eq!(
            cache_control_for("/assets/favicon-dxhc8f7fbe218189b6d.svg"),
            IMMUTABLE_ONE_YEAR
        );
        assert_eq!(
            cache_control_for("/assets/fonts-dxh55766d8d82c28550.css"),
            IMMUTABLE_ONE_YEAR
        );
    }

    #[test]
    fn unhashed_wasm_bundle_revalidates() {
        // dx emits the wasm bundle at stable, unhashed paths.
        assert_eq!(cache_control_for("/wasm/web.js"), REVALIDATE);
        assert_eq!(cache_control_for("/wasm/web_bg.wasm"), REVALIDATE);
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
        assert!(!is_content_hashed(
            "/assets/tailwind-dxhzzzzzzzzzzzzzzzz.css"
        ));
        // A bare 16-hex stem without the `dxh` prefix is Trunk-era, not dx.
        assert!(!is_content_hashed("/frontend-87d64e6150ebfbc8.js"));
    }
}
