//! `/legal/:slug` — one legal document, as the operator configured it.
//!
//! Which documents exist, their titles, dates and text are all configuration (`legal.*`), so this
//! page owns only the frame: the back link, the tab title and the canonical link. The document
//! itself is [`LegalDocumentContent`] from `terrace-legal-dioxus`, which renders the Markdown to
//! elements — never to an HTML string — and asks [`SiteSkin`](crate::legal::SiteSkin) for every
//! class it writes.

use dioxus::prelude::*;
use portfolio_data::CONFIG;
use terrace_legal_dioxus::LegalDocumentContent;
use terrace_legal_model::LocaleTag;

use super::NotFound;
use crate::i18n::use_i18n;
use crate::legal::{document_path, legal_document};
use crate::routes::Route;
use crate::ui::canonical::Canonical;

/// Fetches the document published under `slug` and frames it.
///
/// The fetch is a server future, so the first render — the one a crawler and a reader without
/// JavaScript receive — already carries the full text, and hydration reads it back from the
/// payload instead of asking again. Both languages arrive together (see [`crate::legal`]), so the
/// language switch re-renders without a fetch.
///
/// A slug nothing hosted is published under renders the 404 page, as any unknown URL does: an
/// external document has no page here, and a document an operator removed must not leave a
/// blank one behind.
#[component]
pub fn LegalDocument(slug: String) -> Element {
    let i18n = use_i18n().i18n;
    let key = slug.clone();
    let documents = use_server_future(use_reactive!(
        |(key,)| async move { legal_document(key).await }
    ))?;

    let lang = i18n.read().get_current_language().to_string();
    let state = documents.read();
    let view = match &*state {
        Some(Ok(Some(documents))) => documents.get(&lang).cloned(),
        // The server function cannot fail on the server; a transport failure after hydration is
        // the only way here, and it is reported the way the router reports any missing page.
        Some(Ok(None) | Err(_)) | None => None,
    };
    let Some(view) = view else {
        return rsx! {
            NotFound { segments: vec!["legal".to_owned(), slug] }
        };
    };

    let title = view
        .title
        .clone()
        .filter(|title| !title.trim().is_empty())
        .unwrap_or_else(|| slug.clone());
    let requested = lang.parse::<LocaleTag>().ok();
    let back = i18n.read().t("common.backToHome");

    rsx! {
        document::Title { "{title} · {CONFIG.full_name}" }
        // Declares its own URL rather than inheriting a site-wide canonical that would point at
        // the homepage.
        Canonical { path: document_path(&slug) }
        section { class: "legal-page",
            Link { to: Route::Home {}, class: "legal-back mono", "← {back}" }
            LegalDocumentContent { view, requested }
        }
    }
}
