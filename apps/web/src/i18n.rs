//! Glue around the i18nrs Dioxus integration, and the site's language selection.
//!
//! The `I18nProvider` is mounted in [`crate::app::App`]; every component reads
//! the active language and translations through [`use_i18n`], which returns the
//! `I18nContext { i18n, set_language }`. Read a key with `i18n().t("key")` and
//! switch language with [`switch_language`].
//!
//! # Where the language comes from
//!
//! On the server, `portfolio_server::locale` negotiates it from the request. The server stamps
//! the outcome onto `<html lang>` of every page it sends, including pages answered from the
//! incremental cache, whose render never ran. The wasm client reads that attribute first
//! ([`detect_locale`]), so it hydrates in the language the HTML is written in whatever the
//! cookie says.

use dioxus::prelude::*;
use i18nrs::dioxus::I18nContext;
use portfolio_data::{DEFAULT_LANGUAGE, LANG_QUERY_PARAM, Language};
#[cfg(feature = "web")]
use portfolio_data::{LANG_PAIR_PREFIX, lang_cookie};

/// The i18n context provided by the `I18nProvider`.
pub fn use_i18n() -> I18nContext {
    use_context::<I18nContext>()
}

/// Switches the page to `code`: persists it as the cookie, drops a `?lang=` the address may
/// carry (it would otherwise override the new choice on the next load) and re-renders.
pub fn switch_language(ctx: &I18nContext, code: &str) {
    persist_locale(code);
    ctx.set_language.call(code.to_owned());
}

/// Server-side initial language, resolved synchronously from the request so the
/// SSR renders the correct language on the first (and only) server render.
///
/// i18nrs detects the cookie in a `use_future`, which resolves *after* the
/// synchronous `use_signal` that initializes its `I18n` — too late for SSR — so
/// its provider falls back to `default_language`, which is this.
///
/// Writes nothing: the cookie is set by the page middleware, which also sees the responses
/// the incremental cache answers without running this.
#[cfg(feature = "server")]
pub fn detect_locale() -> String {
    use dioxus_fullstack_core::FullstackContext;

    let Some(ctx) = FullstackContext::current() else {
        return DEFAULT_LANGUAGE.code.to_owned();
    };
    let parts = ctx.parts_mut();
    let header = |name: &str| parts.headers.get(name).and_then(|v| v.to_str().ok());
    portfolio_server::locale::negotiate_locale(
        parts.uri.query(),
        header("cookie"),
        header("accept-language"),
    )
    .language
    .code
    .to_owned()
}

/// Client-side initial language: the `lang` the server stamped onto `<html>`, then the `lang`
/// cookie, then the default.
///
/// `<html lang>` comes first because it is the one signal guaranteed to describe the HTML
/// being hydrated. A page answered from the incremental cache carries whatever language it
/// was negotiated for, and the cookie can be absent or stale by then.
#[cfg(all(feature = "web", not(feature = "server")))]
pub fn detect_locale() -> String {
    use web_sys::wasm_bindgen::JsCast;

    let document = web_sys::window().and_then(|w| w.document());
    let from_html = document
        .as_ref()
        .and_then(web_sys::Document::document_element)
        .and_then(|root| root.get_attribute("lang"))
        .and_then(|lang| portfolio_data::language(&lang));
    let from_cookie = || {
        document
            .as_ref()
            .and_then(|d| d.dyn_ref::<web_sys::HtmlDocument>())
            .and_then(|d| d.cookie().ok())
            .and_then(|cookie| {
                cookie
                    .split(';')
                    .map(str::trim)
                    .find_map(|pair| pair.strip_prefix(LANG_PAIR_PREFIX))
                    .and_then(portfolio_data::language)
            })
    };
    from_html
        .or_else(from_cookie)
        .unwrap_or(DEFAULT_LANGUAGE)
        .code
        .to_owned()
}

/// Persists the selected language in the `lang` cookie on the client and removes a
/// `?lang=` from the address bar, which would otherwise outrank the cookie on the next load.
#[cfg(feature = "web")]
fn persist_locale(code: &str) {
    use web_sys::wasm_bindgen::JsCast;

    let Some(window) = web_sys::window() else {
        return;
    };
    if let Some(doc) = window
        .document()
        .and_then(|d| d.dyn_into::<web_sys::HtmlDocument>().ok())
    {
        let _ = doc.set_cookie(&lang_cookie(code));
    }

    let location = window.location();
    let (Ok(path), Ok(search), Ok(hash)) =
        (location.pathname(), location.search(), location.hash())
    else {
        return;
    };
    let query = search.trim_start_matches('?');
    let kept: Vec<&str> = query
        .split('&')
        .filter(|pair| !pair.is_empty() && !pair.starts_with(LANG_PAIR_PREFIX))
        .collect();
    if kept.len() == query.split('&').filter(|p| !p.is_empty()).count() {
        return;
    }
    let url = if kept.is_empty() {
        format!("{path}{hash}")
    } else {
        format!("{path}?{}{hash}", kept.join("&"))
    };
    if let Ok(history) = window.history() {
        let _ =
            history.replace_state_with_url(&web_sys::wasm_bindgen::JsValue::NULL, "", Some(&url));
    }
}

/// No-op off the wasm client (the SSR binary and feature-less checks never touch
/// `document.cookie`), so language-switch call sites can call it unconditionally.
#[cfg(not(feature = "web"))]
fn persist_locale(_code: &str) {}

/// The absolute URL of `path` in `lang`: bare for the default language, `?lang=` otherwise.
///
/// These are the URLs the canonical link, the `hreflang` alternates and the sitemap
/// advertise, so every language is reachable — and indexable — at an address of its own.
#[must_use]
pub fn localized_url(path: &str, lang: &Language) -> String {
    let base = portfolio_data::CONFIG.url;
    match (path, lang == DEFAULT_LANGUAGE) {
        // `CONFIG.url` carries no trailing slash, so the root needs no path segment appended —
        // `https://example.com/` and `https://example.com` would otherwise be two URLs.
        ("/", true) => base.to_owned(),
        (path, true) => format!("{base}{path}"),
        (path, false) => format!("{base}{path}?{LANG_QUERY_PARAM}={}", lang.code),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_language_has_its_own_url() {
        let de = portfolio_data::language("de").unwrap();
        assert_eq!(
            localized_url("/", DEFAULT_LANGUAGE),
            portfolio_data::CONFIG.url
        );
        assert_eq!(
            localized_url("/", de),
            format!("{}/?lang=de", portfolio_data::CONFIG.url)
        );
        assert_eq!(
            localized_url("/licenses", de),
            format!("{}/licenses?lang=de", portfolio_data::CONFIG.url)
        );
        assert_eq!(
            localized_url("/licenses", DEFAULT_LANGUAGE),
            format!("{}/licenses", portfolio_data::CONFIG.url)
        );
    }
}
