//! Per-route `rel="canonical"` link.
//!
//! Every page must declare *itself* as canonical. A single site-wide canonical
//! rendered from the shared head would point `/licenses` and every `/legal/…` page at the
//! homepage, which tells search engines to fold those pages into `/` and drop
//! them — while `sitemap.xml` simultaneously submits all three. The two signals
//! have to agree, so the URL is built per route here.
//!
//! The same component declares the page's language variants: one
//! `<link rel="alternate" hreflang>` per site language plus `x-default`, each pointing at the
//! address [`localized_url`] gives that language. Without them a crawler, which negotiates no
//! language, only ever finds the default one.

use dioxus::prelude::*;
use portfolio_data::{DEFAULT_LANGUAGE, SITE_LANGUAGES, language_or_default};

use crate::i18n::{localized_url, use_i18n};

/// Renders `<link rel="canonical">` and the `hreflang` alternates for `path`, a route path
/// such as `/` or `/legal/imprint`, in the page's current language. Every URL is absolute and
/// built from `CONFIG.url`, so the canonical host stays the configured one even when the page
/// is reached through another hostname.
#[component]
pub fn Canonical(#[props(into)] path: String) -> Element {
    let i18n = use_i18n().i18n;
    let current = language_or_default(i18n.read().get_current_language());
    // (rel, hreflang, href): the alternates for every language, `x-default`, then the canonical
    // for the language being shown.
    let mut links: Vec<(&'static str, Option<&'static str>, String)> = SITE_LANGUAGES
        .iter()
        .map(|language| {
            (
                "alternate",
                Some(language.code),
                localized_url(&path, language),
            )
        })
        .collect();
    // `Alternate`, capitalized, on purpose: `document::Link` deduplicates on `href` and `rel`,
    // and `x-default` shares its URL with the default language's alternate, so a second
    // lowercase `alternate` for it would be dropped. `rel` values are ASCII case-insensitive in
    // HTML, so browsers and crawlers read both the same.
    links.push((
        "Alternate",
        Some("x-default"),
        localized_url(&path, DEFAULT_LANGUAGE),
    ));
    links.push(("canonical", None, localized_url(&path, current)));

    rsx! {
        // Keyed by the whole tag because `document::Link` writes its tag once, on mount. A route
        // that keeps its component across a navigation — `/legal/:slug` linking to another
        // document, or a language switch — re-renders this with new URLs, and without the key
        // the head would keep announcing the previous ones. A new key replaces the element.
        for (rel, hreflang, href) in links {
            document::Link {
                key: "{rel}:{hreflang:?}:{href}",
                rel,
                hreflang: hreflang.map(str::to_owned),
                href,
            }
        }
    }
}
