//! Finishing document responses: the negotiated language, the per-document
//! Content-Security-Policy, and the memo that serves a finished page without rendering it again.
//!
//! [`localize_page`] is layered *outside* the Dioxus router, so it sees every page request —
//! incremental-cache hits included, whose render never ran. That makes it the one place that can
//! guarantee the three things a document needs whichever way it was produced: `<html lang>`, the
//! `lang` cookie agreeing with it, and a policy derived from the bytes being sent.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use axum::{
    body::Body,
    http::{HeaderMap, HeaderValue, StatusCode, header},
    middleware,
    response::IntoResponse,
};

use super::csp;
use super::isr::{PagePaths, with_locale_query};
use crate::i18n::{Negotiated, lang_cookie, negotiate_locale};

/// What [`localize_page`] carries per request.
///
/// Cloned for every request the middleware sees, so the policy and the memo are behind an
/// [`Arc`] rather than copied.
#[derive(Clone)]
pub(super) struct PageState {
    /// Whether the incremental cache is active, and therefore whether the request URI has to
    /// carry the negotiated language for the cache to key on.
    pub(super) isr_enabled: bool,
    /// The Content-Security-Policy this server serves. See [`csp`].
    pub(super) policy: Arc<csp::SitePolicy>,
    /// Finished pages, memoized per `(path, language)`. See [`PageMemo`].
    pub(super) pages: Arc<PageMemo>,
    /// The pages worth remembering. See [`PagePaths`].
    pub(super) paths: PagePaths,
    /// How long a memoized page may be served: the incremental cache's TTL, so configuring
    /// time-based revalidation revalidates this too. `None` keeps a page for the process's
    /// lifetime, like the cache.
    pub(super) ttl: Option<Duration>,
}

/// A finished page: the headers and body exactly as they will be sent, and the inline-script
/// hashes taken from those very bytes.
///
/// Stored and served as a unit and never mixed, which is what makes memoizing them safe: a
/// policy is only ever applied to the document it was scanned from.
pub(super) struct RenderedPage {
    /// The render's own response headers, without the per-response ones
    /// ([`finish_page`] adds those).
    headers: HeaderMap,
    /// The body with `lang` already stamped onto its `<html>` tag.
    body: axum::body::Bytes,
    /// What [`csp::SitePolicy::scan`] found in [`Self::body`].
    scan: csp::DocumentScan,
    /// When the render happened, for [`PageState::ttl`].
    rendered_at: Instant,
}

/// Finished pages, keyed by `(path, language)`.
///
/// Consulted *before* the router: a hit is answered without rendering, reading the incremental
/// cache from disk, validating UTF-8, stamping `lang` or hashing a script — only the nonce and
/// the cookie are per response.
///
/// Sound for exactly the reason the incremental cache is: a [`PagePaths`] page is rendered from
/// compile-time data and configuration read once at boot, so its bytes are a function of the
/// path and the negotiated language for the life of the process (or for [`PageState::ttl`],
/// when one is configured). Both halves of the key are `&'static str` drawn from fixed lists,
/// which bounds the map at `paths.len() * LANGUAGES.len()` entries — it needs no eviction.
pub(super) type PageMemo = RwLock<HashMap<(&'static str, &'static str), Arc<RenderedPage>>>;

/// The memoized page for `key`, if one has been rendered and is still within `ttl`.
///
/// A poisoned lock is recovered from rather than propagated: nothing can panic while it is
/// held, and a request must not fail over a structure it could just as well have missed in.
fn memo_get(
    memo: &PageMemo,
    key: (&'static str, &'static str),
    ttl: Option<Duration>,
) -> Option<Arc<RenderedPage>> {
    memo.read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get(&key)
        .filter(|page| ttl.is_none_or(|ttl| page.rendered_at.elapsed() < ttl))
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
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .insert(key, Arc::clone(&page));
    page
}

/// Middleware finishing every `GET` the server answers with a document:
///
/// 1. A [`PagePaths`] page already rendered in this language is answered from [`PageMemo`]
///    without reaching the router at all.
/// 2. When `isr_enabled`, the language is folded into the URI query, so the incremental
///    renderer — which keys its cache by path and query only — keeps one entry per language.
/// 3. The response declares `Vary: Accept-Language, Cookie`, the request headers its content
///    depends on, so a shared cache in front of the origin does not hand one language to
///    another's reader.
/// 4. The opening `<html>` tag is stamped with `lang`, so the language is declared in the first
///    bytes a crawler receives — and the wasm client, which reads it back, hydrates in it.
/// 5. The `lang` cookie is written when the request's does not match, including on pages the
///    incremental cache answered, whose render never ran.
/// 6. The document gets its own Content-Security-Policy, derived from the inline scripts in the
///    bytes about to be sent (see [`csp`]).
///
/// The language is decided by `negotiate_locale`, as it is for the render, so the cache key,
/// the `<html lang>`, the cookie and the HTML always agree. Requests that are not `GET` —
/// server-function calls are `POST`s — pass straight through.
pub(super) async fn localize_page(
    axum::extract::State(state): axum::extract::State<PageState>,
    mut request: axum::extract::Request,
    next: middleware::Next,
) -> axum::response::Response {
    if request.method() != axum::http::Method::GET {
        return next.run(request).await;
    }

    let negotiated = {
        let headers = request.headers();
        let read_header = |name| headers.get(name).and_then(|v| v.to_str().ok());
        negotiate_locale(
            request.uri().query(),
            read_header(header::COOKIE),
            read_header(header::ACCEPT_LANGUAGE),
        )
    };
    let lang = negotiated.language.code;

    // The compile-time entry rather than the request's own slice, so the memo keys on a
    // program constant. `None` for anything else: the catch-all route is shared by every
    // unknown URL, so remembering one of their renders would answer the next with it.
    let memo_key = state.paths.get(request.uri().path());

    if let Some(page) = memo_key.and_then(|path| memo_get(&state.pages, (path, lang), state.ttl)) {
        let mut response = axum::response::Response::new(Body::empty());
        *response.headers_mut() = page.headers.clone();
        return finish_page(response.into_parts().0, &state.policy, &page, negotiated);
    }

    if state.isr_enabled && memo_key.is_some() {
        *request.uri_mut() = with_locale_query(request.uri(), lang);
    }

    // Whether the response is a page is decided by its content type, not by what the request
    // asked for, so the 404 page is localized too.
    rewrite_html_response(next.run(request).await, negotiated, &state, memo_key).await
}

/// Largest HTML page body this server will buffer in order to stamp `<html lang>` on it.
/// Rendered pages are tens of kilobytes; the limit only bounds memory if something upstream
/// ever produces an unexpectedly large `text/html` response.
const MAX_HTML_REWRITE_BYTES: usize = 4 * 1024 * 1024;

/// Finishes a rendered response: `Vary`, `<html lang>`, the cookie and the document policy —
/// and memoizes it under `memo_key` when it is a successful render of a [`PagePaths`] page.
///
/// Only `text/html` responses are touched: a request that merely *accepts* HTML may still be
/// answered with JSON or an asset, and those neither vary by language nor have a tag to
/// annotate. They pick up the subresource policy from the layer outside this one.
///
/// The SSR body arrives as a stream whose chunk boundaries are not ours to rely on, so the page
/// is buffered. That is affordable because rendering is not streamed out of order
/// (`StreamingMode::Disabled`, the default); enabling out-of-order streaming would mean
/// revisiting this. A body that is unreadable or larger than [`MAX_HTML_REWRITE_BYTES`] fails
/// the request rather than serving a truncated page.
async fn rewrite_html_response(
    mut response: axum::response::Response,
    negotiated: Negotiated,
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

    let (mut parts, body) = response.into_parts();
    let lang = negotiated.language.code;
    // Only a successful render is worth remembering: an error page shares its path with the
    // document it failed to produce, so keying one as the other would answer a later request
    // with the error — or hand the error page's body a 200's policy.
    let key = memo_key
        .filter(|_| parts.status == StatusCode::OK)
        .map(|path| (path, lang));

    let Ok(bytes) = axum::body::to_bytes(body, MAX_HTML_REWRITE_BYTES).await else {
        tracing::error!("page body was unreadable or exceeded {MAX_HTML_REWRITE_BYTES} bytes");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    let Ok(html) = std::str::from_utf8(&bytes) else {
        // A document that is not UTF-8 cannot be scanned for the scripts a policy would have to
        // admit, so it keeps the subresource policy the outer layer supplies.
        tracing::error!("page body was not valid UTF-8; serving it unmodified");
        return axum::response::Response::from_parts(parts, Body::from(bytes));
    };

    // Only what the render itself decided is kept for replay; the per-response headers are
    // added by `finish_page` every time.
    parts.headers.remove(header::CONTENT_LENGTH);
    parts.headers.remove(header::SET_COOKIE);
    let localized = with_html_lang(html, lang);
    let page = RenderedPage {
        headers: parts.headers.clone(),
        scan: state.policy.scan(&localized),
        body: axum::body::Bytes::from(localized),
        rendered_at: Instant::now(),
    };
    let page = match key {
        Some(key) => memo_insert(&state.pages, key, page),
        None => Arc::new(page),
    };
    finish_page(parts, &state.policy, &page, negotiated)
}

/// Assembles the response for a finished page: the policy for the scripts in its body, the
/// `lang` cookie when the request's disagrees, then the body itself.
///
/// Reached from both the memo hit and the first render, which must not drift — in particular
/// over the `Content-Length` below, which is wrong on either path.
fn finish_page(
    mut parts: axum::http::response::Parts,
    policy: &csp::SitePolicy,
    page: &RenderedPage,
    negotiated: Negotiated,
) -> axum::response::Response {
    policy.apply_to_document(&mut parts.headers, &page.scan);
    parts.headers.remove(header::CONTENT_LENGTH);
    if negotiated.persist
        && let Ok(cookie) = HeaderValue::try_from(lang_cookie(negotiated.language.code))
    {
        parts.headers.append(header::SET_COOKIE, cookie);
    }
    axum::response::Response::from_parts(parts, Body::from(page.body.clone()))
}

/// Rewrites the opening `<html …>` tag of `html` to carry `lang="<locale>"`, replacing any
/// `lang` already present. Returns the document unchanged when it has no `<html>` tag.
fn with_html_lang(html: &str, locale: &str) -> String {
    let Some(open) = html.find("<html") else {
        return html.to_string();
    };
    // The tag is plain server-generated markup, so the first `>` after `<html` ends it; there
    // is no attribute value in between that could contain one.
    let Some(close) = html[open..].find('>').map(|i| open + i) else {
        return html.to_string();
    };

    let attrs = &html[open + "<html".len()..close];
    let mut out = String::with_capacity(html.len() + 16);
    out.push_str(&html[..open]);
    out.push_str("<html");
    for attr in attrs.split_whitespace() {
        let name = attr.split('=').next().unwrap_or(attr);
        if !name.eq_ignore_ascii_case("lang") {
            out.push(' ');
            out.push_str(attr);
        }
    }
    // `locale` is a `LANGUAGES` entry, never request text, so it needs no escaping.
    out.push_str(" lang=\"");
    out.push_str(locale);
    out.push('"');
    out.push_str(&html[close..]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::isr::tests::paths;
    use crate::server::{api_router, legal};
    use axum::{
        Router,
        body::to_bytes,
        http::{Request, StatusCode},
        routing::get,
    };
    use portfolio_config::AssetsConfig;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tower::ServiceExt;
    use tower_http::set_header::SetResponseHeaderLayer;

    /// The middleware state, with the policy the deployment's defaults produce.
    fn page_state(isr_enabled: bool) -> PageState {
        PageState {
            isr_enabled,
            policy: Arc::new(csp::SitePolicy::new(&portfolio_config::CspConfig::default())),
            pages: Arc::new(PageMemo::default()),
            paths: paths(),
            ttl: None,
        }
    }

    /// A `GET` for a page, optionally advertising a preferred language and a cookie.
    fn page_request(path: &str, accept_language: Option<&str>) -> Request<Body> {
        page_request_with_cookie(path, accept_language, None)
    }

    fn page_request_with_cookie(
        path: &str,
        accept_language: Option<&str>,
        cookie: Option<&str>,
    ) -> Request<Body> {
        let mut builder = Request::builder().uri(path);
        if let Some(value) = accept_language {
            builder = builder.header(header::ACCEPT_LANGUAGE, value);
        }
        if let Some(value) = cookie {
            builder = builder.header(header::COOKIE, value);
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

    fn cookie_of(response: &axum::response::Response) -> Option<String> {
        response
            .headers()
            .get(header::SET_COOKIE)
            .map(|value| value.to_str().unwrap().to_owned())
    }

    /// A router whose page handler renders something different on every call, and the number
    /// of calls it received — so a response that repeats itself, with the count unchanged, can
    /// only have come from [`PageMemo`].
    fn counting_page_router(state: PageState) -> (Router, Arc<AtomicUsize>) {
        let renders = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&renders);
        let handler = move || {
            let renders = Arc::clone(&renders);
            async move {
                let n = renders.fetch_add(1, Ordering::SeqCst) + 1;
                axum::http::Response::builder()
                    .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                    .body(Body::from(format!(
                        "<!DOCTYPE html><html><body>render {n}<script>window.x={n};</script></body></html>"
                    )))
                    .unwrap()
            }
        };
        let router = Router::new()
            .route("/", get(handler.clone()))
            .route("/{*rest}", get(handler))
            .layer(middleware::from_fn_with_state(state, localize_page));
        (router, counter)
    }

    #[test]
    fn html_lang_is_stamped_onto_the_opening_tag() {
        assert_eq!(
            with_html_lang("<!DOCTYPE html><html><head></head></html>", "de"),
            "<!DOCTYPE html><html lang=\"de\"><head></head></html>"
        );
        assert_eq!(
            with_html_lang("<html lang=\"en\" data-x=\"1\">", "de"),
            "<html data-x=\"1\" lang=\"de\">"
        );
        assert_eq!(with_html_lang("<p>fragment</p>", "de"), "<p>fragment</p>");
    }

    #[tokio::test]
    async fn every_page_request_is_tagged_regardless_of_its_accept_header() {
        // Tagging must not depend on the client advertising `text/html`: a request that omits
        // it still gets a rendered page. The stub echoes the URI the router finally saw.
        async fn echo_uri(request: axum::extract::Request) -> String {
            request.uri().to_string()
        }

        let app = Router::new()
            .route("/", get(echo_uri))
            .layer(middleware::from_fn_with_state(
                page_state(true),
                localize_page,
            ));

        for accept in [Some("text/html,application/xhtml+xml"), Some("*/*"), None] {
            let mut builder = Request::builder()
                .uri("/")
                .header(header::COOKIE, "lang=de");
            if let Some(accept) = accept {
                builder = builder.header(header::ACCEPT, accept);
            }
            let response = app
                .clone()
                .oneshot(builder.body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(
                body_text(response).await,
                "/?__isr_locale=de",
                "Accept: {accept:?} was not tagged"
            );
        }
    }

    #[tokio::test]
    async fn non_page_responses_are_left_alone() {
        let response = api_router(AssetsConfig::default(), legal::tests::fixture())
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
        assert!(!response.headers().contains_key(header::SET_COOKIE));
    }

    /// What the directives say belongs to [`csp`]; this pins that the response pipeline reaches
    /// it, on the same buffered body it rewrites.
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

        let policy = csp_of(&response);
        assert!(policy.contains("'sha256-"), "{policy}");
        assert!(policy.contains("'nonce-"), "{policy}");
        assert_eq!(
            response.headers().get(header::CACHE_CONTROL).unwrap(),
            "no-cache"
        );
    }

    /// The second request for a cacheable page never reaches the router: the same bytes, with
    /// a fresh nonce.
    #[tokio::test]
    async fn a_cacheable_page_is_served_from_the_memo_without_rendering() {
        let (router, renders) = counting_page_router(page_state(false));

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
        assert_eq!(second.status(), StatusCode::OK);
        assert_eq!(
            second.headers().get(header::CONTENT_TYPE).unwrap(),
            "text/html; charset=utf-8"
        );
        assert_eq!(
            second.headers().get(header::VARY).unwrap(),
            "accept-language, cookie"
        );
        let second_csp = csp_of(&second);
        let second_body = body_text(second).await;

        assert_eq!(
            renders.load(Ordering::SeqCst),
            1,
            "the memo hit rendered again"
        );
        assert!(first_body.contains("<html lang=\"en\">"), "{first_body}");
        assert_eq!(first_body, second_body);

        let hashes = |csp: &str| csp.matches("'sha256-").count();
        assert_eq!(hashes(&first_csp), 1, "{first_csp}");
        assert_eq!(hashes(&first_csp), hashes(&second_csp));
        // A nonce reused across responses is `'unsafe-inline'` with extra steps.
        assert_ne!(first_csp, second_csp);
    }

    #[tokio::test]
    async fn the_memo_keeps_one_entry_per_locale() {
        let (router, _) = counting_page_router(page_state(false));

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

    /// A memo entry older than the configured TTL is rendered again, as the incremental cache's
    /// would be.
    #[tokio::test]
    async fn an_expired_memo_entry_is_rendered_again() {
        let state = PageState {
            ttl: Some(Duration::ZERO),
            ..page_state(false)
        };
        let (router, renders) = counting_page_router(state);

        router
            .clone()
            .oneshot(page_request("/", None))
            .await
            .unwrap();
        router
            .clone()
            .oneshot(page_request("/", None))
            .await
            .unwrap();
        assert_eq!(renders.load(Ordering::SeqCst), 2);
    }

    /// The cookie is what the client falls back to after `<html lang>`, and what the next full
    /// load negotiates from, so it has to agree with the page — including one answered from the
    /// memo, whose render (and with it any header the render would set) never ran.
    #[tokio::test]
    async fn the_negotiated_language_is_written_back_as_the_cookie() {
        let (router, _) = counting_page_router(page_state(false));

        for attempt in ["render", "memo"] {
            let response = router
                .clone()
                .oneshot(page_request("/", Some("de-DE,de")))
                .await
                .unwrap();
            let cookie = cookie_of(&response).unwrap_or_default();
            assert!(cookie.starts_with("lang=de;"), "{attempt}: {cookie}");
        }

        // A cookie that already agrees is not rewritten on every response.
        let response = router
            .clone()
            .oneshot(page_request_with_cookie("/", None, Some("lang=de")))
            .await
            .unwrap();
        assert_eq!(cookie_of(&response), None);
        assert!(body_text(response).await.contains("<html lang=\"de\">"));
    }

    /// `?lang=` gives each language an address of its own and outranks the cookie.
    #[tokio::test]
    async fn the_query_parameter_selects_the_language() {
        let (router, _) = counting_page_router(page_state(false));

        let response = router
            .clone()
            .oneshot(page_request_with_cookie("/?lang=de", None, Some("lang=en")))
            .await
            .unwrap();
        assert!(cookie_of(&response).unwrap().starts_with("lang=de;"));
        assert!(body_text(response).await.contains("<html lang=\"de\">"));
    }

    #[tokio::test]
    async fn an_uncacheable_route_is_rendered_fresh_every_time() {
        let (router, _) = counting_page_router(page_state(false));

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
        assert!(second.contains("<html lang=\"en\">"), "{second}");
    }

    #[tokio::test]
    async fn a_failed_render_is_not_memoized() {
        let calls = Arc::new(AtomicUsize::new(0));
        let handler = move || {
            let calls = Arc::clone(&calls);
            async move {
                let n = calls.fetch_add(1, Ordering::SeqCst) + 1;
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

        let recovered = router
            .clone()
            .oneshot(page_request("/", None))
            .await
            .unwrap();
        assert_eq!(recovered.status(), StatusCode::OK);
        assert!(body_text(recovered).await.contains("the real page"));
    }

    /// The subresource policy is applied *outside* [`localize_page`], so it runs after the
    /// document already has its own; only `if_not_present` makes that safe.
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

    #[tokio::test]
    async fn non_documents_are_left_to_the_subresource_policy() {
        let response = api_router(AssetsConfig::default(), legal::tests::fixture())
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
