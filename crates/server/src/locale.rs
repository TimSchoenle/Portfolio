//! Negotiating the language of a page request.
//!
//! The order is a `?lang=` query parameter, a valid `lang` cookie, the `Accept-Language` header,
//! then the default language. The query parameter gives every language a URL of its own, which a
//! crawler needs in order to index it: a crawler sends no cookie and usually no
//! `Accept-Language`.

use portfolio_data::{DEFAULT_LANGUAGE, LANG_PAIR_PREFIX, Language, SITE_LANGUAGES, language};

/// What [`negotiate_locale`] decided, and whether the response has to write it back.
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
/// This is the one decision every server-side reader shares: the render, and the page
/// middleware ([`crate::page::localize_page`]) that stamps `<html lang>`, keys the incremental
/// cache and writes the cookie back. Sharing it is what keeps the language of the HTML, its
/// cache entry and the cookie in agreement.
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

/// The language a `lang=` pair in `query` names, if it names a site language.
fn query_locale(query: Option<&str>) -> Option<&'static Language> {
    query?
        .split('&')
        .filter_map(|pair| pair.strip_prefix(LANG_PAIR_PREFIX))
        .find_map(language)
}

/// The language named by a valid `lang` cookie in `cookie_header`, if present.
/// Unknown languages are ignored so the caller falls back to negotiation.
fn cookie_locale(cookie_header: Option<&str>) -> Option<&'static Language> {
    let value = cookie_header?
        .split(';')
        .map(str::trim)
        .find_map(|pair| pair.strip_prefix(LANG_PAIR_PREFIX))?;
    language(value)
}

/// The site language `Accept-Language` prefers most, or the default.
///
/// Honors quality values (RFC 9110 §12.5.4): the highest-weighted range whose primary subtag
/// is a site language wins, earlier ranges win ties, and `q=0` excludes a range. `*` and
/// malformed entries are ignored rather than guessed at.
fn accept_language_locale(accept_language: Option<&str>) -> &'static Language {
    let mut best: Option<(&'static Language, u16)> = None;
    for range in accept_language.unwrap_or_default().split(',') {
        let mut params = range.split(';');
        let tag = params.next().unwrap_or_default().trim();
        let primary = tag.split('-').next().unwrap_or_default();
        let Some(candidate) = SITE_LANGUAGES
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

#[cfg(test)]
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
}
