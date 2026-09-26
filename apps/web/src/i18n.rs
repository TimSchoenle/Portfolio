//! Glue around the i18nrs Dioxus integration, and the site's language negotiation.
//!
//! The `I18nProvider` is mounted in [`crate::app::App`]; every component reads
//! the active language and translations through [`use_i18n`], which returns the
//! `I18nContext { i18n, set_language }`. Read a key with `i18n().t("key")` and
//! switch language with [`switch_language`].
//!
//! # Where the language comes from
//!
//! On the server, `negotiate_locale` decides, in this order: a `?lang=` query parameter, a
//! valid `lang` cookie, the `Accept-Language` header, the default language. The query
//! parameter is what gives every language a URL of its own, which is what a crawler — it
//! sends neither a cookie nor, usually, an `Accept-Language` — needs in order to index it.
//!
//! The server stamps the outcome onto `<html lang>` of every page it sends, including pages
//! answered from the incremental cache, whose render never ran. The wasm client reads that
//! attribute first ([`detect_locale`]), so it hydrates in the language the HTML is written in
//! whatever the cookie says.

use dioxus::prelude::*;
use i18nrs::dioxus::I18nContext;
use portfolio_data::{DEFAULT_LANGUAGE, Language};

/// Storage key holding the selected language ("en" / "de"). Documented in the
/// privacy notice (the template in `legal/privacy.toml`, and the published text a
/// deployment mounts in its place) — keep all of them in sync.
pub const LANG_STORAGE_KEY: &str = "lang";

/// The query parameter naming a language explicitly: `/?lang=de`.
///
/// The same name as the cookie on purpose: both carry a [`Language::code`].
pub const LANG_QUERY_PARAM: &str = LANG_STORAGE_KEY;

/// [`LANG_STORAGE_KEY`] with the `=` that separates a cookie pair's name from
/// its value, as a constant rather than a `format!` at each use site.
///
/// Both cookie readers below scan every pair in the header looking for this
/// needle, and building it inside that loop allocated a `String` per pair.
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

/// The `Set-Cookie` value persisting `code` for a year.
///
/// `SameSite=Lax` and no `Secure`: the value is a UI preference with no authority, and a
/// development server on plain HTTP has to be able to set it too.
#[must_use]
pub fn lang_cookie(code: &str) -> String {
    format!("{LANG_COOKIE_PREFIX}{code}; Path=/; Max-Age=31536000; SameSite=Lax")
}

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

/// What [`negotiate_locale`] decided, and whether the response has to write it back.
#[cfg(feature = "server")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Negotiated {
    /// The language the page is rendered in.
    pub language: &'static Language,
    /// Whether the request's `lang` cookie is absent or names something else, so the response
    /// must set it — otherwise the client could read a cookie that disagrees with the HTML.
    pub persist: bool,
}

/// Negotiates the request's language from its raw query string and headers, *without side
/// effects*: `?lang=` wins, then a valid `lang` cookie, then `Accept-Language`, then the
/// default.
///
/// This is the one decision every server-side reader shares — the render
/// ([`detect_locale`]), the page middleware that stamps `<html lang>`, keys the incremental
/// cache and writes the cookie back — so the language of the HTML, its cache entry and the
/// cookie always agree.
#[cfg(feature = "server")]
#[must_use]
pub fn negotiate_locale(
    query: Option<&str>,
    cookie_header: Option<&str>,
    accept_language: Option<&str>,
) -> Negotiated {
    let cookie = cookie_locale(cookie_header);
    let language = query_locale(query)
        .or(cookie)
        .unwrap_or_else(|| accept_language_locale(accept_language));
    Negotiated {
        language,
        persist: cookie != Some(language),
    }
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
    negotiate_locale(
        parts.uri.query(),
        header("cookie"),
        header("accept-language"),
    )
    .language
    .code
    .to_owned()
}

/// The language a `lang=` pair in `query` names, if it names a site language.
#[cfg(feature = "server")]
fn query_locale(query: Option<&str>) -> Option<&'static Language> {
    query?
        .split('&')
        .filter_map(|pair| pair.strip_prefix(LANG_COOKIE_PREFIX))
        .find_map(portfolio_data::language)
}

/// The language named by a valid `lang` cookie in `cookie_header`, if present.
/// Unknown languages are ignored so the caller falls back to negotiation.
#[cfg(feature = "server")]
fn cookie_locale(cookie_header: Option<&str>) -> Option<&'static Language> {
    let value = cookie_header?
        .split(';')
        .map(str::trim)
        .find_map(|pair| pair.strip_prefix(LANG_COOKIE_PREFIX))?;
    portfolio_data::language(value)
}

/// The site language `Accept-Language` prefers most, or the default.
///
/// Honors quality values (RFC 9110 §12.5.4): the highest-weighted range whose primary subtag
/// is a site language wins, earlier ranges win ties, and `q=0` excludes a range. `*` and
/// malformed entries are ignored rather than guessed at.
#[cfg(feature = "server")]
fn accept_language_locale(accept_language: Option<&str>) -> &'static Language {
    let mut best: Option<(&'static Language, u16)> = None;
    for range in accept_language.unwrap_or_default().split(',') {
        let mut params = range.split(';');
        let tag = params.next().unwrap_or_default().trim();
        let primary = tag.split('-').next().unwrap_or_default();
        let Some(candidate) = portfolio_data::SITE_LANGUAGES
            .iter()
            .find(|lang| lang.code.eq_ignore_ascii_case(primary))
        else {
            continue;
        };
        let Some(weight) = params
            .find_map(|param| param.trim().strip_prefix("q="))
            .map_or(Some(1000), parse_qvalue)
        else {
            continue;
        };
        if weight > 0 && best.is_none_or(|(_, top)| weight > top) {
            best = Some((candidate, weight));
        }
    }
    best.map_or(DEFAULT_LANGUAGE, |(lang, _)| lang)
}

/// A `qvalue` in thousandths (`"0.8"` is 800), or `None` when it is not one.
#[cfg(feature = "server")]
fn parse_qvalue(raw: &str) -> Option<u16> {
    let (whole, fraction) = raw.split_once('.').unwrap_or((raw, ""));
    if fraction.len() > 3 || !fraction.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let digits = |s: &str| s.bytes().fold(0u16, |n, b| n * 10 + u16::from(b - b'0'));
    let milli = match whole {
        "0" => digits(&format!("{fraction:0<3}")),
        "1" if fraction.bytes().all(|b| b == b'0') => 1000,
        _ => return None,
    };
    Some(milli)
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
                    .find_map(|pair| pair.strip_prefix(LANG_COOKIE_PREFIX))
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
        .filter(|pair| !pair.is_empty() && !pair.starts_with(LANG_COOKIE_PREFIX))
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

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::*;

    fn code(query: Option<&str>, cookie: Option<&str>, accept: Option<&str>) -> &'static str {
        negotiate_locale(query, cookie, accept).language.code
    }

    #[test]
    fn a_valid_lang_cookie_wins_over_accept_language() {
        assert_eq!(code(None, Some("lang=de"), Some("en-US,en")), "de");
        assert_eq!(code(None, Some("lang=en"), Some("de-DE,de")), "en");
    }

    #[test]
    fn the_query_parameter_wins_over_everything() {
        assert_eq!(code(Some("lang=de"), Some("lang=en"), Some("en")), "de");
        assert_eq!(
            code(Some("ref=x&lang=en"), Some("lang=de"), Some("de")),
            "en"
        );
        // An unknown value is ignored rather than trusted.
        assert_eq!(code(Some("lang=fr"), Some("lang=de"), None), "de");
    }

    #[test]
    fn the_lang_cookie_is_found_among_others() {
        assert_eq!(
            code(None, Some("theme=dark; lang=de; consent=1"), None),
            "de"
        );
    }

    #[test]
    fn an_unknown_cookie_language_falls_through_to_accept_language() {
        assert_eq!(code(None, Some("lang=fr"), Some("de-DE")), "de");
        assert_eq!(code(None, Some("lang=fr"), Some("en")), "en");
    }

    #[test]
    fn accept_language_decides_when_no_cookie_is_present() {
        assert_eq!(code(None, None, Some("de-DE,de;q=0.9")), "de");
        assert_eq!(code(None, None, Some("de")), "de");
        assert_eq!(code(None, None, Some("fr-FR,fr")), "en");
        assert_eq!(code(None, None, None), "en");
    }

    #[test]
    fn accept_language_honors_quality_values() {
        assert_eq!(code(None, None, Some("en;q=0.1, de;q=0.9")), "de");
        assert_eq!(code(None, None, Some("fr, de;q=0.5, en;q=0.4")), "de");
        // Ties go to the earlier range; `q=0` rules a range out.
        assert_eq!(code(None, None, Some("de;q=0.5, en;q=0.5")), "de");
        assert_eq!(code(None, None, Some("DE;q=0, en;q=0.2")), "en");
        // Malformed weights are skipped, not guessed at.
        assert_eq!(code(None, None, Some("de;q=2, en;q=0.3")), "en");
        assert_eq!(code(None, None, Some("de;q=abc")), "en");
    }

    #[test]
    fn the_cookie_is_written_back_only_when_it_disagrees() {
        assert!(!negotiate_locale(None, Some("lang=de"), None).persist);
        assert!(negotiate_locale(None, None, Some("de")).persist);
        assert!(negotiate_locale(Some("lang=en"), Some("lang=de"), None).persist);
        assert!(negotiate_locale(None, Some("lang=fr"), None).persist);
    }

    #[test]
    fn qvalues_parse_to_thousandths() {
        assert_eq!(parse_qvalue("1"), Some(1000));
        assert_eq!(parse_qvalue("1.000"), Some(1000));
        assert_eq!(parse_qvalue("0.8"), Some(800));
        assert_eq!(parse_qvalue("0.125"), Some(125));
        assert_eq!(parse_qvalue("0"), Some(0));
        for bad in ["1.5", "2", "0.1234", "", "x", "-0.5"] {
            assert_eq!(parse_qvalue(bad), None, "{bad}");
        }
    }

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
