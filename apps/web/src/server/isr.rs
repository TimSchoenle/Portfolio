//! Incremental static regeneration: the Dioxus render cache, keyed per language, and the
//! allowlist that keeps request text out of it.
//!
//! Dioxus's incremental renderer keys on the request path and query alone, and this site
//! negotiates the language per request, so a per-path cache would serve whichever language
//! rendered a URL first to everyone who asked for it afterwards. [`with_locale_query`] puts the
//! negotiated language on the URI and [`isr_map_path`] nests each render under it, which is what
//! makes the key one entry per language per path.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use axum::Router;
use dioxus::server::{DioxusRouterExt, IncrementalRendererConfig, ServeConfig};
use portfolio_config::IsrConfig;
use portfolio_data::LANGUAGES;

/// Query-string marker [`super::page::localize_page`] appends to page requests so the
/// otherwise language-blind, path-keyed incremental cache stores one entry per negotiated
/// language. It is server-internal: the router matches on the path and ignores it, and every
/// link is built from the router or the site config, so it never reaches the rendered HTML or
/// the address bar.
///
/// Because the marker travels in a *client-supplied* URI, it is never trusted on the way in:
/// [`with_locale_query`] strips any copy the request already carried before appending ours, and
/// [`locale_from_query`] only ever returns a [`LANGUAGES`] entry. Without both, a request could
/// choose which cache entry its render lands in (serving one visitor's language to another)
/// and, since the value becomes a path component, where on disk it is written.
pub(super) const ISR_LOCALE_PARAM: &str = "__isr_locale";

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

/// Page paths whose rendered HTML is worth persisting: the fixed routes of
/// `crate::routes::Route`, plus one `/legal/<slug>` per document the configuration hosts. The
/// catch-all `NotFound` route is deliberately absent, and so is every slug nothing is published
/// under.
///
/// The incremental cache is keyed by path, and the catch-all matches every URL that exists — so
/// without this allowlist an unauthenticated client could mint an unbounded number of cache
/// entries (one directory and one HTML file each) by requesting `/1`, `/2`, … until the volume
/// filled up. The same holds for `/legal/:slug`, whose segment is request text. Routes outside
/// the list still render normally; their output is just never stored (see [`isr_map_path`]).
///
/// Cheap to clone: the list is shared.
#[derive(Clone)]
pub(super) struct PagePaths(Arc<[&'static str]>);

/// The routes whose paths do not depend on configuration.
const STATIC_PAGE_PATHS: [&str; 2] = ["/", "/licenses"];

impl PagePaths {
    /// The fixed routes, then `legal`: each a hosted document's path.
    ///
    /// The configured paths are leaked into `&'static str` on purpose, and once per process:
    /// `serve` builds this before the runtime starts, from a configuration that is never
    /// reloaded. That lets the page memo key on a program-lifetime string instead of on request
    /// text or a per-request allocation.
    pub(super) fn new(legal: &[String]) -> Self {
        let configured = legal
            .iter()
            .map(|path| &*Box::leak(path.clone().into_boxed_str()));
        Self(STATIC_PAGE_PATHS.into_iter().chain(configured).collect())
    }

    /// The entry equal to `path`.
    ///
    /// Returning the stored entry rather than a `bool` is what lets the page memo key on a
    /// `&'static str`: the request's own path is borrowed from a URI that does not outlive the
    /// request, whereas this one lives as long as the process.
    pub(super) fn get(&self, path: &str) -> Option<&'static str> {
        self.0.iter().copied().find(|known| *known == path)
    }

    /// Whether a request path is one of these.
    pub(super) fn contains(&self, path: &str) -> bool {
        self.get(path).is_some()
    }
}

/// Name of the sentinel that [`isr_map_path`] maps every non-cacheable route to.
///
/// [`ensure_uncacheable_sentinel`] creates it as a regular **file** inside the cache directory,
/// which is what makes the opt-out work: the renderer persists a page by `create_dir_all`-ing
/// the mapped folder and writing `index.html` into it, and neither can succeed underneath a
/// plain file. Lookups miss for the same reason. Collapsing every such route onto one path
/// rather than giving each its own is deliberate: a shared *cache entry* would serve one unknown
/// URL's render (including its serialized hydration route) for a different unknown URL.
pub(super) const ISR_UNCACHEABLE_SENTINEL: &str = ".uncacheable";

/// The Dioxus SSR and asset router, and whether it came back with the incremental cache on.
///
/// The cache is on only when the configured directory turns out to be writable; the caller
/// tags page requests with the negotiated language only when the flag is `true`.
pub(super) fn dioxus_app_router(isr: &IsrConfig, paths: &PagePaths) -> (Router, bool) {
    match incremental_config(isr, paths) {
        Some(cfg) => (
            Router::new()
                .serve_dioxus_application(ServeConfig::builder().incremental(cfg), crate::app::App),
            true,
        ),
        // Unset: keep the framework default (fresh render, no on-disk cache).
        None => (dioxus::server::router(crate::app::App), false),
    }
}

/// Builds the incremental-render configuration, or `None` when ISR is disabled (no cache
/// directory configured) or the directory is not usable (cannot be created, or exists but is
/// not writable).
pub(super) fn incremental_config(
    isr: &IsrConfig,
    paths: &PagePaths,
) -> Option<IncrementalRendererConfig> {
    let dir = isr.cache_dir()?.to_path_buf();
    // Probe for writability, not mere existence: a mounted volume (a root-owned `emptyDir`, say)
    // can be present yet unwritable to our non-root user, in which case `create_dir_all`
    // succeeds but the renderer would fail to persist every page at runtime.
    if let Err(err) = ensure_writable_dir(&dir) {
        tracing::warn!(
            "ISR disabled: cache directory {} is not usable: {err}",
            dir.display()
        );
        return None;
    }
    if let Err(err) = ensure_uncacheable_sentinel(&dir) {
        tracing::warn!(
            "ISR disabled: cannot create the uncacheable-route sentinel in {}: {err}",
            dir.display()
        );
        return None;
    }

    let invalidate_after = isr.invalidate_after();
    if let Some(ttl) = invalidate_after {
        tracing::info!(
            "ISR enabled: caching rendered pages per locale in {} (revalidate after {}s)",
            dir.display(),
            ttl.as_secs()
        );
    } else {
        tracing::info!(
            "ISR enabled: caching rendered pages per locale in {} (permanent; refreshed only by a new deploy)",
            dir.display()
        );
    }
    let map_dir = dir.clone();
    let map_paths = paths.clone();
    // Which entries have already been announced as created, so the log line fires once per
    // creation rather than on every `map_path` call (the renderer calls it for the cache-miss
    // lookup and again for the write). The page memo answers every repeat request before the
    // renderer is reached, so this — and the file-system probe in `is_new_cache_entry` — runs
    // about once per page and language per process rather than on every request.
    let announced: Mutex<HashSet<PathBuf>> = Mutex::new(HashSet::new());
    let mut config = IncrementalRendererConfig::new()
        .static_dir(dir)
        // Fold the locale marker into the on-disk path: the default mapping drops the query
        // string, collapsing every language onto one file.
        .map_path(move |route| {
            let mapped = isr_map_path(&map_dir, route, &map_paths);
            if is_cacheable_route(route, &map_paths) && is_new_cache_entry(&announced, &mapped) {
                log_cache_entry_created(route, &mapped);
            }
            mapped
        })
        // Keep any pages regenerated by a previous pod across restarts.
        .clear_cache(false);
    // Only impose a time-based TTL when one is explicitly configured; the default is a
    // permanent cache, invalidated instead by the next build starting empty.
    if let Some(ttl) = invalidate_after {
        config = config.invalidate_after(ttl);
    }
    Some(config)
}

/// Ensures `dir` exists and is writable by this process: creates it (and any missing
/// parents), then probes it with a throwaway file, which is best-effort removed afterwards.
pub(super) fn ensure_writable_dir(dir: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    let probe = dir.join(".isr-write-test");
    std::fs::write(&probe, b"")?;
    let _ = std::fs::remove_file(&probe);
    Ok(())
}

/// Creates the [`ISR_UNCACHEABLE_SENTINEL`] as a regular file inside `dir`, so the renderer
/// cannot create directories or write pages beneath it.
///
/// A directory left at that path by an older build (only possible when a persistent cache
/// volume is shared across deploys) is removed first: left in place it would turn the opt-out
/// into a single shared cache entry, which is what the sentinel exists to prevent.
pub(super) fn ensure_uncacheable_sentinel(dir: &Path) -> std::io::Result<()> {
    let sentinel = dir.join(ISR_UNCACHEABLE_SENTINEL);
    if sentinel.is_dir() {
        std::fs::remove_dir_all(&sentinel)?;
    }
    std::fs::write(
        &sentinel,
        b"Routes mapped here are deliberately never cached.\n",
    )
}

/// Returns `uri` with `ISR_LOCALE_PARAM=<locale>` appended to its query string, preserving any
/// query already present *except* a copy of the marker itself.
///
/// Stripping the incoming marker is what keeps the cache key ours. The value becomes both the
/// cache bucket and an on-disk path component, so a request allowed to supply its own could
/// file a render under a language it is not written in and escape the cache directory with
/// `../` or a leading `/`. If the rewrite does not parse, the original URI is returned so the
/// page still renders.
pub(super) fn with_locale_query(uri: &axum::http::Uri, locale: &str) -> axum::http::Uri {
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

/// Maps an incremental-cache route to its on-disk folder, mirroring Dioxus's default layout but
/// nesting each render under its locale sub-directory. A route without the marker maps straight
/// under `static_dir`, matching the default.
///
/// Routes outside [`PagePaths`] map to the [`ISR_UNCACHEABLE_SENTINEL`] instead, which the
/// renderer can neither read a page from nor write one to.
pub(super) fn isr_map_path(static_dir: &Path, route: &str, paths: &PagePaths) -> PathBuf {
    let (path, query) = route.split_once('?').unwrap_or((route, ""));
    if !paths.contains(path) {
        return static_dir.join(ISR_UNCACHEABLE_SENTINEL).join("route");
    }

    let mut mapped = static_dir.to_path_buf();
    if let Some(locale) = locale_from_query(query) {
        // Safe as a path component only because `locale_from_query` rejects anything that is
        // not a known language code.
        mapped.push(locale);
    }
    for segment in path.split('/') {
        mapped.push(segment);
    }
    mapped
}

/// Whether a full incremental-cache route (path plus query) addresses a page whose render may
/// be persisted.
fn is_cacheable_route(route: &str, paths: &PagePaths) -> bool {
    let (path, _) = route.split_once('?').unwrap_or((route, ""));
    paths.contains(path)
}

/// Extracts the [`ISR_LOCALE_PARAM`] value from a raw query string, if it names a language the
/// site supports.
///
/// The validation is the load-bearing part: the marker rides in on a client-supplied URI, so
/// anything that is not a [`LANGUAGES`] entry is forged — and this value becomes a directory
/// name.
fn locale_from_query(query: &str) -> Option<&str> {
    query
        .split('&')
        .filter_map(|pair| pair.strip_prefix(ISR_LOCALE_QUERY_PREFIX))
        .find(|locale| LANGUAGES.contains(locale))
}

/// Decides whether `mapped` is about to receive a *freshly created* cache entry, and records
/// that so the caller logs it exactly once.
///
/// The renderer calls `map_path` twice around a creation — the cache-miss lookup, then the
/// write — and again for every later cache hit, so the decision keys off whether a rendered
/// `.html` already exists:
/// * none yet, and not already announced → a new entry (`true`, remembered so the paired
///   write call stays quiet);
/// * one exists → the entry is live; forget any announcement so a post-TTL regeneration is
///   reported again (`false`).
pub(super) fn is_new_cache_entry(announced: &Mutex<HashSet<PathBuf>>, mapped: &Path) -> bool {
    // Probed before the lock is taken, so no request waits on another's file-system call.
    let cached = has_cached_render(mapped);
    let mut announced = announced
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if cached {
        announced.remove(mapped);
        false
    } else {
        announced.insert(mapped.to_path_buf())
    }
}

/// Whether `mapped` already holds a rendered page.
///
/// Dioxus's `FileSystemCache` uses two layouts depending on whether a TTL is configured, and
/// both have to be recognized:
/// * permanent cache (no TTL): `<mapped>/index.html`;
/// * finite TTL: `<mapped>/index/<timestamp>.html`, so any `.html` in that directory counts.
pub(super) fn has_cached_render(mapped: &Path) -> bool {
    if mapped.join("index.html").is_file() {
        return true;
    }
    std::fs::read_dir(mapped.join("index"))
        .into_iter()
        .flatten()
        .flatten()
        .any(|entry| entry.path().extension().is_some_and(|ext| ext == "html"))
}

/// Logs, at INFO, that the renderer just created a new on-disk cache entry, naming the
/// language taken from the server-internal marker.
fn log_cache_entry_created(route: &str, mapped: &Path) {
    let (path, query) = route.split_once('?').unwrap_or((route, ""));
    let locale = locale_from_query(query).unwrap_or("default");
    tracing::info!(
        "ISR cache entry created for path \"{path}\" (locale \"{locale}\") at {}",
        mapped.display()
    );
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;
    use crate::server::legal;

    /// The allowlist the fixture catalog produces.
    pub(crate) fn paths() -> PagePaths {
        PagePaths::new(&legal::hosted_paths(&legal::tests::fixture()))
    }

    /// A unique scratch path under the OS temp dir; only its name is reserved.
    pub(crate) fn scratch_path(tag: &str) -> PathBuf {
        use std::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("isr-test-{tag}-{}-{n}", std::process::id()))
    }

    #[test]
    fn writable_dir_probe_accepts_a_creatable_directory() {
        let dir = scratch_path("ok").join("nested");
        assert!(ensure_writable_dir(&dir).is_ok());
        assert!(dir.is_dir());
        assert!(!dir.join(".isr-write-test").exists());
        let _ = std::fs::remove_dir_all(dir.parent().unwrap());
    }

    #[test]
    fn writable_dir_probe_rejects_a_path_blocked_by_a_file() {
        let file = scratch_path("blocked");
        std::fs::write(&file, b"not a directory").unwrap();
        assert!(ensure_writable_dir(&file.join("cache")).is_err());
        let _ = std::fs::remove_file(&file);
    }

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
    fn a_forged_locale_never_becomes_a_path_component() {
        let base = Path::new("/cache");
        let untagged = isr_map_path(base, "/", &paths());
        for forged in [
            "/?__isr_locale=../../../../etc/evil",
            "/?__isr_locale=/var/www/html",
            "/?__isr_locale=fr",
            "/?__isr_locale=",
        ] {
            let mapped = isr_map_path(base, forged, &paths());
            assert_eq!(mapped, untagged, "{forged} was not neutralized");
            assert!(
                mapped.starts_with(base),
                "{forged} escaped the cache directory"
            );
        }
    }

    #[test]
    fn only_allowlisted_pages_are_cacheable() {
        let paths = paths();
        for path in ["/", "/legal/imprint", "/legal/privacy", "/licenses"] {
            assert!(paths.contains(path), "{path} should be cacheable");
        }
        // `/legal/terms` is published but hosted elsewhere; `/imprint` is only a redirect.
        for path in [
            "/nope",
            "/1",
            "/imprint",
            "/legal/nope",
            "/legal/terms",
            "/legal/imprint/x",
            "/robots.txt",
            "",
        ] {
            assert!(!paths.contains(path), "{path} should not be cacheable");
        }
    }

    #[test]
    fn uncacheable_routes_collapse_onto_an_unwritable_sentinel() {
        let dir = scratch_path("sentinel");
        std::fs::create_dir_all(&dir).unwrap();
        ensure_uncacheable_sentinel(&dir).unwrap();

        let a = isr_map_path(&dir, "/nope?__isr_locale=en", &paths());
        let b = isr_map_path(&dir, "/other?__isr_locale=de", &paths());
        assert_eq!(a, b);
        assert_ne!(a, isr_map_path(&dir, "/?__isr_locale=en", &paths()));

        assert!(dir.join(ISR_UNCACHEABLE_SENTINEL).is_file());
        assert!(std::fs::create_dir_all(&a).is_err());
        assert!(!has_cached_render(&a));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_sentinel_replaces_a_directory_left_by_an_older_deploy() {
        let dir = scratch_path("sentinel-dir");
        let stale = dir
            .join(ISR_UNCACHEABLE_SENTINEL)
            .join("route")
            .join("index");
        std::fs::create_dir_all(&stale).unwrap();
        std::fs::write(stale.join("old.html"), b"<html></html>").unwrap();

        ensure_uncacheable_sentinel(&dir).unwrap();
        assert!(dir.join(ISR_UNCACHEABLE_SENTINEL).is_file());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_permanent_cache_entry_is_recognised_as_rendered() {
        let mapped = scratch_path("permanent");
        std::fs::create_dir_all(&mapped).unwrap();
        assert!(!has_cached_render(&mapped));

        std::fs::write(mapped.join("index.html"), b"<html></html>").unwrap();
        assert!(has_cached_render(&mapped));

        let _ = std::fs::remove_dir_all(&mapped);
    }

    #[test]
    fn map_path_separates_locales_and_falls_back_without_a_marker() {
        let base = Path::new("/cache");
        let has =
            |p: &Path, name: &str| p.components().any(|c| c.as_os_str().to_str() == Some(name));

        let de = isr_map_path(base, "/?__isr_locale=de", &paths());
        let en = isr_map_path(base, "/?__isr_locale=en", &paths());
        let untagged = isr_map_path(base, "/", &paths());

        assert_ne!(de, en);
        assert_ne!(de, untagged);
        assert!(has(&de, "de"));
        assert!(has(&en, "en"));
        assert!(!has(&untagged, "de") && !has(&untagged, "en"));

        let imprint_de = isr_map_path(base, "/legal/imprint?__isr_locale=de", &paths());
        assert!(has(&imprint_de, "de") && has(&imprint_de, "imprint"));
        assert_ne!(imprint_de, de);
    }

    /// ISR is off unless the configuration names a cache directory. The TTL and empty-value
    /// semantics belong to `portfolio_config::IsrConfig` and are tested there; this pins that
    /// the server asks it rather than reading the environment behind its back.
    #[test]
    fn isr_is_off_without_a_configured_cache_directory() {
        assert!(incremental_config(&IsrConfig::default(), &paths()).is_none());
    }

    #[test]
    fn a_writable_cache_directory_turns_isr_on() {
        let dir = scratch_path("isr-config");
        let on = IsrConfig {
            cache_dir: Some(dir.clone()),
            ttl_secs: 0,
        };
        assert!(incremental_config(&on, &paths()).is_some());
        assert!(dir.join(ISR_UNCACHEABLE_SENTINEL).is_file());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A path under a regular file can never be created — the one "unwritable" shape that
    /// reproduces on every platform (root ignores a mode-0555 directory).
    #[test]
    fn an_unusable_cache_directory_disables_isr_instead_of_failing() {
        let blocker = scratch_path("isr-blocker");
        std::fs::write(&blocker, b"not a directory").unwrap();
        let unusable = IsrConfig {
            cache_dir: Some(blocker.join("cache")),
            ttl_secs: 0,
        };
        assert!(incremental_config(&unusable, &paths()).is_none());
        let _ = std::fs::remove_file(&blocker);
    }

    #[test]
    fn a_new_cache_entry_is_announced_once_then_re_armed_after_invalidation() {
        let mapped = scratch_path("entry");
        let index = mapped.join("index");
        let announced: Mutex<HashSet<PathBuf>> = Mutex::new(HashSet::new());

        assert!(is_new_cache_entry(&announced, &mapped));
        assert!(!is_new_cache_entry(&announced, &mapped));

        std::fs::create_dir_all(&index).unwrap();
        std::fs::write(index.join("deadbeef.html"), b"<html></html>").unwrap();
        assert!(!is_new_cache_entry(&announced, &mapped));

        std::fs::remove_file(index.join("deadbeef.html")).unwrap();
        assert!(is_new_cache_entry(&announced, &mapped));

        let _ = std::fs::remove_dir_all(&mapped);
    }
}
