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
//! the asset filename stem ends in a `dxh`-prefixed hex segment (e.g.
//! `tailwind-dxh31cffa1cc71c7bb.css`). `dx bundle` puts the whole client bundle
//! under those names — the loader *and* the wasm binary — so everything in
//! `public/assets/` is immutable. Nothing is served from `/wasm/`; that path
//! exists only inside the build tree, before bundling renames and hashes it.

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
        // The generated Open Graph card, served unhashed from a fixed path
        // because the meta tag naming it has to stay stable.
        || path.ends_with(".png")
        || path.ends_with(".css")
    {
        return ONE_HOUR;
    }
    // The SSR HTML, the fallback, and anything else: revalidate.
    REVALIDATE
}

/// Whether the path's filename carries a `dx`/manganis content hash: a
/// `dxh`-prefixed hex segment at the end of the stem, e.g.
/// `tailwind-dxh31cffa1cc71c7bb.css`.
///
/// The digit count is **not** fixed. manganis renders the 64-bit asset hash as
/// plain hex, so a hash with leading zero nibbles is written short — one real
/// `dx bundle` of this crate produced `web-dxh9c6f94a783aca5d3.js` (16 digits)
/// and `web_bg-dxhfc275e0429871eb.wasm` (15) side by side. Requiring exactly 16
/// therefore failed to recognise roughly one asset in sixteen as hashed, and
/// those fell through to `no-cache`: the wasm binary and the stylesheet, the two
/// largest and most cacheable files on the page, were revalidated on every visit.
fn is_content_hashed(path: &str) -> bool {
    let file = path.rsplit('/').next().unwrap_or(path);
    let stem = file.split('.').next().unwrap_or(file);
    let last = stem.rsplit('-').next().unwrap_or(stem);
    let Some(hash) = last.strip_prefix("dxh") else {
        return false;
    };
    // A 64-bit value is at most 16 hex digits; the upper bound is what keeps an
    // arbitrarily long `-dxh…` segment from passing as a content hash.
    !hash.is_empty() && hash.len() <= 16 && hash.bytes().all(|b| b.is_ascii_hexdigit())
}

/// Response middleware that sets a `Cache-Control` TTL for the request path
/// unless the handler already provided one.
pub async fn set_cache_control(request: Request, next: Next) -> Response {
    // Classified before the request is consumed. `cache_control_for` answers with
    // a `&'static str`, so carrying its verdict across the await costs nothing —
    // whereas carrying the path to classify it afterwards meant copying the path
    // onto the heap for every request the server answers, asset requests
    // included.
    let value = cache_control_for(request.uri().path());
    let mut response = next.run(request).await;

    if !response.headers().contains_key(header::CACHE_CONTROL) {
        response
            .headers_mut()
            .insert(header::CACHE_CONTROL, HeaderValue::from_static(value));
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The exact file names one real `dx bundle --release` of this crate emitted
    /// into `public/`. Two of the four carry a 15-digit hash and two a 16-digit
    /// one, which is the whole reason this is asserted against observed output
    /// rather than against a hand-written example.
    const BUNDLED_ASSETS: [&str; 4] = [
        "/assets/web-dxh9c6f94a783aca5d3.js",
        "/assets/web_bg-dxhfc275e0429871eb.wasm",
        "/assets/tailwind-dxh31cffa1cc71c7bb.css",
        "/assets/fonts-dxh492b30d149ad9286.css",
    ];

    #[test]
    fn hashed_assets_are_immutable_for_a_year() {
        for asset in BUNDLED_ASSETS {
            assert_eq!(
                cache_control_for(asset),
                IMMUTABLE_ONE_YEAR,
                "{asset} is content-hashed and must never be revalidated"
            );
        }
        assert_eq!(
            cache_control_for("/assets/favicon-dxhc8f7fbe218189b6d.svg"),
            IMMUTABLE_ONE_YEAR
        );
    }

    /// The client bundle is the largest thing the page loads, and its hash is the
    /// one most likely to come out short — `{:x}` drops leading zero nibbles. A
    /// length check that missed that served it `no-cache` on every visit.
    #[test]
    fn a_short_hash_is_still_a_hash() {
        assert!(is_content_hashed("/assets/web_bg-dxhfc275e0429871eb.wasm"));
        assert!(is_content_hashed("/assets/tailwind-dxh31cffa1cc71c7bb.css"));
        // Down to a single digit, which is what a hash of 0x…0f would render as.
        assert!(is_content_hashed("/assets/thing-dxhf.css"));
        // But not beyond what 64 bits can express, and not an empty segment.
        assert!(!is_content_hashed("/assets/thing-dxh0123456789abcdef0.css"));
        assert!(!is_content_hashed("/assets/thing-dxh.css"));
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
