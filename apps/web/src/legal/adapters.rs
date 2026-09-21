//! The four things [`terrace_legal_dioxus::LegalProvider`] needs from this site.

use dioxus::prelude::*;
use i18nrs::I18n;
use terrace_legal_dioxus::{LegalRouting, LegalSkin, LegalText, LegalTransport, LocalFuture, Part};
use terrace_legal_model::{LegalDocumentView, LegalIndexEntry, LocaleTag};

use super::{LegalIndexes, legal_document};
use crate::routes::Route;

/// Fetches documents through this site's server functions.
///
/// The index is answered from the [`LegalIndexes`] the shell already resolved for the first
/// render, so the provider's own index fetch costs no request. A document is fetched through
/// [`legal_document`], which is what the page itself uses too.
pub struct SiteTransport {
    indexes: LegalIndexes,
}

impl SiteTransport {
    /// Serves the index out of `indexes`.
    pub fn new(indexes: LegalIndexes) -> Self {
        Self { indexes }
    }
}

impl LegalTransport for SiteTransport {
    type Error = String;

    fn index(&self, lang: &str) -> LocalFuture<Result<Vec<LegalIndexEntry>, String>> {
        let entries = self.indexes.get(lang).cloned().unwrap_or_default();
        Box::pin(async move { Ok(entries) })
    }

    fn document(&self, slug: &str, lang: &str) -> LocalFuture<Result<LegalDocumentView, String>> {
        let slug = slug.to_owned();
        let lang = lang.to_owned();
        Box::pin(async move {
            let documents = legal_document(slug.clone())
                .await
                .map_err(|error| error.to_string())?
                .ok_or_else(|| format!("no document is published under `{slug}`"))?;
            documents
                .get(&lang)
                .cloned()
                .ok_or_else(|| format!("`{slug}` has no text for `{lang}`"))
        })
    }
}

/// The components' words, from the site's translation files.
pub struct SiteText {
    i18n: Signal<I18n>,
}

impl SiteText {
    /// Reads translations through `i18n`, so a language switch is picked up on the next render.
    pub fn new(i18n: Signal<I18n>) -> Self {
        Self { i18n }
    }

    fn t(&self, key: &str) -> String {
        self.i18n.read().t(key)
    }
}

impl LegalText for SiteText {
    /// Every document this site requires carries a configured title in both languages, so this
    /// is reached only for an optional document an operator left untitled — which then reads as
    /// its slug rather than as a name this site made up for it.
    fn known_title(&self, _slug: &str) -> Option<String> {
        None
    }

    fn heading(&self) -> String {
        self.t("legal.heading")
    }

    fn updated(&self, date: &str) -> String {
        format!("{}: {date}", self.t("common.lastUpdated"))
    }

    fn shown_in(&self, locale: &LocaleTag) -> String {
        format!(
            "{}: {}",
            self.t("legal.shownIn"),
            locale.as_str().to_uppercase()
        )
    }
}

/// Links a hosted document through the site's router.
pub struct SiteRouting;

impl LegalRouting for SiteRouting {
    fn document_link(&self, slug: &str, label: String, class: &'static str) -> Element {
        rsx! {
            Link {
                to: Route::LegalDocument { slug: slug.to_owned() },
                class,
                "{label}"
            }
        }
    }
}

/// The site's classes for each part a component renders. They are defined in
/// `assets/input.css`, next to the rest of the page styles.
pub struct SiteSkin;

impl LegalSkin for SiteSkin {
    fn class(&self, part: Part) -> &'static str {
        match part {
            Part::Page => "legal-document",
            Part::Meta | Part::LocaleNote => "legal-meta mono text-muted",
            Part::Prose => "legal-prose",
            // Links inherit the styling of the list they sit in: the footer column and the
            // palette each style their own anchors.
            _ => "",
        }
    }
}
