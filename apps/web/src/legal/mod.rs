//! The legal documents, as `terrace-legal` serves them and `terrace-legal-dioxus` shows them.
//!
//! The documents are configuration (`legal.*`, see `portfolio-config`), so the client knows
//! nothing about them until the server says so. This module is the whole of that conversation:
//! two server functions, the value types they return, and the four adapters
//! [`LegalProvider`](terrace_legal_dioxus::LegalProvider) is mounted with.
//!
//! # Why every language at once
//!
//! Both server functions answer for every language in [`portfolio_data::LANGUAGES`] in one
//! response rather than for the language on screen. That is what keeps the site's language switch instant and the render stable:
//!
//! - The footer's links are part of the first server render, so they have to be fetched with
//!   `use_server_future`, which suspends. A fetch keyed on the language would suspend the whole
//!   shell again on every switch; one fetch with no reactive input resolves once, on the server,
//!   and hydrates from the serialized result.
//! - A document page switching language re-renders from data it already holds instead of asking
//!   the server and showing nothing meanwhile.
//!
//! The cost is the other language's text in the hydration payload — a few kilobytes of Markdown
//! on two pages, against a round-trip and a blank frame per switch.

mod adapters;

use dioxus::prelude::*;
#[cfg(feature = "server")]
use portfolio_data::LANGUAGES;
use serde::{Deserialize, Serialize};
use terrace_legal_model::{LegalDocumentView, LegalIndexEntry, LegalKind};

pub use adapters::{SiteRouting, SiteSkin, SiteText, SiteTransport};

/// A value negotiated once per site language, in [`portfolio_data::LANGUAGES`] order.
///
/// A `Vec` of pairs rather than a map: there are two languages, the order is the site's, and the
/// type crosses the server-function boundary where a plain sequence is the least to agree on.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerLanguage<T>(pub Vec<(String, T)>);

impl<T> Default for PerLanguage<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<T> PerLanguage<T> {
    /// The value for `lang`, or the first language's when `lang` is not a site language.
    ///
    /// The fallback is unreachable through the interface, which only ever holds a
    /// [`portfolio_data::LANGUAGES`] entry; it exists so that a stale cookie cannot turn into an
    /// empty page.
    pub fn get(&self, lang: &str) -> Option<&T> {
        self.0
            .iter()
            .find(|(candidate, _)| candidate == lang)
            .or_else(|| self.0.first())
            .map(|(_, value)| value)
    }
}

/// The published documents, titled and ordered for each site language.
pub type LegalIndexes = PerLanguage<Vec<LegalIndexEntry>>;

/// One hosted document, negotiated for each site language.
pub type LegalDocuments = PerLanguage<LegalDocumentView>;

/// Every published document's index entry, for each site language.
///
/// # Errors
///
/// The transport's own, and — on the server — a call that arrives before the catalog loaded at
/// boot was installed, which `serve` rules out by installing it before the listener exists.
#[expect(
    clippy::unused_async,
    reason = "a server function is async by contract"
)]
#[server]
pub async fn legal_indexes() -> Result<LegalIndexes, ServerFnError> {
    crate::server::legal::indexes().ok_or_else(not_loaded)
}

/// The document published under `slug`, for each site language, or `None` when nothing hosted
/// is published under it.
///
/// An external document is `None` too: it has no body to show, and its link points away from
/// this site in the first place.
///
/// # Errors
///
/// As [`legal_indexes`].
#[expect(
    clippy::unused_async,
    reason = "a server function is async by contract"
)]
#[server]
pub async fn legal_document(slug: String) -> Result<Option<LegalDocuments>, ServerFnError> {
    if crate::server::legal::indexes().is_none() {
        return Err(not_loaded());
    }
    Ok(crate::server::legal::documents(&slug))
}

/// The error for a server function reached before the catalog was installed.
#[cfg(feature = "server")]
fn not_loaded() -> ServerFnError {
    ServerFnError::new("the legal documents are not loaded")
}

/// The index in the language on screen, from the context the shell provides.
///
/// Empty when no provider is mounted, which only a component rendered outside the shell sees.
pub fn use_legal_entries(lang: &str) -> Vec<LegalIndexEntry> {
    try_consume_context::<LegalIndexes>()
        .and_then(|indexes| indexes.get(lang).cloned())
        .unwrap_or_default()
}

/// The route path of a hosted document.
///
/// One definition for the router, the sitemap, the render cache's allowlist and the redirects
/// from the pre-configuration paths, which must all agree on it.
pub fn document_path(slug: &str) -> String {
    format!("/legal/{slug}")
}

/// Whether `entry` is served by this site rather than linked.
pub fn is_hosted(entry: &LegalIndexEntry) -> bool {
    entry.kind == LegalKind::Inline
}

/// Negotiates `value` for every site language.
///
/// Shared by both server functions so the language list and its order live in one place.
#[cfg(feature = "server")]
pub(crate) fn per_language<T>(mut value: impl FnMut(&str) -> Option<T>) -> Option<PerLanguage<T>> {
    let mut pairs = Vec::with_capacity(LANGUAGES.len());
    for lang in LANGUAGES {
        pairs.push((lang.to_owned(), value(lang)?));
    }
    Some(PerLanguage(pairs))
}
