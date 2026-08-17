//! The `Content-Security-Policy` this server sends, built with `csp-shell`.
//!
//! Two policies, because a stylesheet and a document do not need the same one:
//!
//! * every response that is **not** a document carries [`SitePolicy::subresource`], which admits
//!   no inline script at all — a `.css`, a `.woff2`, a PDF or a JSON body has none to admit, and
//!   an allowance nothing uses is an allowance an HTML-sniffing browser bug could use;
//! * a document carries [`SitePolicy::apply_to_document`], derived from the HTML about to be sent.
//!
//! # Why the document policy is per response
//!
//! Dioxus renders inline script into every page: the streaming bootstrap, and
//! `window.initial_dioxus_hydration_data="…"`, whose text is the serialised state of *that*
//! render. There is no shell on disk carrying it, so nothing about it is knowable at start-up —
//! `csp-shell`'s file scanner is the wrong half of the crate here, and [`csp_shell::scan_shell`],
//! which takes the document's text, is the right one. The body is already buffered for the
//! `<html lang>` rewrite, so scanning it costs a SHA-256 per inline script and no extra read.
//!
//! The alternative — `'unsafe-inline'`, which is what this server sent before — admits an
//! injected `<script>` exactly as readily as the hydration bootstrap. It remains reachable
//! through `csp.hash_inline_scripts = false`, because the failure mode of getting hashing wrong
//! is a blank page rather than a loud one.
//!
//! # What Cloudflare adds
//!
//! Cloudflare's bot products inject their own inline `<script>` at the edge, *after* this process
//! has hashed what it renders, so no hash here can cover it. A nonce can: Cloudflare parses the
//! `Content-Security-Policy` response header and copies the nonce onto what it injects, which is
//! why nothing is stamped into the document. The other two presets are plain origin allowances,
//! off unless the deployment turns them on. See `portfolio_config::CloudflareConfig`.
//!
//! # Where it differs from the string constant this replaced
//!
//! `Csp::spa_wasm` is a tighter starting point than the hand-written header was, so two
//! directives moved and one had to be put back:
//!
//! * `base-uri` is `'none'` rather than `'self'` — this site renders no `<base>` element (Dioxus
//!   applies a base path by nesting the router, not by emitting a tag), so nothing may set one;
//! * `img-src` and `font-src` drop the `https:` and `data:` sources the preset adds, since every
//!   image and every font on this site is served from its own origin;
//! * `upgrade-insecure-requests` is restored, which the preset does not carry.

use std::collections::HashSet;
use std::sync::Mutex;

use axum::http::{HeaderMap, HeaderValue, header};
use csp_shell::{
    Csp, Directive, ScanWarning, Scheme, Source, SourceDirective, presets::cloudflare,
};
use portfolio_config::CspConfig;

/// The policies this server serves, rendered as far ahead of the response as each one can be.
#[derive(Debug)]
pub(super) struct SitePolicy {
    /// For responses that are not documents. Constant, so it is rendered once.
    subresource: HeaderValue,
    /// For documents.
    document: DocumentPolicy,
    /// Scanner limits already reported, so a steady-state one is logged once rather than on
    /// every response.
    ///
    /// A warning is a property of what the renderer emits, not of a single request, so the
    /// second occurrence carries no information the first did not — and at any real request rate
    /// an unconditional `error!` would bury the line it was meant to make visible. Taken only
    /// when a scan actually warns, which is never in the normal case.
    reported: Mutex<HashSet<ScanWarning>>,
}

/// What a document's own bytes contribute to its policy: the `'sha256-…'` sources for the inline
/// scripts found in them.
///
/// Carried as a distinct type so a caller can hold one alongside the bytes it was taken from and
/// apply it repeatedly. It is inert for a policy that does not hash — see [`SitePolicy::scan`].
#[derive(Debug, Clone, Default)]
pub(super) struct DocumentScan(csp_shell::ScanResult);

/// How a document response gets its policy.
#[derive(Debug, Clone)]
enum DocumentPolicy {
    /// `'unsafe-inline'`: nothing depends on the response, so the header is rendered once.
    Constant(HeaderValue),
    /// The builder the served document's hashes — and, when Cloudflare's nonce is reserved, a
    /// freshly minted nonce — are folded into per response.
    PerResponse(Csp),
}

impl SitePolicy {
    /// Build both policies from the configuration.
    ///
    /// The configuration is validated before this is reached
    /// (`portfolio_config::CspConfig::validate`), so the one combination that cannot be served —
    /// a nonce beside `'unsafe-inline'`, which a browser resolves by ignoring the latter — has
    /// already failed the boot.
    pub(super) fn new(config: &CspConfig) -> Self {
        let base = base_policy(config);

        let subresource = header_value(base.clone().build().headers().content_security_policy);
        let document = if config.hash_inline_scripts {
            DocumentPolicy::PerResponse(if config.cloudflare.script_nonce {
                cloudflare::script_nonce(base)
            } else {
                base
            })
        } else {
            DocumentPolicy::Constant(header_value(
                base.allow_unsafe_inline_script()
                    .build()
                    .headers()
                    .content_security_policy,
            ))
        };

        Self {
            subresource,
            document,
            reported: Mutex::new(HashSet::new()),
        }
    }

    /// The policy for every response that is not a document.
    ///
    /// Applied as a layer *outside* the document rewrite and only when the header is absent, so a
    /// document that has already been given its own policy keeps it.
    pub(super) fn subresource(&self) -> HeaderValue {
        self.subresource.clone()
    }

    /// Scan a document's bytes for the inline scripts its policy has to admit.
    ///
    /// Split from [`apply_to_document`](Self::apply_to_document) so a server that sends the same
    /// document more than once can scan it once: the hashes are a function of the bytes and never
    /// change while those bytes do not, whereas the nonce beside them has to be minted per
    /// response. `html` must be the body as it will be sent — a scan of anything else describes a
    /// document this server is not sending.
    ///
    /// Scanning is skipped entirely for a policy that does not hash; the empty result it returns
    /// is never read, because [`apply_to_document`](Self::apply_to_document) answers that case
    /// from the constant it rendered at start-up.
    pub(super) fn scan(&self, html: &str) -> DocumentScan {
        match &self.document {
            DocumentPolicy::Constant(_) => DocumentScan(csp_shell::ScanResult::default()),
            DocumentPolicy::PerResponse(_) => {
                let scan = csp_shell::scan_shell(html);
                self.report(&scan.warnings);
                DocumentScan(scan)
            }
        }
    }

    /// Give a document response the policy for a [`DocumentScan`] of the very bytes it carries.
    ///
    /// The scan must come from [`scan`](Self::scan) on the same document: the hashes are what
    /// permit that document's inline scripts, and a policy derived from anything else refuses the
    /// scripts it was meant to permit.
    pub(super) fn apply_to_document(&self, headers: &mut HeaderMap, scan: &DocumentScan) {
        let (policy, cache_control) = match &self.document {
            DocumentPolicy::Constant(value) => (value.clone(), None),
            DocumentPolicy::PerResponse(builder) => {
                // The builder is cloned because rendering consumes it, and the hashes have to go
                // in before the nonce is spliced. A dozen small allocations against a response
                // that has already allocated the whole page as a `String`.
                let rendered = builder.clone().with_scan(&scan.0).build().headers();
                (
                    header_value(rendered.content_security_policy),
                    rendered.cache_control,
                )
            }
        };

        headers.insert(header::CONTENT_SECURITY_POLICY, policy);
        // An obligation of the nonce, not a caching preference: a nonce served from a cache is
        // pinned across every reader for the lifetime of that entry, which is `'unsafe-inline'`
        // with extra steps. It overrides rather than defers to `cache::set_cache_control` — that
        // middleware happens to answer `no-cache` for documents already, and this must not
        // depend on it continuing to.
        if let Some(value) = cache_control {
            headers.insert(header::CACHE_CONTROL, HeaderValue::from_static(value));
        }
    }

    /// Report each scanner limit the first time a response hits it.
    ///
    /// A warning means the hash this server computed is one the browser will not, so a script is
    /// about to be refused — and the only other evidence is a blank page and a console message
    /// nobody is watching. Logged at `error` for that reason.
    ///
    /// A poisoned lock is ignored rather than propagated: nothing here can panic while it is
    /// held, and a request must not fail over the bookkeeping for a log line.
    fn report(&self, warnings: &[ScanWarning]) {
        if warnings.is_empty() {
            return;
        }
        let Ok(mut reported) = self.reported.lock() else {
            return;
        };
        for warning in warnings {
            if reported.insert(*warning) {
                tracing::error!(
                    %warning,
                    "inline-script scan hit a scanner limit; the policy may refuse a script this page needs"
                );
            }
        }
    }
}

/// The policy both halves start from: everything that does not depend on the response.
fn base_policy(config: &CspConfig) -> Csp {
    let base = Csp::spa_wasm()
        // Every image and every font is served from this origin; the preset's `https:` and
        // `data:` would admit the whole web and an inline payload for nothing in return.
        .remove_source(SourceDirective::ImgSrc, &Source::Scheme(Scheme::Https))
        .remove_source(SourceDirective::FontSrc, &Source::Scheme(Scheme::Data))
        // `dioxus-web`'s document provider evaluates JavaScript through
        // `js_sys::Function::new_with_args` (`new Function(…)`) — it is how `document::Title`,
        // `document::Stylesheet`, `document::Meta` and friends are applied during hydration.
        // Without this the call throws an uncaught `EvalError` that aborts the wasm client and
        // freezes client-side navigation. `'wasm-unsafe-eval'`, which the preset already carries,
        // covers only the module instantiation.
        .allow_unsafe_eval()
        .set(Directive::UpgradeInsecureRequests)
        .expect("upgrade-insecure-requests has no source list to route");

    let base = if config.cloudflare.turnstile {
        cloudflare::turnstile(base)
    } else {
        base
    };
    if config.cloudflare.web_analytics {
        cloudflare::web_analytics(base)
    } else {
        base
    }
}

/// A rendered policy as a header value.
///
/// Infallible in practice and asserted as such: `csp-shell` assembles the value from types that
/// were checked when they were built, so it is ASCII and carries no `;` the builder did not emit
/// itself. Serving a document with no policy at all would be the worse outcome of the two.
fn header_value(policy: String) -> HeaderValue {
    HeaderValue::try_from(policy)
        .expect("a rendered policy is a valid header value by construction")
}

#[cfg(test)]
mod tests {
    use super::{SitePolicy, header};
    use axum::http::HeaderMap;
    use portfolio_config::{CloudflareConfig, CspConfig};

    /// A page shaped like the ones Dioxus renders: an external module script, which needs no
    /// hash, and two inline ones, whose text is only known per response.
    const PAGE: &str = concat!(
        "<!DOCTYPE html><html><head><script src=\"/wasm/web.js\"></script>",
        "<script>window.__streaming = 1;</script></head>",
        "<body><div id=\"main\"></div>",
        "<script>window.initial_dioxus_hydration_data=\"abc\";</script></body></html>"
    );

    fn document_policy(config: &CspConfig, html: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        let policy = SitePolicy::new(config);
        policy.apply_to_document(&mut headers, &policy.scan(html));
        headers
    }

    fn policy_of(headers: &HeaderMap) -> &str {
        headers
            .get(header::CONTENT_SECURITY_POLICY)
            .expect("a document must carry a policy")
            .to_str()
            .expect("the policy is ASCII")
    }

    /// The directives that carried over from the string constant this replaced, plus the two the
    /// preset tightened. Asserted on the subresource policy, which is the base both halves share.
    #[test]
    fn the_base_policy_states_what_the_site_needs_and_no_more() {
        let policy = SitePolicy::new(&CspConfig::default()).subresource();
        let policy = policy.to_str().expect("the policy is ASCII");

        for expected in [
            "default-src 'self'",
            "script-src 'self' 'wasm-unsafe-eval' 'unsafe-eval'",
            "style-src 'self' 'unsafe-inline'",
            "connect-src 'self'",
            "img-src 'self' data:",
            "font-src 'self'",
            "object-src 'none'",
            "base-uri 'none'",
            "form-action 'self'",
            "frame-ancestors 'none'",
            "upgrade-insecure-requests",
        ] {
            assert!(
                policy.contains(expected),
                "{expected} missing from {policy}"
            );
        }
        // The preset's own `https:` for images and `data:` for fonts are gone, and nothing
        // reintroduced a blanket inline-script allowance on a response that has no script at all.
        assert!(!policy.contains("img-src 'self' https:"), "{policy}");
        assert!(!policy.contains("font-src 'self' data:"), "{policy}");
        assert!(!policy.contains("'unsafe-inline'; script"), "{policy}");
        assert!(!policy.contains("sha256-"), "{policy}");
    }

    /// The default: one hash per inline script in the document, `'unsafe-inline'` gone, and the
    /// external script left to `'self'`.
    #[test]
    fn a_document_is_hashed_rather_than_admitted_wholesale() {
        let headers = document_policy(&CspConfig::default(), PAGE);
        let policy = policy_of(&headers);

        assert_eq!(policy.matches("'sha256-").count(), 2, "{policy}");
        // `'unsafe-inline'` survives in `style-src`, which the preset sets and no hash covers;
        // what must be gone is the script-src one.
        let script_src = policy
            .split("; ")
            .find(|d| d.starts_with("script-src"))
            .expect("script-src is set");
        assert!(!script_src.contains("'unsafe-inline'"), "{script_src}");
    }

    /// The same document rendered twice hashes identically, so a policy derived per response is
    /// still stable for a page that did not change. Only the nonce moves.
    #[test]
    fn the_hashes_are_a_function_of_the_document() {
        let no_nonce = CspConfig {
            cloudflare: CloudflareConfig {
                script_nonce: false,
                ..CloudflareConfig::default()
            },
            ..CspConfig::default()
        };
        let first = document_policy(&no_nonce, PAGE);
        let second = document_policy(&no_nonce, PAGE);
        assert_eq!(policy_of(&first), policy_of(&second));

        // A different document is a different policy, which is the point of deriving it.
        let other = document_policy(&no_nonce, "<script>other()</script>");
        assert_ne!(policy_of(&first), policy_of(&other));

        // No nonce reserved means no cache obligation to discharge.
        assert!(!policy_of(&first).contains("'nonce-"));
        assert!(!first.contains_key(header::CACHE_CONTROL));
    }

    /// Cloudflare's edge injection is covered by a nonce that is minted per response and never
    /// cached — the two halves of the contract, asserted together because either alone is
    /// `'unsafe-inline'` with extra steps.
    #[test]
    fn the_cloudflare_nonce_is_fresh_per_response_and_uncacheable() {
        let first = document_policy(&CspConfig::default(), PAGE);
        let second = document_policy(&CspConfig::default(), PAGE);

        assert!(
            policy_of(&first).contains("'nonce-"),
            "{}",
            policy_of(&first)
        );
        assert_ne!(
            policy_of(&first),
            policy_of(&second),
            "a nonce reused across responses restricts nothing"
        );
        assert_eq!(
            first
                .get(header::CACHE_CONTROL)
                .map(|v| v.to_str().unwrap()),
            Some("no-cache")
        );
    }

    /// One scan, applied twice, still mints two nonces. This is what lets the server memoize a
    /// page's scan beside its bytes and reuse both: the half that depends on the document is
    /// reusable, the half that must not be is not.
    #[test]
    fn a_reused_scan_still_yields_a_fresh_nonce_per_response() {
        let policy = SitePolicy::new(&CspConfig::default());
        let scan = policy.scan(PAGE);

        let mut first = HeaderMap::new();
        let mut second = HeaderMap::new();
        policy.apply_to_document(&mut first, &scan);
        policy.apply_to_document(&mut second, &scan);

        // The hashes are the reusable half: identical, and covering both inline scripts.
        let hashes = |headers: &HeaderMap| policy_of(headers).matches("'sha256-").count();
        assert_eq!(hashes(&first), 2);
        assert_eq!(hashes(&second), 2);
        // The nonce is not: a scan reused across responses must not pin it.
        assert_ne!(policy_of(&first), policy_of(&second));
    }

    /// A subresource never carries the nonce: it is minted for the document Cloudflare rewrites,
    /// and handing the same slot to a cacheable asset would pin one value into a shared cache.
    #[test]
    fn the_subresource_policy_reserves_no_nonce() {
        let policy = SitePolicy::new(&CspConfig::default()).subresource();
        assert!(!policy.to_str().unwrap().contains("'nonce-"));
    }

    /// The escape hatch, in both directions: `'unsafe-inline'` comes back and every hash goes,
    /// because a policy carrying both would make the browser ignore the former.
    #[test]
    fn turning_hashing_off_restores_unsafe_inline_alone() {
        let config = CspConfig {
            hash_inline_scripts: false,
            cloudflare: CloudflareConfig {
                script_nonce: false,
                ..CloudflareConfig::default()
            },
        };
        let headers = document_policy(&config, PAGE);
        let policy = policy_of(&headers);

        let script_src = policy
            .split("; ")
            .find(|d| d.starts_with("script-src"))
            .expect("script-src is set");
        assert!(script_src.contains("'unsafe-inline'"), "{script_src}");
        assert!(!policy.contains("'sha256-"), "{policy}");
        assert!(!policy.contains("'nonce-"), "{policy}");
    }

    /// Turnstile needs its origin in two directives — the script that loads the widget and the
    /// frame the widget renders in. Admitting only the first is the failure the preset exists to
    /// prevent, and it looks like a working policy.
    #[test]
    fn turnstile_admits_its_origin_in_both_directives_it_needs() {
        let config = CspConfig {
            cloudflare: CloudflareConfig {
                turnstile: true,
                ..CloudflareConfig::default()
            },
            ..CspConfig::default()
        };
        let policy = SitePolicy::new(&config).subresource();
        let policy = policy.to_str().unwrap();

        assert!(policy.contains("script-src 'self' 'wasm-unsafe-eval' 'unsafe-eval' https://challenges.cloudflare.com"), "{policy}");
        assert!(
            policy.contains("frame-src 'self' https://challenges.cloudflare.com"),
            "{policy}"
        );
        // The frame directive was absent, so the preset seeded it from `default-src` before
        // appending: creating it empty would have revoked same-origin frames.
        assert!(!policy.contains("frame-src https://"), "{policy}");
    }

    /// Web Analytics is two hosts in two directives, and the `static.` prefix on only one of them
    /// is the detail a hand-written policy gets wrong.
    #[test]
    fn web_analytics_admits_the_beacon_and_the_endpoint_it_reports_to() {
        let config = CspConfig {
            cloudflare: CloudflareConfig {
                web_analytics: true,
                ..CloudflareConfig::default()
            },
            ..CspConfig::default()
        };
        let policy = SitePolicy::new(&config).subresource();
        let policy = policy.to_str().unwrap();

        assert!(
            policy.contains("https://static.cloudflareinsights.com"),
            "{policy}"
        );
        assert!(
            policy.contains("connect-src 'self' https://cloudflareinsights.com"),
            "{policy}"
        );
    }

    /// A product that is off admits nothing. A preset switched on by default would widen the
    /// policy for a script this site never loads.
    #[test]
    fn a_product_that_is_off_admits_no_origin() {
        let policy = SitePolicy::new(&CspConfig::default()).subresource();
        let policy = policy.to_str().unwrap();

        assert!(!policy.contains("cloudflare.com"), "{policy}");
        assert!(!policy.contains("cloudflareinsights.com"), "{policy}");
        assert!(!policy.contains("frame-src"), "{policy}");
    }
}
