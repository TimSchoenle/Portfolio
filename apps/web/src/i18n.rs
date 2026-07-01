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
    use portfolio_data::LANGUAGES;

    let Some(ctx) = FullstackContext::current() else {
        return "en".to_string();
    };

    let cookie_lang = {
        let parts = ctx.parts_mut();
        parts
            .headers
            .get("cookie")
            .and_then(|v| v.to_str().ok())
            .and_then(|c| {
                c.split(';')
                    .map(str::trim)
                    .find_map(|c| c.strip_prefix(&format!("{LANG_STORAGE_KEY}=")))
                    .map(str::to_owned)
            })
    };
    if let Some(lang) = cookie_lang
        && LANGUAGES.iter().any(|l| *l == lang)
    {
        return lang;
    }

    let accept_lang = {
        let parts = ctx.parts_mut();
        let primary = parts
            .headers
            .get("accept-language")
            .and_then(|v| v.to_str().ok())
            .and_then(|al| al.split(',').next())
            .unwrap_or("")
            .trim()
            .to_lowercase();
        if primary.starts_with("de") {
            "de"
        } else {
            "en"
        }
    };

    // Persist the negotiated language so the client reads the same value from
    // `document.cookie` during hydration (no server/client mismatch).
    if let Ok(value) = http::HeaderValue::from_str(&format!(
        "{LANG_STORAGE_KEY}={accept_lang}; Path=/; Max-Age=31536000; SameSite=Lax"
    )) {
        ctx.add_response_header(http::header::SET_COOKIE, value);
    }
    accept_lang.to_string()
}

/// Client-side initial language, read synchronously from the `lang` cookie the
/// server set during negotiation. Because the SSR HTML was rendered with the
/// same value (see the `server` variant above), the wasm client hydrates with
/// the matching language — no i18nrs `get_cookie` server round-trip required.
#[cfg(feature = "web")]
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
