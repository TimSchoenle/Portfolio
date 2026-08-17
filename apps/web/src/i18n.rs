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

/// [`LANG_STORAGE_KEY`] with the `=` that separates a cookie pair's name from
/// its value, as a constant rather than a `format!` at each use site.
///
/// Both cookie readers below scan every pair in the header looking for this
/// needle, and building it inside that loop allocated a `String` per pair, on
/// every request — the cookie header is read for each page navigation, so the
/// cost was proportional to how many cookies a visitor happened to carry.
const LANG_COOKIE_PREFIX: &str = "lang=";

/// Compile-time proof that the two constants above have not drifted apart. They
/// are separate literals because `concat!` takes only literals, so nothing but
/// this check stops a rename of one from silently leaving the other behind —
/// which would read the cookie under a name nothing ever writes.
const _: () = {
    let key = LANG_STORAGE_KEY.as_bytes();
    let prefix = LANG_COOKIE_PREFIX.as_bytes();
    assert!(prefix.len() == key.len() + 1);
    assert!(prefix[key.len()] == b'=');
    let mut i = 0;
    while i < key.len() {
        assert!(prefix[i] == key[i]);
        i += 1;
    }
};

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
        return accept_language_locale(None).to_string();
    };

    // `parts_mut` borrows the context, and the write-back below borrows it
    // again, so the decision is made — and that borrow released — inside this
    // block. Only the negotiated `&'static str` crosses out of it, which is why
    // the header values no longer have to be copied to outlive it.
    let (locale, from_cookie) = {
        let parts = ctx.parts_mut();
        let header = |name: &str| parts.headers.get(name).and_then(|v| v.to_str().ok());
        match cookie_locale(header("cookie")) {
            Some(lang) => (lang, true),
            None => (accept_language_locale(header("accept-language")), false),
        }
    };

    // A valid `lang` cookie is authoritative and needs no write-back. Otherwise
    // persist what Accept-Language chose, so the client reads the same value
    // from `document.cookie` during hydration (no mismatch).
    if !from_cookie
        && let Ok(value) = http::HeaderValue::from_str(&format!(
            "{LANG_COOKIE_PREFIX}{locale}; Path=/; Max-Age=31536000; SameSite=Lax"
        ))
    {
        ctx.add_response_header(http::header::SET_COOKIE, value);
    }
    locale.to_string()
}

/// Negotiates the request locale from raw headers *without any side effects*: a
/// valid `lang` cookie wins, otherwise `Accept-Language` decides (German when it
/// starts with `de`, English otherwise).
///
/// This is the same decision [`detect_locale`] makes, factored out so the
/// server's ISR cache-key middleware can reproduce it from an Axum request. The
/// cached entry and the HTML rendered into it must agree on the language, so
/// both paths route through this one function.
///
/// The result is a `&'static str` borrowed from [`LANGUAGES`], not an owned
/// `String`: this runs on every request the server answers, and the language is
/// always one of two compile-time constants, so there is nothing to allocate.
///
/// [`LANGUAGES`]: portfolio_data::LANGUAGES
#[cfg(feature = "server")]
pub fn negotiate_locale(
    cookie_header: Option<&str>,
    accept_language: Option<&str>,
) -> &'static str {
    cookie_locale(cookie_header).unwrap_or_else(|| accept_language_locale(accept_language))
}

/// The locale named by a valid `lang` cookie in `cookie_header`, if present.
/// Unknown languages are ignored so the caller falls back to negotiation.
#[cfg(feature = "server")]
fn cookie_locale(cookie_header: Option<&str>) -> Option<&'static str> {
    use portfolio_data::LANGUAGES;

    let value = cookie_header?
        .split(';')
        .map(str::trim)
        .find_map(|pair| pair.strip_prefix(LANG_COOKIE_PREFIX))?;
    // Returning the matching `LANGUAGES` entry rather than the header's own
    // slice is what makes the `&'static str` sound, and it restates in the type
    // that only a supported language ever leaves this function.
    LANGUAGES.into_iter().find(|lang| *lang == value)
}

/// The locale implied by the `Accept-Language` header's primary tag: German when
/// it starts with `de`, English (the site default) otherwise.
#[cfg(feature = "server")]
fn accept_language_locale(accept_language: Option<&str>) -> &'static str {
    let primary = accept_language
        .and_then(|al| al.split(',').next())
        .unwrap_or("")
        .trim();
    // Language tags are ASCII (BCP 47), so an ASCII-insensitive compare of the
    // primary subtag's first two bytes decides this without lowercasing the
    // whole tag into a fresh `String` on every request. `get` rather than a
    // slice index: a malformed header may put a multi-byte character there, and
    // that must answer "not German", not panic.
    match primary.get(..2) {
        Some(tag) if tag.eq_ignore_ascii_case("de") => "de",
        _ => "en",
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
                .find_map(|pair| pair.strip_prefix(LANG_COOKIE_PREFIX))
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
            "{LANG_COOKIE_PREFIX}{lang}; Path=/; Max-Age=31536000; SameSite=Lax"
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
