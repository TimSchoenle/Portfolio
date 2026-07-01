//! Native SSR server.
//!
//! Extends the Dioxus fullstack router (`dioxus::server::router`, which serves
//! the hydrating SSR page, static assets and any `#[server]` endpoints) with the
//! public JSON API, the SEO documents, and the security-header / compression /
//! cache-control layers that used to live in the standalone `apps/server` crate.

mod api;
mod assets;
mod cache;
mod seo;

use axum::{
    Router,
    http::{HeaderName, HeaderValue, header},
    middleware,
    routing::get,
};
use tower_http::{
    compression::CompressionLayer, set_header::SetResponseHeaderLayer, trace::TraceLayer,
};

/// Content-Security-Policy. `'unsafe-inline'` covers Dioxus's inline hydration
/// bootstrap and the serialized-state `<script>`; `'wasm-unsafe-eval'`
/// instantiates the WASM module. `'unsafe-eval'` is required because
/// `dioxus-web`'s document provider evaluates JavaScript via `new Function(...)`
/// (`js_sys::Function::new_with_args`) — this is how `document::Title`,
/// `document::Stylesheet`, `document::Link`, `document::Meta`, etc. are applied
/// on the client during hydration. Without it, `new Function` throws an
/// (uncaught) `EvalError` that aborts the wasm client and freezes client-side
/// navigation. Server-function calls are same-origin `fetch`, so
/// `connect-src 'self'` suffices.
const CSP: &str = "default-src 'self'; \
     script-src 'self' 'unsafe-inline' 'unsafe-eval' 'wasm-unsafe-eval'; \
     style-src 'self' 'unsafe-inline'; \
     font-src 'self'; \
     img-src 'self' data:; \
     connect-src 'self'; \
     object-src 'none'; \
     base-uri 'self'; \
     form-action 'self'; \
     frame-ancestors 'none'; \
     upgrade-insecure-requests";

/// Runs the Axum SSR server. `dioxus::serve` sets up the async runtime and the
/// default logger, then binds to `IP`/`PORT` (default `127.0.0.1:8080`) and
/// serves the router below with graceful shutdown.
pub fn serve() {
    dioxus::serve(|| async move { Ok(router()) });
}

/// The full application router: the Dioxus SSR/asset router with our routes and
/// layers mounted on top. Custom routes take precedence over the SSR fallback;
/// layers apply to every response (SSR HTML, static assets, API, SEO).
fn router() -> Router {
    let static_header = |name: HeaderName, value: &'static str| {
        SetResponseHeaderLayer::overriding(name, HeaderValue::from_static(value))
    };

    dioxus::server::router(crate::app::App)
        .merge(api_router())
        .merge(assets::router())
        .layer(static_header(header::CONTENT_SECURITY_POLICY, CSP))
        .layer(static_header(header::X_CONTENT_TYPE_OPTIONS, "nosniff"))
        .layer(static_header(header::X_FRAME_OPTIONS, "DENY"))
        .layer(static_header(header::REFERRER_POLICY, "no-referrer"))
        .layer(static_header(
            header::STRICT_TRANSPORT_SECURITY,
            "max-age=31536000; includeSubDomains; preload",
        ))
        .layer(static_header(
            HeaderName::from_static("permissions-policy"),
            "camera=(), microphone=(), geolocation=(), interest-cohort=()",
        ))
        // Compress text assets per Accept-Encoding; already-encoded payloads are
        // skipped, so this never double-compresses the SSR HTML.
        .layer(CompressionLayer::new())
        // Assign a `Cache-Control` TTL per asset class, but only when a handler
        // did not already set one (so the API's own headers win). Runs last.
        .layer(middleware::from_fn(cache::set_cache_control))
        .layer(TraceLayer::new_for_http())
}

/// The public HTTP API + SEO documents, as a standalone sub-router so it can be
/// exercised in isolation by the integration tests below (without the SSR
/// render machinery).
fn api_router() -> Router {
    Router::new()
        .route("/api/health", get(api::health))
        .route("/api/health/live", get(api::live))
        .route("/api/health/ready", get(api::ready))
        // Short, kubelet-friendly aliases for the conventional probe paths.
        .route("/livez", get(api::live))
        .route("/readyz", get(api::ready))
        .route("/api/v1/profile", get(api::profile))
        .route("/api/v1/profile/schema", get(api::schema))
        .route("/robots.txt", get(seo::robots))
        .route("/sitemap.xml", get(seo::sitemap))
        .route("/site.webmanifest", get(seo::webmanifest))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
    };
    use portfolio_data::CONFIG;
    use serde_json::Value;
    use tower::ServiceExt;

    /// Dispatches a `GET` through the real API router and returns the response.
    async fn get_(path: &str) -> axum::response::Response {
        api_router()
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    async fn json(response: axum::response::Response) -> Value {
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn health_route_is_wired_and_uncached() {
        let response = get_("/api/health").await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
        assert_eq!(json(response).await["status"], "healthy");
    }

    #[tokio::test]
    async fn liveness_routes_are_wired_and_uncached() {
        for path in ["/api/health/live", "/livez"] {
            let response = get_(path).await;
            assert_eq!(response.status(), StatusCode::OK, "{path}");
            assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
            assert_eq!(json(response).await["status"], "alive");
        }
    }

    #[tokio::test]
    async fn readiness_routes_are_wired_and_uncached() {
        for path in ["/api/health/ready", "/readyz"] {
            let response = get_(path).await;
            assert!(
                matches!(
                    response.status(),
                    StatusCode::OK | StatusCode::SERVICE_UNAVAILABLE
                ),
                "{path}: unexpected status {}",
                response.status()
            );
            assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
        }
    }

    #[tokio::test]
    async fn profile_route_serves_the_cached_document() {
        let response = get_("/api/v1/profile").await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers()[header::CACHE_CONTROL],
            "public, max-age=3600"
        );
        let doc = json(response).await;
        assert_eq!(doc["email"], CONFIG.email);
        assert_eq!(
            doc["$schema"],
            format!("{}{}", CONFIG.url, portfolio_data::profile::SCHEMA_PATH)
        );
    }

    #[tokio::test]
    async fn schema_route_describes_the_profile() {
        let response = get_("/api/v1/profile/schema").await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(json(response).await["type"], "object");
    }

    #[tokio::test]
    async fn seo_routes_are_wired() {
        assert_eq!(get_("/robots.txt").await.status(), StatusCode::OK);
        assert_eq!(get_("/sitemap.xml").await.status(), StatusCode::OK);
        assert_eq!(get_("/site.webmanifest").await.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn unknown_api_path_is_not_found() {
        assert_eq!(
            get_("/api/does-not-exist").await.status(),
            StatusCode::NOT_FOUND
        );
    }

    #[tokio::test]
    async fn profile_rejects_non_get_methods() {
        let response = api_router()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/profile")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }
}
