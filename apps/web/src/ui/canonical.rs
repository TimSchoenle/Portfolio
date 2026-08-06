//! Per-route `rel="canonical"` link.
//!
//! Every page must declare *itself* as canonical. A single site-wide canonical
//! rendered from the shared head would point `/imprint` and `/privacy` at the
//! homepage, which tells search engines to fold those pages into `/` and drop
//! them — while `sitemap.xml` simultaneously submits all three. The two signals
//! have to agree, so the URL is built per route here.

use dioxus::prelude::*;
use portfolio_data::CONFIG;

/// Renders `<link rel="canonical">` for `path`, a route path such as `/` or
/// `/imprint`. The absolute URL is built from [`CONFIG`]`.url` so the canonical
/// host stays the configured one even when the page is reached through another
/// hostname.
#[component]
pub fn Canonical(path: &'static str) -> Element {
    // `CONFIG.url` carries no trailing slash, so the root needs no path segment
    // appended — `https://example.com/` and `https://example.com` would
    // otherwise be advertised as two different canonical URLs.
    let href = if path == "/" {
        CONFIG.url.to_string()
    } else {
        format!("{}{path}", CONFIG.url)
    };

    rsx! {
        document::Link { rel: "canonical", href: "{href}" }
    }
}
