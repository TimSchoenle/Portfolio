//! The language marker that gives the incremental render cache one entry per language.
//!
//! Dioxus's incremental renderer keys its cache on the request path and query alone, and this
//! site negotiates the language per request. [`crate::page::localize_page`] therefore folds the
//! negotiated language into the request URI with [`with_locale_query`], and the cache's path
//! mapper reads it back with [`locale_from_query`].

use portfolio_data::LANGUAGES;

/// The query-string key [`crate::page::localize_page`] appends to page requests.
///
/// It makes the otherwise language-blind, path-keyed incremental cache store one entry per
/// negotiated language. It is server-internal: the router matches on the path and ignores it, and every
/// link is built from the router or the site config, so it never reaches the rendered HTML or
/// the address bar.
///
/// Because the marker travels in a *client-supplied* URI, it is never trusted on the way in:
/// [`with_locale_query`] strips any copy the request already carried before appending ours, and
/// [`locale_from_query`] only ever returns a [`LANGUAGES`] entry. Without both, a request could
/// choose which cache entry its render lands in (serving one visitor's language to another)
/// and, since the value becomes a path component, where on disk it is written.
pub const ISR_LOCALE_PARAM: &str = "__isr_locale";

/// [`ISR_LOCALE_PARAM`] with the `=` that separates a query pair's key from its value, so
/// [`locale_from_query`] — called by the renderer's path mapper on every cacheable page
/// request — does not build the needle with a `format!` each time.
const ISR_LOCALE_QUERY_PREFIX: &str = "__isr_locale=";

/// Compile-time proof that the two constants above have not drifted apart: a mapper that stops
/// recognizing the marker silently collapses every language back onto one cache entry.
const _: () = {
    let key = ISR_LOCALE_PARAM.as_bytes();
    let prefix = ISR_LOCALE_QUERY_PREFIX.as_bytes();
    assert!(prefix.len() == key.len() + 1);
    assert!(prefix[key.len()] == b'=');
    let mut i = 0;
    while i < key.len() {
        assert!(prefix[i] == key[i]);
        i += 1;
    }
};

/// Returns `uri` with `ISR_LOCALE_PARAM=<locale>` appended to its query string, preserving any
/// query already present *except* a copy of the marker itself.
///
/// Stripping the incoming marker is what keeps the cache key ours. The value becomes both the
/// cache bucket and an on-disk path component, so a request allowed to supply its own could
/// file a render under a language it is not written in and escape the cache directory with
/// `../` or a leading `/`. If the rewrite does not parse, the original URI is returned so the
/// page still renders.
#[must_use]
pub fn with_locale_query(uri: &axum::http::Uri, locale: &str) -> axum::http::Uri {
    let path = uri.path();
    let retained: Vec<&str> = uri
        .query()
        .unwrap_or("")
        .split('&')
        .filter(|pair| !pair.is_empty() && !is_locale_marker(pair))
        .collect();

    let mut combined = String::from(path);
    combined.push('?');
    for pair in retained {
        combined.push_str(pair);
        combined.push('&');
    }
    combined.push_str(ISR_LOCALE_QUERY_PREFIX);
    combined.push_str(locale);

    let Ok(path_and_query) = combined.parse::<axum::http::uri::PathAndQuery>() else {
        return uri.clone();
    };
    let mut parts = uri.clone().into_parts();
    parts.path_and_query = Some(path_and_query);
    axum::http::Uri::from_parts(parts).unwrap_or_else(|_| uri.clone())
}

/// Whether a raw `key=value` query pair carries the server-internal locale marker, whatever
/// its value (including none).
fn is_locale_marker(pair: &str) -> bool {
    pair.split('=').next() == Some(ISR_LOCALE_PARAM)
}

/// Extracts the [`ISR_LOCALE_PARAM`] value from a raw query string, if it names a language the
/// site supports.
///
/// The validation is the load-bearing part: the marker rides in on a client-supplied URI, so
/// anything that is not a [`LANGUAGES`] entry is forged — and this value becomes a directory
/// name.
#[must_use]
pub fn locale_from_query(query: &str) -> Option<&str> {
    query
        .split('&')
        .filter_map(|pair| pair.strip_prefix(ISR_LOCALE_QUERY_PREFIX))
        .find(|locale| LANGUAGES.contains(locale))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locale_query_is_appended_to_a_bare_path() {
        let tagged = with_locale_query(&"/".parse().unwrap(), "de");
        assert_eq!(
            tagged.path_and_query().unwrap().as_str(),
            "/?__isr_locale=de"
        );
    }

    #[test]
    fn locale_query_preserves_an_existing_query() {
        let tagged = with_locale_query(&"/imprint?ref=x".parse().unwrap(), "en");
        assert_eq!(
            tagged.path_and_query().unwrap().as_str(),
            "/imprint?ref=x&__isr_locale=en"
        );
    }

    #[test]
    fn a_client_supplied_locale_marker_is_stripped_before_ours_is_appended() {
        let tagged = with_locale_query(&"/?__isr_locale=de".parse().unwrap(), "en");
        assert_eq!(
            tagged.path_and_query().unwrap().as_str(),
            "/?__isr_locale=en"
        );

        // Including when smuggled among real parameters, repeated, or valueless — and
        // unrelated parameters still survive.
        let tagged = with_locale_query(
            &"/imprint?__isr_locale=de&ref=x&__isr_locale&__isr_locale=fr"
                .parse()
                .unwrap(),
            "en",
        );
        assert_eq!(
            tagged.path_and_query().unwrap().as_str(),
            "/imprint?ref=x&__isr_locale=en"
        );
    }

    #[test]
    fn only_a_site_language_is_read_back() {
        assert_eq!(locale_from_query("ref=x&__isr_locale=de"), Some("de"));
        assert_eq!(locale_from_query("__isr_locale=../etc"), None);
        assert_eq!(locale_from_query("__isr_locale=fr"), None);
        assert_eq!(locale_from_query(""), None);
    }
}
