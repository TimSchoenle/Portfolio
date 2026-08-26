//! Native SSR server.
//!
//! Extends the Dioxus fullstack router (`dioxus::server::router`, which serves
//! the hydrating SSR page, static assets and any `#[server]` endpoints) with the
//! public JSON API, the SEO documents, and the security-header / compression /
//! cache-control layers that used to live in the standalone `apps/server` crate.

mod api;
mod assets;
mod cache;
mod csp;
mod seo;
mod telemetry;

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};

use axum::{
    Router,
    body::Body,
    http::{HeaderName, HeaderValue, StatusCode, header},
    middleware,
    response::IntoResponse,
    routing::get,
};
use dioxus::server::{DioxusRouterExt, IncrementalRendererConfig, ServeConfig};
use portfolio_config::{AssetsConfig, IsrConfig, ServerConfig};
use portfolio_data::LANGUAGES;
use tower_http::{
    compression::{
        CompressionLayer,
        predicate::{NotForContentType, Predicate, SizeAbove},
    },
    set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
};

/// Everything this server reads from its configuration.
///
/// Exit code for a configuration the process cannot start with (`EX_CONFIG`
/// from `sysexits.h`), so an operator can tell a bad config apart from a crash
/// without reading the logs.
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
        async move { Ok(router(&config)) }
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
/// [`SentryConfig::validate`](portfolio_config::SentryConfig::validate). Failing here means the
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
    config
}

/// The full application router: the Dioxus SSR/asset router with our routes and
/// layers mounted on top. Custom routes take precedence over the SSR fallback;
/// layers apply to every response (SSR HTML, static assets, API, SEO).
fn router(config: &ServerConfig) -> Router {
    let static_header = |name: HeaderName, value: &'static str| {
        SetResponseHeaderLayer::overriding(name, HeaderValue::from_static(value))
    };

    let policy = Arc::new(csp::SitePolicy::new(&config.csp));
    let (dioxus_app, isr_enabled) = dioxus_app_router(&config.isr);
    let app = dioxus_app
        .merge(api_router(config.assets.clone()))
        .merge(assets::router());
    // Page handling for navigations, innermost so it sees the request before the
    // router and the response before compression: it stamps the negotiated
    // language onto `<html lang>`, declares the `Vary` axes the response actually
    // depends on, gives the document the Content-Security-Policy derived from the
    // very bytes it is about to send, and — when ISR is active — tags the URI so
    // the per-path incremental cache keeps a separate entry per language.
    let app = app.layer(middleware::from_fn_with_state(
        PageState {
            isr_enabled,
            policy: Arc::clone(&policy),
            pages: Arc::new(PageMemo::default()),
        },
        localize_page,
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

/// What [`localize_page`] carries per request.
///
/// Cloned for every request the middleware sees, so the policy — which owns a rendered header
/// value and, when hashing is on, the builder each document's hashes are folded into — is behind
/// an [`Arc`] rather than copied.
#[derive(Clone)]
struct PageState {
    /// Whether the incremental cache is active, and therefore whether the request URI has to
    /// carry the negotiated locale for the cache to key on.
    isr_enabled: bool,
    /// The Content-Security-Policy this server serves. See [`csp`].
    policy: Arc<csp::SitePolicy>,
    /// Finished pages, memoized per `(path, locale)`. See [`PageMemo`].
    pages: Arc<PageMemo>,
}

/// A finished page: the body exactly as it will be sent, and the inline-script hashes taken from
/// those very bytes.
///
/// The two are stored and served as a unit and never mixed, which is what makes memoizing them
/// safe: a policy is only ever applied to the document it was scanned from, so a remembered entry
/// cannot go stale against the bytes it describes.
struct RenderedPage {
    /// The body with `lang` already stamped onto its `<html>` tag.
    body: axum::body::Bytes,
    /// What [`csp::SitePolicy::scan`] found in [`Self::body`].
    scan: csp::DocumentScan,
}

/// Finished pages, keyed by `(path, locale)`.
///
/// [`localize_page`] is layered *outside* the Dioxus router, so it runs on incremental-cache hits
/// as well as misses. Without this memo every page request re-validated tens of kilobytes of
/// UTF-8, copied all of it into a fresh `String` to stamp `<html lang>`, and ran SHA-256 over
/// every inline script — the serialized hydration payload among them — only to rebuild a policy
/// byte-identical to the one before it.
///
/// Sound for exactly the reason the incremental cache itself is: a [`CACHEABLE_PATHS`] page is
/// rendered entirely from compile-time data, so its bytes are a function of the path and the
/// negotiated locale and of nothing else. Both halves of the key are `&'static str` drawn from
/// those same compile-time lists, which is also what bounds the map at
/// `CACHEABLE_PATHS.len() * LANGUAGES.len()` entries — it is a memo, not a cache, and so needs
/// neither eviction nor a TTL.
///
/// What it deliberately does *not* do is answer the request itself: the render still goes through
/// the router, so the incremental cache stays the one thing deciding whether a page is rendered
/// or read back. This only removes the post-processing that used to run on both paths.
type PageMemo = RwLock<HashMap<(&'static str, &'static str), Arc<RenderedPage>>>;

/// The memoized page for `key`, if one has been rendered in this process.
///
/// A poisoned lock is recovered from rather than propagated, matching
/// [`is_new_cache_entry`]: nothing can panic while it is held, and a request must not fail over a
/// bookkeeping structure it could just as well have missed in.
fn memo_get(memo: &PageMemo, key: (&'static str, &'static str)) -> Option<Arc<RenderedPage>> {
    memo.read()
        .unwrap_or_else(|err| err.into_inner())
        .get(&key)
        .cloned()
}

/// Records a finished page under `key` and hands back the shared handle to it.
fn memo_insert(
    memo: &PageMemo,
    key: (&'static str, &'static str),
    page: RenderedPage,
) -> Arc<RenderedPage> {
    let page = Arc::new(page);
    memo.write()
        .unwrap_or_else(|err| err.into_inner())
        .insert(key, Arc::clone(&page));
    page
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

/// Query-string marker [`localize_page`] appends to page requests so the
/// otherwise language-blind, path-keyed incremental cache stores one entry per
/// negotiated locale. It is server-internal: the router matches on the path and
/// ignores it, and every link is built from the router / site config, so it
/// never leaks into the rendered HTML or the browser's address bar.
///
/// Because the marker travels in a *client-supplied* URI, it is never trusted on
/// the way in: [`with_locale_query`] strips any copy the request already carried
/// before appending ours, and [`locale_from_query`] only ever returns a value
/// that is a known [`LANGUAGES`] entry. Without both, a request could choose
/// which cache entry its render lands in (serving one visitor's language to
/// another) and, since the value becomes a path component, where on disk it is
/// written.
const ISR_LOCALE_PARAM: &str = "__isr_locale";

/// [`ISR_LOCALE_PARAM`] with the `=` that separates a query pair's key from its
/// value. A constant rather than a `format!` at the use site: [`locale_from_query`]
/// is called by the incremental renderer's path mapper on every cacheable page
/// request, and building the needle there allocated a `String` each time.
const ISR_LOCALE_QUERY_PREFIX: &str = "__isr_locale=";

/// Compile-time proof that the two constants above have not drifted apart, for
/// the same reason as `crate::i18n::LANG_COOKIE_PREFIX`: `concat!` takes only
/// literals, so nothing else would catch a rename of one leaving the other
/// behind — and a mapper that stops recognising the marker silently collapses
/// every language back onto one cache entry.
const _: () = {
    let key = ISR_LOCALE_PARAM.as_bytes();
    let prefix = ISR_LOCALE_QUERY_PREFIX.as_bytes();
    assert!(prefix.len() == key.len() + 1);
    assert!(prefix[key.len()] == b'=');
    let mut i = 0;
    while i < key.len() {
        assert!(prefix[i] == key[i]);
        i += 1;
    }
};

/// Page paths whose rendered HTML is worth persisting. These mirror the concrete
/// variants of `crate::routes::Route`; the catch-all `NotFound` route is
/// deliberately absent.
///
/// The incremental cache is keyed by path, and the catch-all matches every URL
/// that exists — so without this allowlist an unauthenticated client could mint
/// an unbounded number of cache entries (one directory and one HTML file each)
/// simply by requesting `/1`, `/2`, … until the cache volume filled up. Routes
/// outside the list still render normally; their output is just never stored
/// (see [`isr_map_path`]).
const CACHEABLE_PATHS: [&str; 4] = ["/", "/imprint", "/privacy", "/licenses"];

/// Whether a request path is one of the [`CACHEABLE_PATHS`].
fn is_cacheable_path(path: &str) -> bool {
    cacheable_path(path).is_some()
}

/// The [`CACHEABLE_PATHS`] entry equal to `path`.
///
/// Returning the compile-time entry rather than a `bool` is what lets [`PageMemo`] key on a
/// `&'static str`: the request's own path is borrowed from a URI that does not outlive the
/// request, whereas this one is a program constant, so the memo owns nothing and can never be
/// keyed by attacker-supplied text.
fn cacheable_path(path: &str) -> Option<&'static str> {
    CACHEABLE_PATHS.into_iter().find(|known| *known == path)
}

/// Name of the sentinel that [`isr_map_path`] maps every non-cacheable route to.
///
/// [`ensure_uncacheable_sentinel`] creates it as a regular **file** inside the
/// cache directory, which is what makes the opt-out work: the renderer persists
/// a page by `create_dir_all`-ing the mapped folder and writing `index.html`
/// into it, and neither can succeed underneath a plain file. Lookups miss for
/// the same reason, so non-cacheable routes render fresh every time and leave
/// nothing behind. Collapsing them onto one path rather than giving each its own
/// is deliberate: a shared *cache entry* would serve one unknown URL's render
/// (including its serialized hydration route) for a different unknown URL.
const ISR_UNCACHEABLE_SENTINEL: &str = ".uncacheable";

/// The Dioxus SSR and asset router, and whether it came back with the incremental cache on.
///
/// The cache is on only when the configured directory turns out to be writable, so the caller
/// layers the locale-tagging half of [`localize_page`] onto the returned router only when the
/// flag is `true`.
///
/// Dioxus's incremental renderer keys on the request path and query alone, and this site
/// negotiates the locale per request, so a per-path cache would serve whichever language
/// rendered a URL first to everyone who asked for it afterwards.
/// [`with_locale_query`] puts the negotiated locale on the URI and [`isr_map_path`] nests each
/// render under it, which is what makes the key one entry per language per path.
fn dioxus_app_router(isr: &IsrConfig) -> (Router, bool) {
    match incremental_config(isr) {
        Some(cfg) => (
            Router::new()
                .serve_dioxus_application(ServeConfig::builder().incremental(cfg), crate::app::App),
            true,
        ),
        // Unset: keep the framework default (fresh render, no on-disk cache).
        None => (dioxus::server::router(crate::app::App), false),
    }
}

/// Builds the incremental-render configuration, or `None` when ISR is disabled
/// (no cache directory configured) or the cache directory is not usable (cannot
/// be created, or exists but is not writable).
fn incremental_config(isr: &IsrConfig) -> Option<IncrementalRendererConfig> {
    let dir = isr.cache_dir()?.to_path_buf();
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
    // The opt-out for non-cacheable routes is a file the renderer cannot write
    // underneath; without it every unknown URL would persist a cache entry.
    if let Err(err) = ensure_uncacheable_sentinel(&dir) {
        tracing::warn!(
            "ISR disabled: cannot create the uncacheable-route sentinel in {}: {err}",
            dir.display()
        );
        return None;
    }

    let invalidate_after = isr.invalidate_after();
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
            // Non-cacheable routes all share the sentinel path and never produce
            // an entry, so announcing them would be noise about a write that is
            // guaranteed not to happen.
            if is_cacheable_route(route) && is_new_cache_entry(&announced, &mapped) {
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

/// Creates the [`ISR_UNCACHEABLE_SENTINEL`] as a regular file inside `dir`, so
/// the renderer cannot create directories or write pages beneath it.
///
/// A directory left at that path by an older build (only possible when a
/// persistent cache volume is shared across deploys) is removed first: left in
/// place it would turn the opt-out into a single shared cache entry, which is
/// exactly what the sentinel exists to prevent.
fn ensure_uncacheable_sentinel(dir: &std::path::Path) -> std::io::Result<()> {
    let sentinel = dir.join(ISR_UNCACHEABLE_SENTINEL);
    if sentinel.is_dir() {
        std::fs::remove_dir_all(&sentinel)?;
    }
    std::fs::write(
        &sentinel,
        b"Routes mapped here are deliberately never cached.\n",
    )
}

/// Middleware applying the request's negotiated locale to a page navigation
/// (`GET` with `Accept: text/html`), in three ways:
///
/// 1. When `isr_enabled`, the locale is folded into the URI query so the
///    incremental renderer — which keys its cache by path-and-query only —
///    stores a distinct entry per language instead of serving whichever locale
///    rendered a path first.
/// 2. The response declares `Vary: Accept-Language, Cookie`, the two request
///    headers its content actually depends on. Without it any shared cache in
///    front of the origin (a CDN, a corporate proxy) keys on the URL alone and
///    happily hands a German render to an English visitor — the same bug the
///    per-locale cache key fixes on our side of the wire.
/// 3. The opening `<html>` tag is stamped with `lang`, so the language is
///    declared in the very first bytes a crawler or a JavaScript-less reader
///    receives rather than only after the client has hydrated.
/// 4. The document is given its own Content-Security-Policy, derived from the
///    inline scripts in the very bytes about to be sent (see [`csp`]). It rides
///    along here because the body has to be buffered for point 3 regardless.
///
/// The decision mirrors `crate::i18n::detect_locale` exactly (both go through
/// `negotiate_locale`), so the cache key, the `<html lang>` and the HTML
/// rendered into them always agree. Sub-resource and server-function requests
/// are left untouched (they never hit the incremental cache and carry no
/// language).
async fn localize_page(
    axum::extract::State(state): axum::extract::State<PageState>,
    mut request: axum::extract::Request,
    next: middleware::Next,
) -> axum::response::Response {
    // Only `GET` can produce a cacheable page; server-function calls are POSTs.
    if request.method() != axum::http::Method::GET {
        return next.run(request).await;
    }

    // Read straight out of the request rather than copied out of it:
    // `negotiate_locale` answers with a `&'static str` from `LANGUAGES`, so
    // nothing has to outlive the borrow. This middleware sees every request the
    // server answers, and the two `String`s this used to build were allocated on
    // all of them — including the asset requests that never reach the branch
    // below.
    let locale = {
        let headers = request.headers();
        let read_header = |name| headers.get(name).and_then(|v| v.to_str().ok());
        crate::i18n::negotiate_locale(
            read_header(header::COOKIE),
            read_header(header::ACCEPT_LANGUAGE),
        )
    };

    // Tag only what the cache will actually store. Notably this does *not* also
    // require an `Accept: text/html` header: a client that omits one still gets
    // a rendered page, so gating on it left every such request sharing a single
    // untagged, language-blind cache entry — whichever locale happened to fill
    // it first. Requests that are not pages map to the sentinel anyway, so
    // tagging them would change nothing.
    // Resolved before the request is consumed, and to the compile-time entry
    // rather than to the request's own slice, so [`PageMemo`] keys on a program
    // constant. `None` for anything else: the catch-all route is shared by every
    // unknown URL, so remembering one of their renders under it would answer the
    // next unknown URL with the previous one's document.
    let memo_key = cacheable_path(request.uri().path());

    if state.isr_enabled && memo_key.is_some() {
        *request.uri_mut() = with_locale_query(request.uri(), locale);
    }

    // Whether the response is a page at all is decided by its content type, not
    // by what the request asked for, so the 404 page is localized too.
    rewrite_html_response(next.run(request).await, locale, &state, memo_key).await
}

/// Largest HTML page body this server will buffer in order to stamp `<html
/// lang>` on it. Rendered pages are a few tens of kilobytes, so the limit exists
/// only to bound memory if something upstream ever produces an unexpectedly
/// large `text/html` response.
const MAX_HTML_REWRITE_BYTES: usize = 4 * 1024 * 1024;

/// Finishes a document response: marks it language-dependent (`Vary`), stamps
/// `lang="<locale>"` onto its opening `<html>` tag, and gives it the
/// Content-Security-Policy derived from the bytes it will actually carry.
///
/// Only `text/html` responses are touched: a request that merely *accepts* HTML
/// may still be answered with JSON or an asset (a browser navigating straight to
/// `/api/v1/profile`, say), and those neither vary by language nor have a tag to
/// annotate. Everything else passes straight through and picks up the
/// subresource policy from the layer outside this one.
///
/// The tag sits in the first bytes of the document, but the SSR body arrives as
/// a stream whose chunk boundaries are not ours to rely on, so the page is
/// buffered to rewrite it. That is affordable because rendering is not streamed
/// out of order (`StreamingMode::Disabled`, the default) and pages are tens of
/// kilobytes; enabling out-of-order streaming would mean revisiting this. A body
/// that is unreadable or larger than [`MAX_HTML_REWRITE_BYTES`] fails the
/// request rather than silently serving a truncated page.
///
/// The buffer is what makes the per-document policy affordable too: the inline
/// scripts are hashed out of the same string, after the `lang` rewrite, so the
/// hashes describe the response as sent rather than an earlier version of it.
///
/// All of that happens once per `(path, locale)`. `memo_key` names the
/// [`CACHEABLE_PATHS`] entry this request is for, if any; a second request for
/// the same page is answered from [`PageMemo`] without buffering, validating,
/// copying or re-hashing anything — only the nonce in its policy is new.
async fn rewrite_html_response(
    mut response: axum::response::Response,
    locale: &'static str,
    state: &PageState,
    memo_key: Option<&'static str>,
) -> axum::response::Response {
    let is_html = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|value| value.starts_with("text/html"));
    if !is_html {
        return response;
    }

    // Declared before the body work so it survives every early return below.
    response.headers_mut().insert(
        header::VARY,
        HeaderValue::from_static("accept-language, cookie"),
    );

    let (parts, body) = response.into_parts();
    // Only a successful render is worth remembering, and worth being served from
    // memory. An error page shares its path with the document it failed to
    // produce, so keying one as the other would answer a later 500 with the last
    // good page — or, worse, hand the error page's body a 200's policy.
    let key = memo_key
        .filter(|_| parts.status == StatusCode::OK)
        .map(|path| (path, locale));

    if let Some(page) = key.and_then(|key| memo_get(&state.pages, key)) {
        // The stored bytes are already localized and the stored scan describes
        // exactly them, so there is nothing here to buffer, validate, copy or
        // hash. The upstream body is dropped undrained, which is sound because
        // the renderer does not stream out of order — the response was complete
        // before this middleware saw it.
        drop(body);
        return finish_page(parts, &state.policy, &page);
    }

    let Ok(bytes) = axum::body::to_bytes(body, MAX_HTML_REWRITE_BYTES).await else {
        tracing::error!("page body was unreadable or exceeded {MAX_HTML_REWRITE_BYTES} bytes");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    let Ok(html) = std::str::from_utf8(&bytes) else {
        // Already broken: a document that is not UTF-8 cannot be scanned for the
        // scripts a policy would have to admit, so it also keeps the subresource
        // policy the outer layer supplies rather than a hashed one.
        tracing::error!("page body was not valid UTF-8; serving it unmodified");
        return axum::response::Response::from_parts(parts, Body::from(bytes));
    };

    let localized = with_html_lang(html, locale);
    let page = RenderedPage {
        scan: state.policy.scan(&localized),
        body: axum::body::Bytes::from(localized),
    };
    let page = match key {
        Some(key) => memo_insert(&state.pages, key, page),
        None => Arc::new(page),
    };
    finish_page(parts, &state.policy, &page)
}

/// Assembles the response for a finished page: the policy for the scripts in its
/// body, then the body itself.
///
/// Split out because it is reached from both the memo hit and the first render,
/// and the two must not drift — in particular over the `Content-Length` below,
/// which is wrong on either path.
fn finish_page(
    mut parts: axum::http::response::Parts,
    policy: &csp::SitePolicy,
    page: &RenderedPage,
) -> axum::response::Response {
    policy.apply_to_document(&mut parts.headers, &page.scan);
    // The rewrite changes the body length, so any `Content-Length` computed
    // before it is now a lie. Drop it and let the new body's known size speak.
    parts.headers.remove(header::CONTENT_LENGTH);
    axum::response::Response::from_parts(parts, Body::from(page.body.clone()))
}

/// Rewrites the opening `<html …>` tag of `html` to carry `lang="<locale>"`,
/// replacing any `lang` already present. Returns the document unchanged when it
/// has no `<html>` tag (nothing to annotate) — the page is still served.
fn with_html_lang(html: &str, locale: &str) -> String {
    let Some(open) = html.find("<html") else {
        return html.to_string();
    };
    // The tag is plain server-generated markup, so the first `>` after `<html`
    // ends it; there is no attribute value in between that could contain one.
    let Some(close) = html[open..].find('>').map(|i| open + i) else {
        return html.to_string();
    };

    let attrs = &html[open + "<html".len()..close];
    let mut kept: Vec<&str> = Vec::new();
    for attr in attrs.split_whitespace() {
        let name = attr.split('=').next().unwrap_or(attr);
        if !name.eq_ignore_ascii_case("lang") {
            kept.push(attr);
        }
    }

    let mut out = String::with_capacity(html.len() + 16);
    out.push_str(&html[..open]);
    out.push_str("<html");
    for attr in kept {
        out.push(' ');
        out.push_str(attr);
    }
    // `locale` is a `LANGUAGES` entry, never request text, so it needs no
    // escaping to sit safely inside the attribute. Pushed in pieces rather than
    // through a `format!`, which would allocate a throwaway `String` per page.
    out.push_str(" lang=\"");
    out.push_str(locale);
    out.push('"');
    out.push_str(&html[close..]);
    out
}

/// Returns `uri` with `ISR_LOCALE_PARAM=<locale>` appended to its query string,
/// preserving any query already present *except* a copy of the marker itself.
///
/// Stripping the incoming marker is what keeps the cache key ours. The value
/// becomes both the cache bucket and an on-disk path component, so a request
/// allowed to supply its own would be able to file a render under a language it
/// is not written in (serving German to the next English visitor) and to escape
/// the cache directory entirely with `../` or a leading `/`. On the unlikely
/// chance the rewrite does not parse, the original URI is returned unchanged so
/// the page still renders.
fn with_locale_query(uri: &axum::http::Uri, locale: &str) -> axum::http::Uri {
    let path = uri.path();
    let retained: Vec<&str> = uri
        .query()
        .unwrap_or("")
        .split('&')
        .filter(|pair| !pair.is_empty() && !is_locale_marker(pair))
        .collect();

    let mut combined = String::from(path);
    combined.push('?');
    for pair in retained {
        combined.push_str(pair);
        combined.push('&');
    }
    combined.push_str(ISR_LOCALE_PARAM);
    combined.push('=');
    combined.push_str(locale);

    let Ok(path_and_query) = combined.parse::<axum::http::uri::PathAndQuery>() else {
        return uri.clone();
    };
    let mut parts = uri.clone().into_parts();
    parts.path_and_query = Some(path_and_query);
    axum::http::Uri::from_parts(parts).unwrap_or_else(|_| uri.clone())
}

/// Whether a raw `key=value` query pair carries the server-internal locale
/// marker, whatever its value (including no value at all).
fn is_locale_marker(pair: &str) -> bool {
    pair.split('=').next() == Some(ISR_LOCALE_PARAM)
}

/// Maps an incremental-cache route to its on-disk folder, mirroring Dioxus's
/// default layout but nesting each render under its locale sub-directory. The
/// framework's default mapping discards the query string, which would collapse
/// every language onto a single file; keying off the [`ISR_LOCALE_PARAM`] marker
/// keeps the languages separate (and stable across restarts). A route without
/// the marker maps straight under `static_dir`, matching the default.
///
/// Routes outside [`CACHEABLE_PATHS`] map to the [`ISR_UNCACHEABLE_SENTINEL`]
/// instead, which the renderer can neither read a page from nor write one to.
fn isr_map_path(static_dir: &std::path::Path, route: &str) -> PathBuf {
    let (path, query) = route.split_once('?').unwrap_or((route, ""));
    if !is_cacheable_path(path) {
        return static_dir.join(ISR_UNCACHEABLE_SENTINEL).join("route");
    }

    let mut mapped = static_dir.to_path_buf();
    if let Some(locale) = locale_from_query(query) {
        // Safe as a path component only because `locale_from_query` rejects
        // anything that is not a known language code.
        mapped.push(locale);
    }
    for segment in path.split('/') {
        mapped.push(segment);
    }
    mapped
}

/// Whether a full incremental-cache route (path plus query) addresses a page
/// whose render may be persisted.
fn is_cacheable_route(route: &str) -> bool {
    let (path, _) = route.split_once('?').unwrap_or((route, ""));
    is_cacheable_path(path)
}

/// Extracts the [`ISR_LOCALE_PARAM`] value from a raw query string, if it names
/// a language the site actually supports.
///
/// The validation is the load-bearing part: the marker is server-internal, but
/// it rides in on a client-supplied URI, so anything that is not a [`LANGUAGES`]
/// entry is a forged value rather than a locale — and this value goes on to
/// become a directory name. Rejecting it here means the only strings that ever
/// reach [`isr_map_path`] are the compile-time language codes.
fn locale_from_query(query: &str) -> Option<&str> {
    query
        .split('&')
        .filter_map(|pair| pair.strip_prefix(ISR_LOCALE_QUERY_PREFIX))
        .find(|locale| LANGUAGES.contains(locale))
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

/// Whether `mapped` already holds a rendered page.
///
/// Dioxus's `FileSystemCache` uses two different on-disk layouts depending on
/// whether a TTL is configured, and both have to be recognised here — checking
/// only the timestamped one made this always report "not cached" under the
/// default (permanent) configuration, which in turn kept every announced path
/// in memory forever:
/// * permanent cache (no TTL): the page is `<mapped>/index.html`;
/// * finite TTL: the page is `<mapped>/index/<timestamp>.html`, so any `.html`
///   in that sub-directory counts.
fn has_cached_render(mapped: &Path) -> bool {
    if mapped.join("index.html").is_file() {
        return true;
    }
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
///
/// `assets` is the router's state purely for the readiness probe, which is the
/// one handler whose answer depends on configuration rather than on
/// compile-time data. Holding it as state rather than re-reading it per request
/// means the probe cannot start disagreeing with the rest of the process about
/// where the bundle is.
fn api_router(assets: AssetsConfig) -> Router {
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
        .with_state(assets)
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
        api_router(AssetsConfig::default())
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

    /// The middleware state, with the policy the deployment's defaults produce.
    fn page_state(isr_enabled: bool) -> PageState {
        PageState {
            isr_enabled,
            policy: Arc::new(csp::SitePolicy::new(&portfolio_config::CspConfig::default())),
            pages: Arc::new(PageMemo::default()),
        }
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
        let bare = api_router(AssetsConfig::default())
            .oneshot(request())
            .await
            .unwrap();
        let layered = api_router(AssetsConfig::default())
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
        let response = api_router(AssetsConfig::default())
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
    fn a_client_supplied_locale_marker_is_stripped_before_ours_is_appended() {
        // The marker decides the cache bucket, so a request must never be able
        // to bring its own: the negotiated value has to be the only one left.
        let tagged = with_locale_query(&"/?__isr_locale=de".parse().unwrap(), "en");
        assert_eq!(
            tagged.path_and_query().unwrap().as_str(),
            "/?__isr_locale=en"
        );

        // Including when it is smuggled among real parameters, repeated, or
        // valueless — and unrelated parameters still survive.
        let tagged = with_locale_query(
            &"/imprint?__isr_locale=de&ref=x&__isr_locale&__isr_locale=fr"
                .parse()
                .unwrap(),
            "en",
        );
        assert_eq!(
            tagged.path_and_query().unwrap().as_str(),
            "/imprint?ref=x&__isr_locale=en"
        );
    }

    #[test]
    fn a_forged_locale_never_becomes_a_path_component() {
        let base = std::path::Path::new("/cache");
        let untagged = isr_map_path(base, "/");

        // Traversal, absolute paths and unknown languages are all rejected by
        // `locale_from_query`, so the route maps exactly as if no marker were
        // present rather than escaping the cache directory.
        for forged in [
            "/?__isr_locale=../../../../etc/evil",
            "/?__isr_locale=/var/www/html",
            "/?__isr_locale=fr",
            "/?__isr_locale=",
        ] {
            let mapped = isr_map_path(base, forged);
            assert_eq!(mapped, untagged, "{forged} was not neutralised");
            assert!(
                mapped.starts_with(base),
                "{forged} escaped the cache directory"
            );
        }
    }

    #[test]
    fn only_allowlisted_pages_are_cacheable() {
        for path in ["/", "/imprint", "/privacy", "/licenses"] {
            assert!(is_cacheable_path(path), "{path} should be cacheable");
        }
        // The catch-all 404 route matches unboundedly many URLs; caching them
        // would let any client fill the cache volume.
        for path in ["/nope", "/1", "/imprint/x", "/robots.txt", ""] {
            assert!(!is_cacheable_path(path), "{path} should not be cacheable");
        }
    }

    #[test]
    fn uncacheable_routes_collapse_onto_an_unwritable_sentinel() {
        let dir = scratch_path("sentinel");
        std::fs::create_dir_all(&dir).unwrap();
        ensure_uncacheable_sentinel(&dir).unwrap();

        // Every non-cacheable route maps to the same place, so their number
        // cannot grow the cache...
        let a = isr_map_path(&dir, "/nope?__isr_locale=en");
        let b = isr_map_path(&dir, "/other?__isr_locale=de");
        assert_eq!(a, b);
        assert_ne!(a, isr_map_path(&dir, "/?__isr_locale=en"));

        // ...and that place cannot hold a render, because a regular file sits
        // where the renderer would need a directory.
        assert!(dir.join(ISR_UNCACHEABLE_SENTINEL).is_file());
        assert!(std::fs::create_dir_all(&a).is_err());
        assert!(!has_cached_render(&a));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_sentinel_replaces_a_directory_left_by_an_older_deploy() {
        let dir = scratch_path("sentinel-dir");
        let stale = dir
            .join(ISR_UNCACHEABLE_SENTINEL)
            .join("route")
            .join("index");
        std::fs::create_dir_all(&stale).unwrap();
        std::fs::write(stale.join("old.html"), b"<html></html>").unwrap();

        ensure_uncacheable_sentinel(&dir).unwrap();
        assert!(dir.join(ISR_UNCACHEABLE_SENTINEL).is_file());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_permanent_cache_entry_is_recognised_as_rendered() {
        // Without a TTL the renderer writes `<mapped>/index.html` rather than a
        // timestamped file under `<mapped>/index/`; both layouts must count as
        // cached, or the announcement bookkeeping never releases the path.
        let mapped = scratch_path("permanent");
        std::fs::create_dir_all(&mapped).unwrap();
        assert!(!has_cached_render(&mapped));

        std::fs::write(mapped.join("index.html"), b"<html></html>").unwrap();
        assert!(has_cached_render(&mapped));

        let _ = std::fs::remove_dir_all(&mapped);
    }

    #[test]
    fn html_lang_is_stamped_onto_the_opening_tag() {
        assert_eq!(
            with_html_lang("<!DOCTYPE html><html><head></head></html>", "de"),
            "<!DOCTYPE html><html lang=\"de\"><head></head></html>"
        );
        // An existing `lang` is replaced, not duplicated, and sibling attributes
        // are preserved.
        assert_eq!(
            with_html_lang("<html lang=\"en\" data-x=\"1\">", "de"),
            "<html data-x=\"1\" lang=\"de\">"
        );
        // A document without an `<html>` tag is served through untouched.
        assert_eq!(with_html_lang("<p>fragment</p>", "de"), "<p>fragment</p>");
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

    /// ISR is off unless the configuration names a cache directory. That answer
    /// is what [`dioxus_app_router`] turns into the flag gating locale tagging,
    /// so getting it wrong would leave every language sharing one cache entry.
    ///
    /// The TTL and empty-value semantics themselves belong to
    /// `portfolio_config::IsrConfig` and are tested there; what this pins is
    /// that the server asks it rather than reading the environment behind its
    /// back.
    #[test]
    fn isr_is_off_without_a_configured_cache_directory() {
        assert!(incremental_config(&IsrConfig::default()).is_none());
    }

    #[test]
    fn a_writable_cache_directory_turns_isr_on() {
        let dir = scratch_path("isr-config");
        let on = IsrConfig {
            cache_dir: Some(dir.clone()),
            ttl_secs: 0,
        };

        assert!(incremental_config(&on).is_some());
        // The sentinel is what keeps non-cacheable routes from minting entries;
        // enabling ISR must have created it as a regular file.
        assert!(dir.join(ISR_UNCACHEABLE_SENTINEL).is_file());

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A cache directory the process cannot write to fails safe: the server
    /// renders every request fresh rather than refusing to start or persisting
    /// nothing while believing it caches.
    #[test]
    fn an_unusable_cache_directory_disables_isr_instead_of_failing() {
        // A path *under a regular file* can never be created, which is the one
        // "unwritable" shape that reproduces identically on every platform
        // (a mode-0555 directory does not, since root ignores it).
        let blocker = scratch_path("isr-blocker");
        std::fs::write(&blocker, b"not a directory").unwrap();

        let unusable = IsrConfig {
            cache_dir: Some(blocker.join("cache")),
            ttl_secs: 0,
        };
        assert!(incremental_config(&unusable).is_none());

        let _ = std::fs::remove_file(&blocker);
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

    #[tokio::test]
    async fn every_page_request_is_tagged_regardless_of_its_accept_header() {
        // Tagging must not depend on the client advertising `text/html`: a
        // request that omits it still gets a rendered page, and gating on the
        // header used to funnel all such requests into one shared, language-
        // blind cache entry. The middleware is exercised through a stub that
        // echoes the URI the router finally saw.
        async fn echo_uri(request: axum::extract::Request) -> String {
            request.uri().to_string()
        }

        let app = Router::new()
            .route("/", get(echo_uri))
            .layer(middleware::from_fn_with_state(
                page_state(true),
                localize_page,
            ));

        let seen = |accept: Option<&'static str>| {
            let app = app.clone();
            async move {
                let mut builder = Request::builder()
                    .uri("/")
                    .header(header::COOKIE, "lang=de");
                if let Some(accept) = accept {
                    builder = builder.header(header::ACCEPT, accept);
                }
                let response = app
                    .oneshot(builder.body(Body::empty()).unwrap())
                    .await
                    .unwrap();
                let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
                String::from_utf8(bytes.to_vec()).unwrap()
            }
        };

        for accept in [Some("text/html,application/xhtml+xml"), Some("*/*"), None] {
            assert_eq!(
                seen(accept).await,
                "/?__isr_locale=de",
                "Accept: {accept:?} was not tagged"
            );
        }
    }

    #[tokio::test]
    async fn non_page_responses_are_left_alone() {
        // A request may accept HTML and still be answered with JSON; those do
        // not vary by language and have no tag to stamp.
        let response = api_router(AssetsConfig::default())
            .layer(middleware::from_fn_with_state(
                page_state(false),
                localize_page,
            ))
            .oneshot(
                Request::builder()
                    .uri("/api/v1/profile")
                    .header(header::ACCEPT, "text/html")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert!(!response.headers().contains_key(header::VARY));
    }

    /// A document's policy is derived from the document: the hash of the inline script it
    /// carries, the nonce Cloudflare copies onto what it injects at the edge, and the
    /// `Cache-Control` that keeps that nonce from being shared between readers.
    ///
    /// What the directives say belongs to [`csp`] and is tested there; what this pins is that
    /// the response pipeline actually reaches it, on the same buffered body it rewrites.
    #[tokio::test]
    async fn a_page_carries_a_policy_derived_from_its_own_html() {
        async fn page() -> axum::response::Response {
            axum::http::Response::builder()
                .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                .body(Body::from(
                    "<!DOCTYPE html><html><body><script>window.x=1;</script></body></html>",
                ))
                .unwrap()
        }

        let response = Router::new()
            .route("/", get(page))
            .layer(middleware::from_fn_with_state(
                page_state(false),
                localize_page,
            ))
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        let headers = response.headers();
        let policy = headers
            .get(header::CONTENT_SECURITY_POLICY)
            .expect("a document must carry a policy")
            .to_str()
            .unwrap();
        assert!(policy.contains("'sha256-"), "{policy}");
        assert!(policy.contains("'nonce-"), "{policy}");
        assert_eq!(headers.get(header::CACHE_CONTROL).unwrap(), "no-cache");
    }

    /// A `GET` for a page, optionally advertising a preferred language.
    fn page_request(path: &str, accept_language: Option<&str>) -> Request<Body> {
        let mut builder = Request::builder().uri(path);
        if let Some(value) = accept_language {
            builder = builder.header(header::ACCEPT_LANGUAGE, value);
        }
        builder.body(Body::empty()).unwrap()
    }

    async fn body_text(response: axum::response::Response) -> String {
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        String::from_utf8(bytes.to_vec()).expect("the page is UTF-8")
    }

    fn csp_of(response: &axum::response::Response) -> String {
        response
            .headers()
            .get(header::CONTENT_SECURITY_POLICY)
            .expect("a document must carry a policy")
            .to_str()
            .unwrap()
            .to_owned()
    }

    /// A router whose page handler renders something different on every call, so
    /// a response that repeats itself can only have come from [`PageMemo`].
    fn counting_page_router(state: PageState) -> Router {
        let renders = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let handler = move || {
            let renders = Arc::clone(&renders);
            async move {
                let n = renders.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                axum::http::Response::builder()
                    .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                    .body(Body::from(format!(
                        "<!DOCTYPE html><html><body>render {n}<script>window.x={n};</script></body></html>"
                    )))
                    .unwrap()
            }
        };
        Router::new()
            .route("/", get(handler.clone()))
            .route("/{*rest}", get(handler))
            .layer(middleware::from_fn_with_state(state, localize_page))
    }

    /// The second request for a cacheable page is answered from the memo: the
    /// same bytes, with none of the buffering, copying or re-hashing that
    /// produced them the first time.
    #[tokio::test]
    async fn a_cacheable_page_is_served_from_the_memo() {
        let router = counting_page_router(page_state(false));

        let first = router
            .clone()
            .oneshot(page_request("/", None))
            .await
            .unwrap();
        let first_csp = csp_of(&first);
        let first_body = body_text(first).await;

        let second = router
            .clone()
            .oneshot(page_request("/", None))
            .await
            .unwrap();
        let second_csp = csp_of(&second);
        let second_body = body_text(second).await;

        assert!(first_body.contains("render 1"), "{first_body}");
        assert!(first_body.contains("<html lang=\"en\">"), "{first_body}");
        assert_eq!(
            first_body, second_body,
            "the second response must be the memoized bytes"
        );

        // The hashes describe the document, so they repeat with it...
        let hashes = |csp: &str| csp.matches("'sha256-").count();
        assert_eq!(hashes(&first_csp), 1, "{first_csp}");
        assert_eq!(hashes(&first_csp), hashes(&second_csp));
        // ...while the nonce beside them must not: one reused across responses
        // is `'unsafe-inline'` with extra steps, memo or no memo.
        assert_ne!(first_csp, second_csp);
    }

    /// One entry per language. Serving whichever locale rendered a path first to
    /// every later visitor is the bug the locale in the key exists to prevent —
    /// the same one the incremental cache's locale marker prevents on disk.
    #[tokio::test]
    async fn the_memo_keeps_one_entry_per_locale() {
        let router = counting_page_router(page_state(false));

        let english = body_text(
            router
                .clone()
                .oneshot(page_request("/", None))
                .await
                .unwrap(),
        )
        .await;
        let german = body_text(
            router
                .clone()
                .oneshot(page_request("/", Some("de-DE,de")))
                .await
                .unwrap(),
        )
        .await;

        assert!(english.contains("<html lang=\"en\">"), "{english}");
        assert!(german.contains("<html lang=\"de\">"), "{german}");
        assert!(english.contains("render 1"), "{english}");
        assert!(german.contains("render 2"), "{german}");
    }

    /// The catch-all route is shared by every URL that does not exist, so a memo
    /// entry under it would answer one unknown URL with another's document.
    #[tokio::test]
    async fn an_uncacheable_route_is_rendered_fresh_every_time() {
        let router = counting_page_router(page_state(false));

        let first = body_text(
            router
                .clone()
                .oneshot(page_request("/1", None))
                .await
                .unwrap(),
        )
        .await;
        let second = body_text(
            router
                .clone()
                .oneshot(page_request("/2", None))
                .await
                .unwrap(),
        )
        .await;

        assert!(first.contains("render 1"), "{first}");
        assert!(second.contains("render 2"), "{second}");
        // Still localized: the 404 page is a page.
        assert!(second.contains("<html lang=\"en\">"), "{second}");
    }

    /// A failed render shares its path with the page it could not produce, so it
    /// must not take that page's place — otherwise one transient error would
    /// pin the error document for the rest of the process's life.
    #[tokio::test]
    async fn a_failed_render_is_not_memoized() {
        let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let handler = move || {
            let calls = Arc::clone(&calls);
            async move {
                let n = calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                let (status, text) = if n == 1 {
                    (StatusCode::INTERNAL_SERVER_ERROR, "the render failed")
                } else {
                    (StatusCode::OK, "the real page")
                };
                axum::http::Response::builder()
                    .status(status)
                    .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                    .body(Body::from(format!(
                        "<!DOCTYPE html><html><body>{text}</body></html>"
                    )))
                    .unwrap()
            }
        };
        let router = Router::new()
            .route("/", get(handler))
            .layer(middleware::from_fn_with_state(
                page_state(false),
                localize_page,
            ));

        let failed = router
            .clone()
            .oneshot(page_request("/", None))
            .await
            .unwrap();
        assert_eq!(failed.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert!(body_text(failed).await.contains("the render failed"));

        // Had the failure been remembered, this would still be the error page.
        let recovered = router
            .clone()
            .oneshot(page_request("/", None))
            .await
            .unwrap();
        assert_eq!(recovered.status(), StatusCode::OK);
        assert!(body_text(recovered).await.contains("the real page"));
    }

    /// The document policy survives the layer that supplies the subresource one.
    ///
    /// This mirrors the two layers [`router`] puts either side of the page middleware, and it is
    /// the assumption the whole design rests on: the subresource policy is applied *outside*
    /// [`localize_page`], so it runs on the way out, after the document already has its own. Only
    /// `if_not_present` makes that safe — with the `overriding` every other security header uses,
    /// a document's hashes would be replaced by a policy that admits no inline script at all, and
    /// every page would render blank.
    #[tokio::test]
    async fn the_subresource_layer_does_not_overwrite_a_document_policy() {
        async fn page() -> axum::response::Response {
            axum::http::Response::builder()
                .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                .body(Body::from(
                    "<html><body><script>window.x=1;</script></body></html>",
                ))
                .unwrap()
        }

        let state = page_state(false);
        let subresource = state.policy.subresource();
        let response = Router::new()
            .route("/", get(page))
            .layer(middleware::from_fn_with_state(state, localize_page))
            .layer(SetResponseHeaderLayer::if_not_present(
                header::CONTENT_SECURITY_POLICY,
                subresource.clone(),
            ))
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        let served = response.headers().get(header::CONTENT_SECURITY_POLICY);
        assert_ne!(served, Some(&subresource));
        assert!(
            served.unwrap().to_str().unwrap().contains("'sha256-"),
            "the document policy was replaced by the subresource one"
        );
    }

    /// A response that is not a document gets no policy here, so the one the outer layer applies
    /// — which admits no inline script at all — is the one that stands.
    #[tokio::test]
    async fn non_documents_are_left_to_the_subresource_policy() {
        let response = api_router(AssetsConfig::default())
            .layer(middleware::from_fn_with_state(
                page_state(false),
                localize_page,
            ))
            .oneshot(
                Request::builder()
                    .uri("/api/v1/profile")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert!(
            !response
                .headers()
                .contains_key(header::CONTENT_SECURITY_POLICY)
        );
    }
}
