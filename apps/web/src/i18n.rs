//! Glue around the i18nrs Dioxus integration.
//!
//! The `I18nProvider` is mounted in [`crate::app::App`]; every component reads
//! the active language and translations through [`use_i18n`], which returns the
//! `I18nContext { i18n, set_language }`. Read a key with `i18n().t("key")` and
//! switch language with `set_language.call("de".into())`.

use dioxus::prelude::*;
use i18nrs::dioxus::I18nContext;

/// Storage key holding the selected language ("en" / "de"). Documented in the
/// privacy policy — keep both in sync.
pub const LANG_STORAGE_KEY: &str = "lang";

/// The i18n context provided by the `I18nProvider`.
pub fn use_i18n() -> I18nContext {
    use_context::<I18nContext>()
}

/// The language to switch to from `current`.
pub fn other_language(current: &str) -> &'static str {
    if current == "de" { "en" } else { "de" }
}

/// Server-side initial language, resolved synchronously from the request so the
/// SSR renders the correct language on the first (and only) server render.
///
/// i18nrs detects the cookie in a `use_future`, which resolves *after* the
/// synchronous `use_signal` that initializes its `I18n` — too late for SSR — so
/// its provider falls back to `default_language`. We compute that fallback here:
/// a valid `lang` cookie wins; otherwise Accept-Language is honored and written
/// back as the cookie so the client hydrates with the same language.
#[cfg(feature = "server")]
pub fn detect_locale() -> String {
    use dioxus_fullstack_core::FullstackContext;

    let Some(ctx) = FullstackContext::current() else {
        return "en".to_string();
    };

    let (cookie, accept_language) = {
        let parts = ctx.parts_mut();
        let header = |name: &str| {
            parts
                .headers
                .get(name)
                .and_then(|v| v.to_str().ok())
                .map(str::to_owned)
        };
        (header("cookie"), header("accept-language"))
    };

    // A valid `lang` cookie is authoritative and needs no write-back.
    if let Some(lang) = cookie_locale(cookie.as_deref()) {
        return lang;
    }

    // Otherwise honor Accept-Language and persist the choice so the client reads
    // the same value from `document.cookie` during hydration (no mismatch).
    let accept_lang = accept_language_locale(accept_language.as_deref());
    if let Ok(value) = http::HeaderValue::from_str(&format!(
        "{LANG_STORAGE_KEY}={accept_lang}; Path=/; Max-Age=31536000; SameSite=Lax"
    )) {
        ctx.add_response_header(http::header::SET_COOKIE, value);
    }
    accept_lang.to_string()
}

/// Negotiates the request locale from raw headers *without any side effects*: a
/// valid `lang` cookie wins, otherwise `Accept-Language` decides (German when it
/// starts with `de`, English otherwise).
///
/// This is the same decision [`detect_locale`] makes, factored out so the
/// server's ISR cache-key middleware can reproduce it from an Axum request. The
/// cached entry and the HTML rendered into it must agree on the language, so
/// both paths route through this one function.
#[cfg(feature = "server")]
pub fn negotiate_locale(cookie_header: Option<&str>, accept_language: Option<&str>) -> String {
    cookie_locale(cookie_header)
        .unwrap_or_else(|| accept_language_locale(accept_language).to_string())
}

/// The locale named by a valid `lang` cookie in `cookie_header`, if present.
/// Unknown languages are ignored so the caller falls back to negotiation.
#[cfg(feature = "server")]
fn cookie_locale(cookie_header: Option<&str>) -> Option<String> {
    use portfolio_data::LANGUAGES;

    cookie_header?
        .split(';')
        .map(str::trim)
        .find_map(|c| c.strip_prefix(&format!("{LANG_STORAGE_KEY}=")))
        .map(str::to_owned)
        .filter(|lang| LANGUAGES.iter().any(|l| *l == lang))
}

/// The locale implied by the `Accept-Language` header's primary tag: German when
/// it starts with `de`, English (the site default) otherwise.
#[cfg(feature = "server")]
fn accept_language_locale(accept_language: Option<&str>) -> &'static str {
    let primary = accept_language
        .and_then(|al| al.split(',').next())
        .unwrap_or("")
        .trim()
        .to_lowercase();
    if primary.starts_with("de") {
        "de"
    } else {
        "en"
    }
}

/// Client-side initial language, read synchronously from the `lang` cookie the
/// server set during negotiation. Because the SSR HTML was rendered with the
/// same value (see the `server` variant above), the wasm client hydrates with
/// the matching language — no i18nrs `get_cookie` server round-trip required.
#[cfg(all(feature = "web", not(feature = "server")))]
pub fn detect_locale() -> String {
    use portfolio_data::LANGUAGES;
    use web_sys::wasm_bindgen::JsCast;

    let cookie_lang = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.dyn_into::<web_sys::HtmlDocument>().ok())
        .and_then(|d| d.cookie().ok())
        .and_then(|c| {
            c.split(';')
                .map(str::trim)
                .find_map(|c| c.strip_prefix(&format!("{LANG_STORAGE_KEY}=")))
                .map(str::to_owned)
        });

    match cookie_lang {
        Some(lang) if LANGUAGES.iter().any(|l| *l == lang) => lang,
        _ => "en".to_string(),
    }
}

/// Persists the selected language in the `lang` cookie on the client, replacing
/// i18nrs's `set_cookie` server function. Call it on every language switch so a
/// subsequent full page load reads the same value from the request cookie and
/// the SSR renders the chosen language.
#[cfg(feature = "web")]
pub fn persist_locale(lang: &str) {
    use web_sys::wasm_bindgen::JsCast;

    if let Some(doc) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.dyn_into::<web_sys::HtmlDocument>().ok())
    {
        let _ = doc.set_cookie(&format!(
            "{LANG_STORAGE_KEY}={lang}; Path=/; Max-Age=31536000; SameSite=Lax"
        ));
    }
}

/// No-op off the wasm client (the SSR binary and feature-less checks never touch
/// `document.cookie`), so language-switch call sites can call it unconditionally.
#[cfg(not(feature = "web"))]
pub fn persist_locale(_lang: &str) {}

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::*;

    #[test]
    fn a_valid_lang_cookie_wins_over_accept_language() {
        assert_eq!(negotiate_locale(Some("lang=de"), Some("en-US,en")), "de");
        assert_eq!(negotiate_locale(Some("lang=en"), Some("de-DE,de")), "en");
    }

    #[test]
    fn the_lang_cookie_is_found_among_others() {
        assert_eq!(
            negotiate_locale(Some("theme=dark; lang=de; consent=1"), None),
            "de"
        );
    }

    #[test]
    fn an_unknown_cookie_language_falls_through_to_accept_language() {
        // `fr` is not a supported locale, so negotiation ignores the cookie.
        assert_eq!(negotiate_locale(Some("lang=fr"), Some("de-DE")), "de");
        assert_eq!(negotiate_locale(Some("lang=fr"), Some("en")), "en");
    }

    #[test]
    fn accept_language_decides_when_no_cookie_is_present() {
        assert_eq!(negotiate_locale(None, Some("de-DE,de;q=0.9")), "de");
        assert_eq!(negotiate_locale(None, Some("de")), "de");
        // Anything that is not German (or absent) defaults to English.
        assert_eq!(negotiate_locale(None, Some("fr-FR,fr")), "en");
        assert_eq!(negotiate_locale(None, None), "en");
    }
}
