//! Which responses the server compresses.

use axum::http::{StatusCode, header};
use tower_http::compression::predicate::{NotForContentType, Predicate, SizeAbove};

/// The response-compression predicate: which responses are worth the CPU.
///
/// Assembled from tower-http's parts rather than started from its
/// `DefaultPredicate`, because one of the exclusions that default bundles has to
/// be *narrowed*. `NotForContentType::IMAGES` skips everything under `image/`,
/// which silently included this site's `image/svg+xml` favicon — markup, and the
/// one image here that is worth compressing. Predicates compose only with AND,
/// so an exclusion cannot be taken back once it is in; the rule is therefore
/// stated directly (see [`not_for_raster_images`]) instead of being applied and
/// then undone.
///
/// The remaining default exclusions are kept verbatim — tiny bodies, gRPC and
/// server-sent-event streams — and two of our own are added: woff2 fonts and the
/// resume PDFs are already compressed, so running br/gzip over them only burns
/// CPU and can grow the payload rather than shrink it.
pub(super) fn compression_predicate() -> impl Predicate {
    SizeAbove::default()
        .and(NotForContentType::GRPC)
        .and(NotForContentType::SSE)
        .and(not_for_raster_images)
        .and(NotForContentType::const_new("font/woff2"))
        .and(NotForContentType::const_new("application/pdf"))
}

/// The raster half of tower-http's `NotForContentType::IMAGES`: everything under
/// `image/` is skipped except `image/svg+xml`, which is text and compresses to
/// roughly a third of its size.
///
/// Written as a bare `fn` because `Predicate` is implemented for any
/// `Fn(StatusCode, Version, &HeaderMap, &Extensions) -> bool + Clone`, which is
/// the whole signature — the body is never inspected, so nothing here needs the
/// `http-body` trait bound a manual `impl Predicate` would.
fn not_for_raster_images(
    _status: StatusCode,
    _version: axum::http::Version,
    headers: &axum::http::HeaderMap,
    _extensions: &axum::http::Extensions,
) -> bool {
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    !content_type.starts_with("image/") || content_type.starts_with("image/svg+xml")
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;

    /// Builds a response with the given content type and a body large enough to
    /// clear the default size threshold.
    fn typed_response(content_type: &str) -> axum::response::Response {
        axum::http::Response::builder()
            .header(header::CONTENT_TYPE, content_type)
            .header(header::CONTENT_LENGTH, "5000")
            .body(Body::from(vec![0u8; 5000]))
            .unwrap()
    }

    #[test]
    fn compression_skips_already_compressed_assets() {
        let predicate = compression_predicate();
        // Already-compressed payloads must not be re-compressed.
        assert!(!predicate.should_compress(&typed_response("font/woff2")));
        assert!(!predicate.should_compress(&typed_response("application/pdf")));
        // Text assets and JSON are still compressed.
        assert!(predicate.should_compress(&typed_response("text/html; charset=utf-8")));
        assert!(predicate.should_compress(&typed_response("text/css")));
        assert!(predicate.should_compress(&typed_response("application/json")));
        assert!(predicate.should_compress(&typed_response("application/wasm")));
        // The defaults this predicate is assembled from are still in force.
        assert!(!predicate.should_compress(&typed_response("text/event-stream")));
        assert!(!predicate.should_compress(&typed_response("application/grpc")));
    }

    /// An SVG is markup, not a raster image. tower-http's default predicate skips
    /// everything under `image/`, which took the favicon with it — this asserts
    /// the narrowing that puts it back without admitting the raster types.
    #[test]
    fn svg_is_compressed_but_raster_images_are_not() {
        let predicate = compression_predicate();
        assert!(predicate.should_compress(&typed_response("image/svg+xml")));
        assert!(predicate.should_compress(&typed_response("image/svg+xml; charset=utf-8")));
        for raster in ["image/png", "image/jpeg", "image/webp", "image/avif"] {
            assert!(
                !predicate.should_compress(&typed_response(raster)),
                "{raster} should not be compressed"
            );
        }
    }
}
