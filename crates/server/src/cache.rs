//! The `Cache-Control` policy for every response the server sends.
//!
//! Content-hashed JS, WASM and CSS are cached for a year without revalidation, and the SSR HTML
//! is always revalidated so a fresh deploy is picked up. Web fonts get a moderate TTL, and the
//! resume PDFs and generated metadata (`robots.txt`, `sitemap.xml`, the web manifest) a short one.
//! An error response gets `no-store`.
//!
//! [`set_cache_control`] sets the header only when a handler or inner middleware has not, so the
//! API's own headers win, and an error document whose policy carries a nonce keeps the
//! `no-cache` that nonce requires.
//!
//! What counts as content-hashed is the `dx`/`manganis` naming: a filename stem ending in a
//! `dxh`-prefixed hex segment, as in `tailwind-dxh31cffa1cc71c7bb.css`. `dx bundle` puts the
//! whole client bundle under those names, the loader and the wasm binary alike, so everything
//! under `public/assets/` is immutable. The hex run is not fixed-width: plain hex drops leading
//! zero nibbles, and a rule that assumed sixteen digits served the two largest files on the page
//! uncached.
//!
//! Nothing is served from `/wasm/`. That path exists inside the build tree only, before
//! bundling renames and hashes it.

use axum::{
    extract::Request,
    http::{
        StatusCode,
        header::{self, HeaderValue},
    },
    middleware::Next,
    response::Response,
};

/// The value for immutable, content-hashed assets: a year, never revalidated.
const IMMUTABLE_ONE_YEAR: &str = "public, max-age=31536000, immutable";
/// The value for stable but unhashed media, the web fonts: thirty days.
const THIRTY_DAYS: &str = "public, max-age=2592000";
/// The value for generated metadata and the resume PDFs: one hour.
const ONE_HOUR: &str = "public, max-age=3600";
/// The value for documents: cacheable, but revalidated against the origin on every use, so a new
/// deploy is served the moment it lands.
const REVALIDATE: &str = "no-cache";
/// The value for any `4xx` or `5xx`: never stored.
///
/// The TTLs above describe the asset a path names, not a failure to serve it, and a cache holds
/// an error response with explicit freshness exactly as long as it would the asset (RFC 9111 §3).
const NO_STORE: &str = "no-store";

/// Picks the `Cache-Control` value for a static-asset request path.
fn cache_control_for(path: &str) -> &'static str {
    if is_content_hashed(path) {
        return IMMUTABLE_ONE_YEAR;
    }
    if has_extension(path, "woff2") {
        return THIRTY_DAYS;
    }
    if path.starts_with("/resume/")
        || path.ends_with("robots.txt")
        || path.ends_with("sitemap.xml")
        // The generated Open Graph card is served unhashed from a fixed path
        // because the meta tag naming it has to stay stable.
        || ["pdf", "webmanifest", "svg", "png", "css"]
            .into_iter()
            .any(|ext| has_extension(path, ext))
    {
        return ONE_HOUR;
    }
    // The SSR HTML, the fallback, and anything else: revalidate.
    REVALIDATE
}

/// Reports whether the path's final segment ends in `.<ext>`, ignoring ASCII case.
fn has_extension(path: &str, ext: &str) -> bool {
    std::path::Path::new(path)
        .extension()
        .is_some_and(|found| found.eq_ignore_ascii_case(ext))
}

/// Reports whether the path's file name carries a `dx`/manganis content hash: a `dxh`-prefixed
/// segment of 1 to 16 hex digits at the end of the stem, as in `tailwind-dxh31cffa1cc71c7bb.css`.
// manganis renders the 64-bit asset hash as plain hex, so a hash with leading zero nibbles is
// written short: one real `dx bundle` produced a 16-digit `.js` and a 15-digit `.wasm` side by
// side. Requiring exactly 16 left the wasm binary and the stylesheet revalidated on every visit.
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

/// Returns the `Cache-Control` value for a response with `status` to a path classified as
/// `for_path`: the path's TTL, or [`NO_STORE`] for an error.
// Without the exception a missing font was a `404` every browser and shared cache kept for
// thirty days, and an asset requested from a replica that does not have it yet, such as an old
// pod during a rolling deploy, was an error pinned for as long as its TTL.
fn cache_control_for_response(for_path: &'static str, status: StatusCode) -> &'static str {
    if status.is_client_error() || status.is_server_error() {
        NO_STORE
    } else {
        for_path
    }
}

/// Sets the `Cache-Control` TTL for the request path unless the handler already provided one.
///
/// This is a response middleware. Error responses get `no-store` instead of the path's TTL.
pub async fn set_cache_control(request: Request, next: Next) -> Response {
    // Classified before the request is consumed. `cache_control_for` answers with
    // a `&'static str`, so carrying its verdict across the await costs nothing —
    // whereas carrying the path to classify it afterwards meant copying the path
    // onto the heap for every request the server answers, asset requests
    // included.
    let value = cache_control_for(request.uri().path());
    let mut response = next.run(request).await;

    if !response.headers().contains_key(header::CACHE_CONTROL) {
        let value = cache_control_for_response(value, response.status());
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
        "/assets/portfolio-web-dxh9c6f94a783aca5d3.js",
        "/assets/portfolio-web_bg-dxhfc275e0429871eb.wasm",
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
        assert!(is_content_hashed(
            "/assets/portfolio-web_bg-dxhfc275e0429871eb.wasm"
        ));
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
    fn errors_are_never_stored_whatever_the_path() {
        for status in [
            StatusCode::NOT_FOUND,
            StatusCode::INTERNAL_SERVER_ERROR,
            StatusCode::SERVICE_UNAVAILABLE,
        ] {
            for path in [
                "/assets/portfolio-web_bg-dxhfc275e0429871eb.wasm",
                "/fonts/jetbrains-mono-latin-400-normal.woff2",
                "/resume/cv.pdf",
                "/",
            ] {
                assert_eq!(
                    cache_control_for_response(cache_control_for(path), status),
                    NO_STORE,
                    "{status} for {path}"
                );
            }
        }
    }

    #[test]
    fn success_and_revalidation_keep_the_path_ttl() {
        for status in [
            StatusCode::OK,
            StatusCode::NOT_MODIFIED,
            StatusCode::PERMANENT_REDIRECT,
        ] {
            assert_eq!(
                cache_control_for_response(IMMUTABLE_ONE_YEAR, status),
                IMMUTABLE_ONE_YEAR
            );
        }
    }

    /// Through the middleware itself, so the status check is pinned where it runs.
    #[tokio::test]
    async fn the_middleware_marks_a_missing_asset_no_store() {
        use axum::{Router, body::Body, routing::get};
        use tower::ServiceExt;

        let app = Router::new()
            .route(
                "/fonts/present.woff2",
                get(|| async { (StatusCode::OK, "font") }),
            )
            .route(
                "/fonts/missing.woff2",
                get(|| async { StatusCode::NOT_FOUND }),
            )
            .layer(axum::middleware::from_fn(set_cache_control));
        let cache_control = |path: &'static str| {
            let app = app.clone();
            async move {
                let request = axum::http::Request::builder()
                    .uri(path)
                    .body(Body::empty())
                    .unwrap();
                app.oneshot(request).await.unwrap().headers()[header::CACHE_CONTROL].clone()
            }
        };

        assert_eq!(cache_control("/fonts/present.woff2").await, THIRTY_DAYS);
        assert_eq!(cache_control("/fonts/missing.woff2").await, NO_STORE);
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
