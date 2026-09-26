//! Stable-path static assets embedded in the server binary.
//!
//! The favicon and the vendored woff2 fonts are referenced by absolute URLs
//! (`/favicon.svg`, `/fonts/…`) from `fonts.css`, the masthead logo and the web
//! manifest, so they must live at fixed paths rather than the content-hashed
//! paths `dx`/manganis produces. Embedding them keeps the runtime a single
//! self-contained binary (no asset directory to ship alongside it). Their
//! `Cache-Control` is filled in by the cache middleware (see `cache.rs`).

use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::LazyLock;

use axum::{
    Router,
    extract::Path,
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use portfolio_data::OG_IMAGE_FILE;

const FAVICON: &[u8] = include_bytes!("../../assets/favicon.svg");

/// The generated Open Graph card, embedded by `build.rs`. Empty in dev builds
/// where the resume generator has not run, in which case the route replies 404 —
/// the same contract as the resume PDFs below.
const OG_IMAGE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/og-image.png"));

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

/// An entity tag for one of the compile-time assets below.
///
/// A non-cryptographic hash is the right tool here: an `ETag` only has to differ
/// when the bytes differ, and nothing downstream treats it as a security
/// boundary — so this needs no digest crate linked into the server for it.
/// `DefaultHasher::new` is seeded with fixed keys rather than a per-process
/// random state, which is what makes every replica of a deployment derive the
/// same tag for the same bytes; a shared cache in front of the origin would
/// otherwise see a different validator from each pod.
fn etag_for(bytes: &[u8]) -> HeaderValue {
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    HeaderValue::try_from(format!("\"{:016x}\"", hasher.finish()))
        .expect("a quoted hex digest is a valid header value")
}

/// The entity tags for every embedded asset, derived once on first use.
///
/// Hashing at start-up rather than per request: these are `include_bytes!`
/// constants, so the answer cannot change while the process lives.
static FAVICON_ETAG: LazyLock<HeaderValue> = LazyLock::new(|| etag_for(FAVICON));
static OG_IMAGE_ETAG: LazyLock<HeaderValue> = LazyLock::new(|| etag_for(OG_IMAGE));
static FONT_ETAGS: LazyLock<Vec<HeaderValue>> =
    LazyLock::new(|| FONTS.iter().map(|(_, bytes)| etag_for(bytes)).collect());
static RESUME_ETAGS: LazyLock<Vec<HeaderValue>> = LazyLock::new(|| {
    RESUMES
        .iter()
        .map(|(_, _, bytes)| etag_for(bytes))
        .collect()
});
static APP_ICON_ETAGS: LazyLock<Vec<HeaderValue>> = LazyLock::new(|| {
    APP_ICON_FILES
        .iter()
        .map(|(_, bytes)| etag_for(bytes))
        .collect()
});

/// Serves an embedded asset, answering `304 Not Modified` when the client
/// already holds the bytes.
///
/// These assets sit at stable, unhashed paths, so the `Cache-Control` TTL that
/// `cache.rs` assigns them is the only thing keeping them off the wire — and
/// once it lapses the browser revalidates. With no validator to compare against,
/// the origin had no answer but the entire body again: the resume PDFs paid that
/// in full every hour, and the fonts every thirty days.
fn embedded(
    headers: &HeaderMap,
    content_type: &'static str,
    etag: &HeaderValue,
    bytes: &'static [u8],
) -> Response {
    let mut response = if is_current(headers, etag) {
        StatusCode::NOT_MODIFIED.into_response()
    } else {
        ([(header::CONTENT_TYPE, content_type)], bytes).into_response()
    };
    response.headers_mut().insert(header::ETAG, etag.clone());
    response
}

/// Whether the request's `If-None-Match` names the entity we are about to send.
///
/// `*` matches whatever the origin holds, per RFC 9110, and a `W/` prefix is
/// accepted because `If-None-Match` is defined to use the weak comparison — a
/// cache that revalidates a strong tag weakly is still asking about these exact
/// bytes.
fn is_current(headers: &HeaderMap, etag: &HeaderValue) -> bool {
    let Some(value) = headers
        .get(header::IF_NONE_MATCH)
        .and_then(|value| value.to_str().ok())
    else {
        return false;
    };
    let etag = etag.to_str().unwrap_or_default();
    value
        .split(',')
        .map(str::trim)
        .any(|candidate| candidate == "*" || candidate.trim_start_matches("W/") == etag)
}

/// Routes for the fixed-path embedded assets.
pub fn router() -> Router {
    // One exact route per icon rather than a `/{file}` capture, which would also claim every
    // page path (`/licenses`) that the Dioxus fallback has to answer.
    let icons = (0..APP_ICON_FILES.len()).fold(Router::new(), |router, index| {
        router.route(
            &format!("/{}", APP_ICON_FILES[index].0),
            get(move |headers: HeaderMap| std::future::ready(app_icon(index, &headers))),
        )
    });
    icons
        .route("/favicon.svg", get(favicon))
        .route(&format!("/{OG_IMAGE_FILE}"), get(og_image))
        .route("/fonts/{file}", get(font))
        .route("/resume/{file}", get(resume))
}

async fn favicon(headers: HeaderMap) -> Response {
    embedded(&headers, "image/svg+xml", &FAVICON_ETAG, FAVICON)
}

/// The social card the `og:image` meta tag points at. 404 when the generator has
/// not run, so a dev build advertises an image it does not have rather than
/// serving an empty one that every consumer would reject anyway.
async fn og_image(headers: HeaderMap) -> Response {
    if OG_IMAGE.is_empty() {
        return StatusCode::NOT_FOUND.into_response();
    }
    embedded(&headers, "image/png", &OG_IMAGE_ETAG, OG_IMAGE)
}

async fn font(Path(file): Path<String>, headers: HeaderMap) -> Response {
    match FONTS.iter().position(|(name, _)| *name == file) {
        Some(index) => embedded(&headers, "font/woff2", &FONT_ETAGS[index], FONTS[index].1),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

// Build-generated resume PDFs and app icons, embedded by `build.rs` from tables it derives from
// `portfolio_data`, so a language or an icon added there is served here without an edit. Each
// entry is empty when the generator has not run (development builds), and an empty entry is a
// 404 rather than an empty body. Embedding keeps the server a single self-contained binary and
// sidesteps the non-ASCII resume file names (`Tim-Schönle-Lebenslauf.pdf`) that on-disk static
// serving failed to resolve.
include!(concat!(env!("OUT_DIR"), "/resumes.rs"));
include!(concat!(env!("OUT_DIR"), "/app_icons.rs"));

/// Serves an embedded resume PDF by its exact published file name.
///
/// Accepting only the published names keeps the handler total and rules out path traversal.
async fn resume(Path(file): Path<String>, headers: HeaderMap) -> Response {
    match RESUMES.iter().position(|(_, name, _)| *name == file) {
        Some(index) if !RESUMES[index].2.is_empty() => embedded(
            &headers,
            "application/pdf",
            &RESUME_ETAGS[index],
            RESUMES[index].2,
        ),
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

/// Serves the web manifest icon at `index` in [`APP_ICON_FILES`]; 404 when the generator has
/// not run.
fn app_icon(index: usize, headers: &HeaderMap) -> Response {
    let (_, bytes) = APP_ICON_FILES[index];
    if bytes.is_empty() {
        return StatusCode::NOT_FOUND.into_response();
    }
    embedded(headers, "image/png", &APP_ICON_ETAGS[index], bytes)
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

    async fn get(path: &str, if_none_match: Option<&str>) -> axum::response::Response {
        let mut builder = Request::builder().uri(path);
        if let Some(value) = if_none_match {
            builder = builder.header(header::IF_NONE_MATCH, value);
        }
        router()
            .oneshot(builder.body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    /// A conditional request for an asset the client already holds is answered
    /// with the tag and no body. Without the tag there is nothing to condition
    /// on, so every lapsed TTL re-sent the whole thing.
    #[tokio::test]
    async fn an_unchanged_asset_revalidates_to_304() {
        let first = get("/favicon.svg", None).await;
        assert_eq!(first.status(), StatusCode::OK);
        let etag = first.headers()[header::ETAG].to_str().unwrap().to_owned();
        assert!(
            etag.starts_with('"') && etag.ends_with('"'),
            "an entity tag is quoted: {etag}"
        );

        // The tag itself, the wildcard, the weak form, and a list containing it.
        let accepted = [
            etag.clone(),
            "*".to_string(),
            format!("W/{etag}"),
            format!("\"something-else\", {etag}"),
        ];
        for candidate in accepted {
            let response = get("/favicon.svg", Some(&candidate)).await;
            assert_eq!(
                response.status(),
                StatusCode::NOT_MODIFIED,
                "If-None-Match: {candidate}"
            );
            // The validator is repeated on the 304 so the cache can refresh it.
            assert_eq!(response.headers()[header::ETAG], etag.as_str());
        }

        // A tag the client does not hold still gets the bytes.
        let stale = get("/favicon.svg", Some("\"0000000000000000\"")).await;
        assert_eq!(stale.status(), StatusCode::OK);
    }

    /// Distinct assets get distinct tags. Sharing one would let a cache answer a
    /// request for a font that changed with the copy of another that did not.
    #[tokio::test]
    async fn each_asset_carries_its_own_tag() {
        let favicon = get("/favicon.svg", None).await;
        let font = get("/fonts/space-grotesk-latin-400-normal.woff2", None).await;
        assert_eq!(font.status(), StatusCode::OK);
        assert_ne!(
            favicon.headers()[header::ETAG],
            font.headers()[header::ETAG]
        );
    }

    #[tokio::test]
    async fn unknown_font_is_not_found() {
        assert_eq!(
            status("/fonts/not-a-font.woff2").await,
            StatusCode::NOT_FOUND
        );
    }

    #[tokio::test]
    async fn resume_rejects_names_outside_the_allowlist() {
        // Only the published resume names are accepted; anything else —
        // including path-traversal attempts — is refused before any disk access.
        assert_eq!(
            status("/resume/anything-else.pdf").await,
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            status("/resume/..%2f..%2fCargo.toml").await,
            StatusCode::NOT_FOUND
        );
    }
}
