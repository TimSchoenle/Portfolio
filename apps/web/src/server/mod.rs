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

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

use axum::{
    Router,
    http::{HeaderName, HeaderValue, header},
    middleware,
    routing::get,
};
use dioxus::server::{DioxusRouterExt, IncrementalRendererConfig, ServeConfig};
use tower_http::{
    compression::{
        CompressionLayer,
        predicate::{DefaultPredicate, NotForContentType, Predicate},
    },
    set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
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

    let (dioxus_app, isr_enabled) = dioxus_app_router();
    let app = dioxus_app.merge(api_router()).merge(assets::router());
    // Only when ISR is active: tag page navigations with the negotiated locale so
    // the per-path incremental cache keeps a separate entry per language.
    let app = if isr_enabled {
        app.layer(middleware::from_fn(tag_locale_for_isr))
    } else {
        app
    };

    app.layer(static_header(header::CONTENT_SECURITY_POLICY, CSP))
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
        .layer(CompressionLayer::new().compress_when(compression_predicate()))
        // Assign a `Cache-Control` TTL per asset class, but only when a handler
        // did not already set one (so the API's own headers win). Runs last.
        .layer(middleware::from_fn(cache::set_cache_control))
        .layer(TraceLayer::new_for_http())
}

/// The response-compression predicate. Starts from tower-http's default (which
/// already skips tiny bodies, images, gRPC and server-sent-event streams) and
/// additionally skips our woff2 fonts and resume PDFs: those are already
/// compressed, so running br/gzip over them only burns CPU and can grow the
/// payload rather than shrink it.
fn compression_predicate() -> impl Predicate {
    DefaultPredicate::new()
        .and(NotForContentType::const_new("font/woff2"))
        .and(NotForContentType::const_new("application/pdf"))
}

/// Environment variable naming the writable directory Dioxus's incremental
/// renderer (ISR) caches rendered HTML into. Set empty to disable ISR (the
/// server then renders every request fresh). The image defaults it to a
/// sub-directory of `/tmp`, which the deployment (Helm chart) already provides
/// as a writable mount even under a read-only root filesystem. Keep it *outside*
/// the bundled `public/` asset tree so those assets remain immutable.
const ISR_CACHE_DIR_ENV: &str = "ISR_CACHE_DIR";

/// Environment variable overriding the ISR revalidation interval, in seconds.
/// Unset, empty, `0`, or an unparseable value all mean "never revalidate" (a
/// permanent cache): every page renders from compile-time data, so the only
/// thing that changes the output is a new build/deploy — which starts from an
/// empty cache anyway. A positive value opts into a finite, time-based TTL,
/// useful only when a *persistent* cache volume is shared across deploys.
const ISR_TTL_SECS_ENV: &str = "ISR_TTL_SECS";

/// Query-string marker [`tag_locale_for_isr`] appends to page requests so the
/// otherwise language-blind, path-keyed incremental cache stores one entry per
/// negotiated locale. It is server-internal: the router matches on the path and
/// ignores it, and every link is built from the router / site config, so it
/// never leaks into the rendered HTML or the browser's address bar.
const ISR_LOCALE_PARAM: &str = "__isr_locale";

/// The Dioxus SSR/asset router, with incremental static regeneration enabled
/// when [`ISR_CACHE_DIR_ENV`] names a writable directory.
///
/// Dioxus's incremental renderer keys its cache by request path (and query)
/// only. This site negotiates locale per request (cookie / `Accept-Language`,
/// see `crate::i18n::detect_locale`), so a naive per-path cache would serve
/// whichever language rendered a URL first to every visitor. To keep ISR
/// correct, [`tag_locale_for_isr`] appends the negotiated locale to the request
/// URI and [`isr_map_path`] nests each render under its locale, giving one cache
/// entry per language per path.
/// Returns the Dioxus router and whether ISR is enabled (so the caller can add
/// the locale-tagging middleware only when the cache is active).
fn dioxus_app_router() -> (Router, bool) {
    match incremental_config() {
        Some(cfg) => (
            Router::new()
                .serve_dioxus_application(ServeConfig::builder().incremental(cfg), crate::app::App),
            true,
        ),
        // Unset: keep the framework default (fresh render, no on-disk cache).
        None => (dioxus::server::router(crate::app::App), false),
    }
}

/// Builds the incremental-render configuration from the environment, or `None`
/// when ISR is disabled ([`ISR_CACHE_DIR_ENV`] unset) or the cache directory
/// is not usable (cannot be created, or exists but is not writable).
fn incremental_config() -> Option<IncrementalRendererConfig> {
    let dir = std::env::var_os(ISR_CACHE_DIR_ENV)?;
    // Treat an empty value the same as unset: container platforms routinely
    // inject `KEY=` for a declared-but-unset variable, and that must mean "ISR
    // off", not "cache into the current directory".
    if dir.is_empty() {
        return None;
    }
    let dir = PathBuf::from(dir);
    // Verify the directory can actually be written to, not merely that it
    // exists: a mounted volume (e.g. a root-owned Kubernetes `emptyDir`) can be
    // present yet unwritable to our non-root user, in which case `create_dir_all`
    // succeeds but the renderer would fail to persist every page at runtime.
    if let Err(err) = ensure_writable_dir(&dir) {
        tracing::warn!(
            "ISR disabled: cache directory {} is not usable: {err}",
            dir.display()
        );
        return None;
    }

    let invalidate_after = parse_isr_ttl(std::env::var(ISR_TTL_SECS_ENV).ok().as_deref());
    match invalidate_after {
        Some(ttl) => tracing::info!(
            "ISR enabled: caching rendered pages per locale in {} (revalidate after {}s)",
            dir.display(),
            ttl.as_secs()
        ),
        None => tracing::info!(
            "ISR enabled: caching rendered pages per locale in {} (permanent; refreshed only by a new deploy)",
            dir.display()
        ),
    }
    let map_dir = dir.clone();
    // Remembers which cache entries we have already announced as created, so the
    // log line fires once per creation rather than on every `map_path` call (the
    // renderer invokes it on both the cache-miss lookup and the subsequent write).
    let announced: Mutex<HashSet<PathBuf>> = Mutex::new(HashSet::new());
    let mut config = IncrementalRendererConfig::new()
        .static_dir(dir)
        // Fold the locale marker into the on-disk path: the default mapping
        // drops the query string, collapsing every language onto one file.
        .map_path(move |route| {
            let mapped = isr_map_path(&map_dir, route);
            if is_new_cache_entry(&announced, &mapped) {
                log_cache_entry_created(route, &mapped);
            }
            mapped
        })
        // Keep any pages regenerated by a previous pod across restarts.
        .clear_cache(false);
    // Only impose a time-based TTL when one is explicitly configured; the default
    // is a permanent cache, invalidated instead by the next build starting empty.
    if let Some(ttl) = invalidate_after {
        config = config.invalidate_after(ttl);
    }
    Some(config)
}

/// Parses the configured ISR revalidation interval ([`ISR_TTL_SECS_ENV`]) into an
/// optional invalidation duration. `None` means a permanent cache (no time-based
/// invalidation): unset, empty, a literal `0`, or an unparseable value all fail
/// safe to permanent, since every page renders from compile-time data and the
/// real invalidation boundary is a redeploy (which starts from an empty cache).
/// A positive integer opts back into a finite TTL for setups that share a
/// persistent cache volume across deploys and want time-based churn.
fn parse_isr_ttl(raw: Option<&str>) -> Option<Duration> {
    match raw.map(str::trim) {
        None | Some("") => None,
        Some(secs) => match secs.parse::<u64>() {
            Ok(0) | Err(_) => None,
            Ok(secs) => Some(Duration::from_secs(secs)),
        },
    }
}

/// Ensures `dir` exists and is actually writable by our process. Creates the
/// directory (and any missing parents), then probes it with a throwaway file,
/// so callers can distinguish a usable cache directory from one that merely
/// exists but rejects writes (wrong owner/permissions, read-only mount). The
/// probe file is best-effort removed afterwards.
fn ensure_writable_dir(dir: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    let probe = dir.join(".isr-write-test");
    std::fs::write(&probe, b"")?;
    let _ = std::fs::remove_file(&probe);
    Ok(())
}

/// Middleware that folds the request's negotiated locale into the URI query for
/// page navigations (`GET` with `Accept: text/html`), so the incremental
/// renderer — which keys its cache by path-and-query only — stores a distinct
/// entry per language instead of serving whichever locale rendered a path first.
/// The decision mirrors `crate::i18n::detect_locale` exactly (both go through
/// `negotiate_locale`), so the cache key and the HTML rendered into it always
/// agree on the language. Sub-resource and server-function requests are left
/// untouched (they never hit the incremental cache).
async fn tag_locale_for_isr(
    mut request: axum::extract::Request,
    next: middleware::Next,
) -> axum::response::Response {
    if is_html_navigation(&request) {
        let read_header = |name| {
            request
                .headers()
                .get(name)
                .and_then(|v| v.to_str().ok())
                .map(str::to_owned)
        };
        let cookie = read_header(header::COOKIE);
        let accept_language = read_header(header::ACCEPT_LANGUAGE);
        let locale = crate::i18n::negotiate_locale(cookie.as_deref(), accept_language.as_deref());
        *request.uri_mut() = with_locale_query(request.uri(), &locale);
    }
    next.run(request).await
}

/// True for top-level page loads: a `GET` whose `Accept` header includes
/// `text/html`. Sub-resource requests (JS/wasm/CSS/images) and server-function
/// calls advertise other `Accept` values, so they are left un-tagged.
fn is_html_navigation(request: &axum::extract::Request) -> bool {
    request.method() == axum::http::Method::GET
        && request
            .headers()
            .get(header::ACCEPT)
            .and_then(|v| v.to_str().ok())
            .is_some_and(|accept| accept.contains("text/html"))
}

/// Returns `uri` with `ISR_LOCALE_PARAM=<locale>` appended to its query string,
/// preserving any query already present. On the unlikely chance the rewrite does
/// not parse, the original URI is returned unchanged so the page still renders.
fn with_locale_query(uri: &axum::http::Uri, locale: &str) -> axum::http::Uri {
    let path = uri.path();
    let combined = match uri.query() {
        Some(query) if !query.is_empty() => {
            format!("{path}?{query}&{ISR_LOCALE_PARAM}={locale}")
        }
        _ => format!("{path}?{ISR_LOCALE_PARAM}={locale}"),
    };
    let Ok(path_and_query) = combined.parse::<axum::http::uri::PathAndQuery>() else {
        return uri.clone();
    };
    let mut parts = uri.clone().into_parts();
    parts.path_and_query = Some(path_and_query);
    axum::http::Uri::from_parts(parts).unwrap_or_else(|_| uri.clone())
}

/// Maps an incremental-cache route to its on-disk folder, mirroring Dioxus's
/// default layout but nesting each render under its locale sub-directory. The
/// framework's default mapping discards the query string, which would collapse
/// every language onto a single file; keying off the [`ISR_LOCALE_PARAM`] marker
/// keeps the languages separate (and stable across restarts). A route without
/// the marker maps straight under `static_dir`, matching the default.
fn isr_map_path(static_dir: &std::path::Path, route: &str) -> PathBuf {
    let (path, query) = route.split_once('?').unwrap_or((route, ""));
    let mut mapped = static_dir.to_path_buf();
    if let Some(locale) = locale_from_query(query) {
        mapped.push(locale);
    }
    for segment in path.split('/') {
        mapped.push(segment);
    }
    mapped
}

/// Extracts the [`ISR_LOCALE_PARAM`] value from a raw query string, if present.
fn locale_from_query(query: &str) -> Option<&str> {
    let prefix = format!("{ISR_LOCALE_PARAM}=");
    query.split('&').find_map(|pair| pair.strip_prefix(&prefix))
}

/// Decides whether `mapped` (the on-disk folder Dioxus maps a route to) is about
/// to receive a *freshly created* cache entry, and records that decision so the
/// caller logs it exactly once.
///
/// The renderer calls `map_path` twice around a creation — first for the
/// cache-miss lookup, then for the write — and again for every later cache hit,
/// so a naive log would be noisy or duplicated. We therefore key off whether a
/// rendered `.html` already exists on disk:
/// * No cached render yet, and not already announced → this is a new entry
///   (returns `true`, remembers it so the paired write call stays quiet).
/// * A cached render exists → the entry is live; forget any announcement so a
///   post-TTL regeneration is reported again (returns `false`).
fn is_new_cache_entry(announced: &Mutex<HashSet<PathBuf>>, mapped: &Path) -> bool {
    let mut announced = announced.lock().unwrap_or_else(|e| e.into_inner());
    if has_cached_render(mapped) {
        announced.remove(mapped);
        false
    } else {
        // `insert` returns `false` when the entry was already announced (the
        // second call of the miss→write pair), keeping the log to one line.
        announced.insert(mapped.to_path_buf())
    }
}

/// Whether `mapped` already holds a rendered page, i.e. its `index` sub-directory
/// contains at least one `.html` file (the layout [`isr_map_path`] and Dioxus's
/// `FileSystemCache` write into). Missing directory or no HTML means not cached.
fn has_cached_render(mapped: &Path) -> bool {
    std::fs::read_dir(mapped.join("index"))
        .into_iter()
        .flatten()
        .flatten()
        .any(|entry| entry.path().extension().is_some_and(|ext| ext == "html"))
}

/// Logs, at INFO, that the incremental renderer just created a new on-disk cache
/// entry for a page. The locale is pulled from the server-internal
/// [`ISR_LOCALE_PARAM`] marker so operators can see which language was persisted.
fn log_cache_entry_created(route: &str, mapped: &Path) {
    let (path, query) = route.split_once('?').unwrap_or((route, ""));
    let locale = locale_from_query(query).unwrap_or("default");
    tracing::info!(
        "ISR cache entry created for path \"{path}\" (locale \"{locale}\") at {}",
        mapped.display()
    );
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

    /// A unique scratch path under the OS temp dir; the file/dir itself is not
    /// created, only its name is reserved for the test to use.
    fn scratch_path(tag: &str) -> PathBuf {
        use std::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("isr-test-{tag}-{}-{n}", std::process::id()))
    }

    #[test]
    fn writable_dir_probe_accepts_a_creatable_directory() {
        // A not-yet-existing nested path is created (parents and all) and
        // reported writable, and the probe file must not linger behind.
        let dir = scratch_path("ok").join("nested");
        assert!(ensure_writable_dir(&dir).is_ok());
        assert!(dir.is_dir());
        assert!(!dir.join(".isr-write-test").exists());
        let _ = std::fs::remove_dir_all(dir.parent().unwrap());
    }

    #[test]
    fn writable_dir_probe_rejects_a_path_blocked_by_a_file() {
        // A regular file standing in for a directory component makes the
        // directory uncreatable, so the probe must report the failure rather
        // than pretend ISR is usable.
        let file = scratch_path("blocked");
        std::fs::write(&file, b"not a directory").unwrap();
        let dir = file.join("cache");
        assert!(ensure_writable_dir(&dir).is_err());
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn locale_query_is_appended_to_a_bare_path() {
        let tagged = with_locale_query(&"/".parse().unwrap(), "de");
        assert_eq!(
            tagged.path_and_query().unwrap().as_str(),
            "/?__isr_locale=de"
        );
    }

    #[test]
    fn locale_query_preserves_an_existing_query() {
        let tagged = with_locale_query(&"/imprint?ref=x".parse().unwrap(), "en");
        assert_eq!(
            tagged.path_and_query().unwrap().as_str(),
            "/imprint?ref=x&__isr_locale=en"
        );
    }

    #[test]
    fn map_path_separates_locales_and_falls_back_without_a_marker() {
        let base = std::path::Path::new("/cache");
        let has = |p: &std::path::Path, name: &str| {
            p.components().any(|c| c.as_os_str().to_str() == Some(name))
        };

        let de = isr_map_path(base, "/?__isr_locale=de");
        let en = isr_map_path(base, "/?__isr_locale=en");
        let untagged = isr_map_path(base, "/");

        // Each language lands in its own sub-directory, distinct from the other
        // and from the marker-less (default) mapping.
        assert_ne!(de, en);
        assert_ne!(de, untagged);
        assert!(has(&de, "de"));
        assert!(has(&en, "en"));
        assert!(!has(&untagged, "de") && !has(&untagged, "en"));

        // A different path under the same language shares the language sub-dir
        // but still maps to its own folder.
        let imprint_de = isr_map_path(base, "/imprint?__isr_locale=de");
        assert!(has(&imprint_de, "de") && has(&imprint_de, "imprint"));
        assert_ne!(imprint_de, de);
    }

    #[test]
    fn ttl_defaults_to_permanent_and_honours_a_finite_override() {
        // Unset, blank, whitespace, and an explicit zero all mean "never
        // revalidate": the cache is refreshed by a redeploy, not by elapsed time.
        assert_eq!(parse_isr_ttl(None), None);
        assert_eq!(parse_isr_ttl(Some("")), None);
        assert_eq!(parse_isr_ttl(Some("  ")), None);
        assert_eq!(parse_isr_ttl(Some("0")), None);
        // Garbage fails safe to permanent rather than a surprise interval.
        assert_eq!(parse_isr_ttl(Some("later")), None);
        // A positive value opts back into a finite, time-based TTL.
        assert_eq!(parse_isr_ttl(Some("3600")), Some(Duration::from_secs(3600)));
        assert_eq!(parse_isr_ttl(Some(" 90 ")), Some(Duration::from_secs(90)));
    }

    #[test]
    fn a_new_cache_entry_is_announced_once_then_re_armed_after_invalidation() {
        let mapped = scratch_path("entry");
        let index = mapped.join("index");
        let announced: Mutex<HashSet<PathBuf>> = Mutex::new(HashSet::new());

        // First sighting of an uncached path (the cache-miss lookup) reports a
        // new entry; the paired write call must stay silent so we log once.
        assert!(is_new_cache_entry(&announced, &mapped));
        assert!(!is_new_cache_entry(&announced, &mapped));

        // Once the render is on disk, the entry is live and no longer "new".
        std::fs::create_dir_all(&index).unwrap();
        std::fs::write(index.join("deadbeef.html"), b"<html></html>").unwrap();
        assert!(!is_new_cache_entry(&announced, &mapped));

        // After a TTL invalidation removes the HTML, a regeneration is a fresh
        // creation again and must be reported anew.
        std::fs::remove_file(index.join("deadbeef.html")).unwrap();
        assert!(is_new_cache_entry(&announced, &mapped));

        let _ = std::fs::remove_dir_all(&mapped);
    }

    #[test]
    fn only_html_get_navigations_are_tagged() {
        let build = |method: &str, accept: &str| {
            Request::builder()
                .method(method)
                .uri("/")
                .header(header::ACCEPT, accept)
                .body(Body::empty())
                .unwrap()
        };

        assert!(is_html_navigation(&build(
            "GET",
            "text/html,application/xhtml+xml"
        )));
        // Sub-resource fetches and non-GET methods must not be tagged.
        assert!(!is_html_navigation(&build("GET", "*/*")));
        assert!(!is_html_navigation(&build("POST", "text/html")));

        // A request without an Accept header is treated as a non-navigation.
        let no_accept = Request::builder()
            .method("GET")
            .uri("/")
            .body(Body::empty())
            .unwrap();
        assert!(!is_html_navigation(&no_accept));
    }
}
