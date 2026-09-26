//! Native SSR server.
//!
//! Extends the Dioxus fullstack router (`dioxus::server::router`, which serves the hydrating SSR
//! page, static assets and any `#[server]` endpoints) with the public JSON API, the SEO
//! documents, and the security-header, compression and cache-control layers.
//!
//! - `page` finishes every document: language, cookie, policy, and the memo that answers a
//!   repeat request without rendering;
//! - `isr` is the on-disk incremental render cache and the allowlist keeping it bounded;
//! - `csp`, `cache`, `assets`, `api`, `seo`, `legal` and `telemetry` are what
//!   their names say.

mod api;
mod assets;
mod cache;
mod csp;
mod isr;
pub(crate) mod legal;
mod page;
mod seo;
mod telemetry;

use std::sync::Arc;

use axum::{
    Router,
    http::{HeaderName, HeaderValue, StatusCode, header},
    middleware,
    routing::get,
};
use portfolio_config::{AssetsConfig, ServerConfig};
use terrace_legal::Legal;
use tower_http::{
    compression::{
        CompressionLayer,
        predicate::{NotForContentType, Predicate, SizeAbove},
    },
    set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
};

use isr::PagePaths;
use page::{PageMemo, PageState};

/// Exit code for a configuration the process cannot start with (`EX_CONFIG` from `sysexits.h`),
/// so an operator can tell a bad config apart from a crash without reading the logs.
const EX_CONFIG: i32 = 78;

/// Runs the Axum SSR server. `dioxus::serve` sets up the async runtime and the
/// default logger, then binds to `IP`/`PORT` (default `127.0.0.1:8080`) and
/// serves the router below.
///
/// Configuration is read once, before the runtime exists: a value that cannot
/// be loaded is a start-up failure, not a per-request one, and failing here
/// means the container never reports ready rather than serving a degraded site.
pub fn serve() {
    let config = Arc::new(load_config());
    // The legal catalog is part of the configuration's validity: a deployment without an
    // imprint and a privacy notice in every language is refused here, with every missing key
    // named, before anything binds a port.
    let legal = match legal::build(&config.legal) {
        Ok(legal) => legal,
        Err(issues) => refuse(&issues),
    };
    legal::install(legal.clone());
    let paths = PagePaths::new(&legal::hosted_paths(&legal));

    // Before `dioxus::serve`, and that ordering is the whole of the handover: the framework
    // installs its own subscriber unless one is already set, and a Sentry layer has to be a
    // layer *of* the subscriber rather than something added to a finished one. With
    // `sentry.enabled` off this installs nothing and the framework's subscriber stands.
    //
    // The binding is held for the rest of `serve`, which is the rest of the process —
    // `dioxus::serve` diverges. See `telemetry::TelemetryGuard` for why that means the
    // drop-time flush is unreachable, and why no key claims otherwise.
    let _telemetry = match telemetry::init(&config.sentry) {
        Ok(guard) => guard,
        Err(err) => refuse(&err),
    };

    dioxus::serve(move || {
        let config = Arc::clone(&config);
        let legal = legal.clone();
        let paths = paths.clone();
        async move { Ok(router(&config, legal, paths)) }
    });
}

/// Ends the process with [`EX_CONFIG`], naming what could not be started with.
///
/// Runs before `dioxus::serve` installs a logger — and, when Sentry is on, before this server
/// installs one — so `tracing` would discard this; it goes to stderr directly.
///
/// The error names the key; the report under it names the layer that supplied it, which is the
/// half an operator cannot get at from inside a distroless image with no shell. Neither holds a
/// configuration value, so both are safe in a log that is shipped and retained.
fn refuse(err: &dyn std::fmt::Display) -> ! {
    eprintln!("portfolio: cannot start, the configuration is not usable: {err}");
    eprintln!("{}", portfolio_config::provenance());
    std::process::exit(EX_CONFIG)
}

/// Reads the configuration, or ends the process with [`EX_CONFIG`].
///
/// Two ways to be unusable, and both are start-up failures: a value that cannot be *loaded* (a
/// missing file, an unparseable number, one key supplied by two layers), and a set of values that
/// load individually but cannot be *served* together — see
/// [`CspConfig::validate`](portfolio_config::CspConfig::validate) and
/// [`SentryConfig::validate`](portfolio_config::SentryConfig::validate) and
/// [`HstsConfig::validate`](portfolio_config::HstsConfig::validate). Failing here means the
/// container never reports ready, rather than serving every visitor a blank page or reporting its
/// errors into a void.
fn load_config() -> ServerConfig {
    let config = match portfolio_config::load::<ServerConfig>() {
        Ok(config) => config,
        Err(err) => refuse(&err),
    };
    if let Err(err) = config.csp.validate() {
        refuse(&err);
    }
    if let Err(err) = config.sentry.validate() {
        refuse(&err);
    }
    if let Err(err) = config.hsts.validate() {
        refuse(&err);
    }
    config
}

/// The full application router: the Dioxus SSR/asset router with our routes and
/// layers mounted on top. Custom routes take precedence over the SSR fallback;
/// layers apply to every response (SSR HTML, static assets, API, SEO).
///
/// `legal` serves the JSON routes and the sitemap; `paths` is the render cache's allowlist, built
/// once in [`serve`] from the same catalog.
fn router(config: &ServerConfig, legal: Legal, paths: PagePaths) -> Router {
    let static_header = |name: HeaderName, value: &'static str| {
        SetResponseHeaderLayer::overriding(name, HeaderValue::from_static(value))
    };

    let policy = Arc::new(csp::SitePolicy::new(&config.csp));
    let (dioxus_app, isr_enabled) = isr::dioxus_app_router(&config.isr, &paths);
    let app = dioxus_app
        .merge(api_router(config.assets.clone(), legal))
        .merge(assets::router());
    // Page handling, innermost so it sees the request before the router and the response
    // before compression. See `page::localize_page`.
    let app = app.layer(middleware::from_fn_with_state(
        PageState {
            isr_enabled,
            policy: Arc::clone(&policy),
            pages: Arc::new(PageMemo::default()),
            paths,
            ttl: config.isr.invalidate_after(),
        },
        page::localize_page,
    ));

    // The policy for everything that is not a document, and only where the layer
    // above has not already set a stricter, document-specific one — hence
    // `if_not_present` rather than the `overriding` every other header uses.
    let app = app
        .layer(SetResponseHeaderLayer::if_not_present(
            header::CONTENT_SECURITY_POLICY,
            policy.subresource(),
        ))
        .layer(static_header(header::X_CONTENT_TYPE_OPTIONS, "nosniff"))
        .layer(static_header(header::X_FRAME_OPTIONS, "DENY"))
        .layer(static_header(header::REFERRER_POLICY, "no-referrer"))
        .layer(SetResponseHeaderLayer::overriding(
            header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::try_from(config.hsts.header_value())
                .expect("a rendered HSTS value is ASCII by construction"),
        ))
        // Isolates the browsing context from any cross-origin window that opens it or that it
        // opens, and keeps other origins from embedding this site's resources. Neither is
        // relaxed anywhere: every resource the site loads is its own, and every outbound link
        // already opens with `noopener`.
        .layer(static_header(
            HeaderName::from_static("cross-origin-opener-policy"),
            "same-origin",
        ))
        .layer(static_header(
            HeaderName::from_static("cross-origin-resource-policy"),
            "same-origin",
        ))
        .layer(static_header(
            HeaderName::from_static("permissions-policy"),
            "camera=(), microphone=(), geolocation=(), interest-cohort=()",
        ))
        // Compress text assets per Accept-Encoding; already-encoded payloads are
        // skipped, so this never double-compresses the SSR HTML.
        .layer(CompressionLayer::new().compress_when(compression_predicate()))
        // Assign a `Cache-Control` TTL per asset class, but only when a handler
        // did not already set one (so the API's own headers win). Runs last.
        .layer(middleware::from_fn(cache::set_cache_control));

    // Outside everything that can fail, so a panic or a 500 from any layer above is still
    // reported with its request attached. Absent entirely when Sentry is off, rather than
    // present as a no-op: a request must not pay for a feature nobody switched on.
    //
    // Two layers, and the order between them matters — the last `.layer` call is the outermost,
    // so the hub is bound first and the metadata layer below writes onto the hub it bound.
    // Mounted through `Router::layer`, which runs after routing, which is what puts the
    // `MatchedPath` extension in place for the route-named transaction.
    let app = match telemetry::http_layers(&config.sentry) {
        Some((hub, http)) => app.layer(http).layer(hub),
        None => app,
    };

    app.layer(TraceLayer::new_for_http())
}

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
fn compression_predicate() -> impl Predicate {
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

/// The public HTTP API + SEO documents, as a standalone sub-router so it can be
/// exercised in isolation by the integration tests below (without the SSR
/// render machinery).
///
/// `assets` is the router's state for the readiness probe, and `legal` serves the
/// legal JSON routes and decides which documents the sitemap lists. Both are
/// configuration handed in rather than re-read per request, so neither can start
/// disagreeing with the rest of the process.
fn api_router(assets: AssetsConfig, legal: Legal) -> Router {
    let sitemap = axum::body::Bytes::from(seo::sitemap_xml(&legal::hosted_paths(&legal)));
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
        .route(
            "/sitemap.xml",
            get(move || std::future::ready(seo::sitemap(sitemap.clone()))),
        )
        .route("/site.webmanifest", get(seo::webmanifest))
        .with_state(assets)
        .merge(legal::router(legal))
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
        api_router(AssetsConfig::default(), legal::tests::fixture())
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    async fn bytes(response: axum::response::Response) -> axum::body::Bytes {
        to_bytes(response.into_body(), usize::MAX).await.unwrap()
    }

    async fn json(response: axum::response::Response) -> Value {
        serde_json::from_slice(&bytes(response).await).unwrap()
    }

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

    /// The two Sentry middlewares are mounted on the live stack in [`router`], where nothing else
    /// in this suite reaches — that function builds the Dioxus app router, which wants a bundle on
    /// disk. What is worth pinning without one is that the pair is transparent: the hub layer and
    /// the request-metadata layer sit outside every handler, and a response that goes through them
    /// has to be the response the handler produced, header for header and byte for byte.
    ///
    /// That comparison only means something over a route whose answer depends on nothing but the
    /// request, hence `/api/v1/profile`: a `&'static str` rendered once at startup. The probes
    /// cannot stand in — each stamps the current time to the millisecond, so two of their
    /// responses differ whenever the clock ticks between the calls, which says nothing about the
    /// layers.
    ///
    /// Also the only place the layer *types* are composed the way `router` composes them, so a
    /// version of `sentry-tower` that stops fitting an axum `Router` fails here rather than in the
    /// image build.
    #[tokio::test]
    async fn the_sentry_layers_leave_a_response_untouched() {
        let config = portfolio_config::SentryConfig {
            enabled: true,
            dsn: Some(secrecy::SecretString::from("https://key@sentry.example/42")),
            ..portfolio_config::SentryConfig::default()
        };
        let (hub, http) =
            telemetry::http_layers(&config).expect("a configured block mounts both layers");

        let request = || {
            Request::builder()
                .uri("/api/v1/profile")
                .body(Body::empty())
                .unwrap()
        };
        let bare = api_router(AssetsConfig::default(), legal::tests::fixture())
            .oneshot(request())
            .await
            .unwrap();
        let layered = api_router(AssetsConfig::default(), legal::tests::fixture())
            .layer(http)
            .layer(hub)
            .oneshot(request())
            .await
            .unwrap();

        assert_eq!(bare.status(), layered.status());
        assert_eq!(bare.headers(), layered.headers());
        assert_eq!(bytes(bare).await, bytes(layered).await);
    }

    /// The other half of the switch: with the block at its default nothing is mounted at all, so a
    /// deployment that never asked for reporting pays nothing per request.
    #[test]
    fn the_default_block_mounts_no_sentry_layer() {
        assert!(telemetry::http_layers(&portfolio_config::SentryConfig::default()).is_none());
    }

    #[tokio::test]
    async fn profile_rejects_non_get_methods() {
        let response = api_router(AssetsConfig::default(), legal::tests::fixture())
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
