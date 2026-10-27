//! The static files served at fixed paths from bytes embedded in the server binary.
//!
//! `fonts.css`, the masthead logo and the web manifest reference the favicon and the vendored
//! woff2 fonts by absolute URL (`/favicon.svg`, `/fonts/…`), so those files live at fixed paths,
//! not at the content-hashed paths `dx`/manganis produces. Embedding them keeps the server one
//! self-contained binary. [`crate::cache::set_cache_control`] fills in their `Cache-Control`, and
//! each response carries an `ETag` so a lapsed TTL revalidates to `304`.
//!
//! The application embeds the bytes at compile time and hands them to [`router`] as
//! [`EmbeddedAssets`]. An empty entry is a file whose generator has not run, as in a development
//! build, and is answered with `404`.

use std::future::ready;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;

use axum::{
    Router,
    extract::Path,
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use portfolio_data::OG_IMAGE_FILE;

/// A file name and the bytes served under it.
pub type NamedFile = (&'static str, &'static [u8]);

/// The files [`router`] serves, as the application embedded them.
#[derive(Debug, Clone, Copy)]
pub struct EmbeddedAssets {
    /// The SVG favicon, served at `/favicon.svg`.
    pub favicon: &'static [u8],
    /// The generated Open Graph card, served at `/` + [`OG_IMAGE_FILE`].
    ///
    /// Empty when the resume generator has not run.
    pub og_image: &'static [u8],
    /// The vendored woff2 fonts, served under `/fonts/` by names that must match `fonts.css`.
    pub fonts: &'static [NamedFile],
    /// One resume PDF per site language, served under `/resume/` by its published name.
    ///
    /// An entry is empty when the resume generator has not run.
    pub resumes: &'static [NamedFile],
    /// The web manifest's raster icons, each served at `/` plus its name.
    ///
    /// An entry is empty when the resume generator has not run.
    pub app_icons: &'static [NamedFile],
}

/// One servable file: its bytes, their media type, and the entity tag derived from them.
#[derive(Clone)]
struct Asset {
    content_type: &'static str,
    bytes: &'static [u8],
    etag: HeaderValue,
}

impl Asset {
    /// Wraps `bytes`, hashing them once for the entity tag.
    fn new(content_type: &'static str, bytes: &'static [u8]) -> Self {
        Self {
            content_type,
            bytes,
            etag: etag_for(bytes),
        }
    }

    /// Serves the file, answering `304 Not Modified` when the client already holds the bytes.
    ///
    /// These files sit at stable, unhashed paths, so the `Cache-Control` TTL the cache
    /// middleware assigns them is the only thing keeping them off the wire, and once it lapses
    /// the browser revalidates. Without a validator the origin has no answer but the entire body
    /// again: the resume PDFs paid that in full every hour, and the fonts every thirty days.
    fn serve(&self, headers: &HeaderMap) -> Response {
        if self.bytes.is_empty() {
            return StatusCode::NOT_FOUND.into_response();
        }
        let mut response = if is_current(headers, &self.etag) {
            StatusCode::NOT_MODIFIED.into_response()
        } else {
            ([(header::CONTENT_TYPE, self.content_type)], self.bytes).into_response()
        };
        response
            .headers_mut()
            .insert(header::ETAG, self.etag.clone());
        response
    }
}

/// Files served under a directory by name, looked up from a path capture.
///
/// A lookup, not one route per file, so a name nothing is published under is this router's `404`
/// and not a page render. Only a listed name is ever served, which rules out path traversal.
type Directory = Arc<[(&'static str, Asset)]>;

/// Wraps every file in `files` as an [`Asset`] of `content_type`.
fn directory(files: &[NamedFile], content_type: &'static str) -> Directory {
    files
        .iter()
        .map(|&(name, bytes)| (name, Asset::new(content_type, bytes)))
        .collect()
}

/// Serves the file named `name` from `directory`, or `404`.
fn serve_named(directory: &Directory, name: &str, headers: &HeaderMap) -> Response {
    match directory.iter().find(|(known, _)| *known == name) {
        Some((_, asset)) => asset.serve(headers),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

/// Builds the routes for the fixed-path embedded assets.
///
/// Every entity tag is computed here, once: the bytes are compile-time constants, so the answer
/// cannot change while the process lives.
pub fn router(assets: &EmbeddedAssets) -> Router {
    let single = |router: Router, path: &str, asset: Asset| {
        router.route(
            path,
            get(move |headers: HeaderMap| ready(asset.serve(&headers))),
        )
    };

    // One exact route per icon rather than a `/{file}` capture, which would also claim every
    // page path (`/licenses`) that the Dioxus fallback has to answer.
    let mut router = Router::new();
    for &(name, bytes) in assets.app_icons {
        router = single(router, &format!("/{name}"), Asset::new("image/png", bytes));
    }
    router = single(
        router,
        "/favicon.svg",
        Asset::new("image/svg+xml", assets.favicon),
    );
    router = single(
        router,
        &format!("/{OG_IMAGE_FILE}"),
        Asset::new("image/png", assets.og_image),
    );

    let fonts = directory(assets.fonts, "font/woff2");
    let resumes = directory(assets.resumes, "application/pdf");
    router
        .route(
            "/fonts/{file}",
            get(move |Path(file): Path<String>, headers: HeaderMap| {
                ready(serve_named(&fonts, &file, &headers))
            }),
        )
        .route(
            "/resume/{file}",
            get(move |Path(file): Path<String>, headers: HeaderMap| {
                ready(serve_named(&resumes, &file, &headers))
            }),
        )
}

/// Derives the entity tag for one embedded file, the same on every replica.
// A non-cryptographic hash is enough: an `ETag` only has to differ when the bytes differ, and
// nothing treats it as a security boundary. `DefaultHasher::new` is seeded with fixed keys, not a
// per-process random state, so every replica derives the same tag for the same bytes and a shared
// cache in front of the origin sees one validator.
fn etag_for(bytes: &[u8]) -> HeaderValue {
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    HeaderValue::try_from(format!("\"{:016x}\"", hasher.finish()))
        .expect("a quoted hex digest is a valid header value")
}

/// Reports whether the request's `If-None-Match` names `etag`, the entity about to be sent.
///
/// `*` matches whatever the origin holds, per RFC 9110, and a `W/` prefix is accepted because
/// `If-None-Match` uses the weak comparison.
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    const FIXTURE: EmbeddedAssets = EmbeddedAssets {
        favicon: b"<svg/>",
        og_image: b"",
        fonts: &[
            ("space-grotesk-latin-400-normal.woff2", b"wOF2 regular"),
            ("jetbrains-mono-latin-400-normal.woff2", b"wOF2 mono"),
        ],
        resumes: &[("Resume.pdf", b"%PDF"), ("Lebenslauf.pdf", b"")],
        app_icons: &[("icon-192.png", b"\x89PNG")],
    };

    fn router() -> Router {
        super::router(&FIXTURE)
    }

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

    /// A file whose generator has not run is a `404`, never an empty `200`.
    #[tokio::test]
    async fn an_empty_entry_is_not_found() {
        assert_eq!(status("/og-image.png").await, StatusCode::NOT_FOUND);
        assert_eq!(
            status("/resume/Lebenslauf.pdf").await,
            StatusCode::NOT_FOUND
        );
        assert_eq!(status("/resume/Resume.pdf").await, StatusCode::OK);
        assert_eq!(status("/icon-192.png").await, StatusCode::OK);
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
