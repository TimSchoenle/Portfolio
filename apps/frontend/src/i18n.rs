//! Glue between i18nrs and the rest of the app.

use portfolio_data::LANGUAGES;

/// localStorage key holding the selected language ("en" / "de").
/// Documented in the privacy policy — keep both in sync.
pub const LANG_STORAGE_KEY: &str = "lang";

/// Seeds the language storage from the browser language before the
/// `I18nProvider` mounts.
///
/// i18nrs only applies `default_language` on non-WASM targets; on WASM an
/// empty storage entry would leave the active language at an arbitrary
/// `HashMap` key. Writing a valid value up front sidesteps that and gives us
/// browser-language detection on first visit.
pub fn ensure_language_seeded() {
    let Some(win) = web_sys::window() else { return };
    let Ok(Some(storage)) = win.local_storage() else {
        return;
    };
    let valid = storage
        .get_item(LANG_STORAGE_KEY)
        .ok()
        .flatten()
        .is_some_and(|l| LANGUAGES.contains(&l.as_str()));
    if !valid {
        let preferred = win
            .navigator()
            .language()
            .unwrap_or_default()
            .to_lowercase();
        let lang = if preferred.starts_with("de") {
            "de"
        } else {
            LANGUAGES[0]
        };
        let _ = storage.set_item(LANG_STORAGE_KEY, lang);
    }
}

/// The language to switch to from `current`.
pub fn other_language(current: &str) -> &'static str {
    if current == "de" { "en" } else { "de" }
}

/// Mirrors the active language onto `<html lang>` for a11y and SEO.
pub fn set_document_lang(lang: &str) {
    if let Some(el) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.document_element())
    {
        let _ = el.set_attribute("lang", lang);
    }
}

/// Sets the document title (per page, localized).
pub fn set_document_title(title: &str) {
    if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
        doc.set_title(title);
    }
}
