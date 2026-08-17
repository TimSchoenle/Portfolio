//! Root component: the i18n provider wrapping the router, plus the document
//! head shared across routes.

use std::collections::HashMap;

use dioxus::prelude::*;
use i18nrs::dioxus::I18nProvider;
use portfolio_data::{CONFIG, I18N_DE, I18N_EN, OG_IMAGE_FILE, OG_IMAGE_SIZE};

use crate::i18n::LANG_STORAGE_KEY;
use crate::routes::Route;

/// The web font faces the first paint actually uses: the hero name — the largest
/// contentful element on the page — and the monospace labels in the masthead and
/// chapter rail.
///
/// They are preloaded because `fonts.css` is a separate stylesheet, so without a
/// hint the browser cannot discover them until it has fetched *and parsed* that
/// file: HTML, then CSS, then font, three round trips deep before the text they
/// style can be painted in them. `font-display: swap` keeps the page readable
/// throughout — the preload is what shortens the swap.
///
/// Deliberately only these two. Every other face is either below the fold or in
/// a weight the first screen never reaches, and a preload the page does not
/// consume within a few seconds is bandwidth taken from the wasm bundle and a
/// console warning besides. Keep the paths in sync with `assets/fonts.css` and
/// the embedded table in `server::assets`.
const PRELOADED_FONTS: [&str; 2] = [
    "/fonts/space-grotesk-latin-700-normal.woff2",
    "/fonts/jetbrains-mono-latin-400-normal.woff2",
];

#[component]
pub fn App() -> Element {
    let translations = HashMap::from([("en", I18N_EN), ("de", I18N_DE)]);

    // The embedded repo list, parsed once and shared with the projects section
    // and the command palette. Identical on the server and client, so it
    // hydrates without a mismatch.
    use_context_provider(crate::github::load_repos);

    // Initial language. On the server it is negotiated from the request (cookie
    // / Accept-Language) and written back as a cookie; on the client it is read
    // synchronously from that same `lang` cookie, so SSR and hydration agree
    // without any server round-trip.
    let default_language: String = {
        #[cfg(feature = "server")]
        {
            crate::i18n::detect_locale()
        }
        #[cfg(all(not(feature = "server"), feature = "web"))]
        {
            crate::i18n::detect_locale()
        }
        #[cfg(not(any(feature = "server", feature = "web")))]
        {
            "en".to_string()
        }
    };

    rsx! {
        // Declared before the stylesheets so the browser meets them as early in
        // the document as this component can put them.
        for href in PRELOADED_FONTS {
            document::Link {
                key: "{href}",
                rel: "preload",
                r#as: "font",
                r#type: "font/woff2",
                href,
                // Fonts are fetched in CORS mode even from their own origin, so
                // a preload without this is a second, separate request rather
                // than a warm cache entry for the one the stylesheet makes.
                crossorigin: "anonymous",
            }
        }

        document::Stylesheet { href: asset!("/assets/tailwind.css") }
        document::Stylesheet { href: asset!("/assets/fonts.css") }
        // Stable path served by the server (see server::assets), matching the
        // masthead logo and the web manifest icon.
        document::Link { rel: "icon", r#type: "image/svg+xml", href: "/favicon.svg" }

        SiteHead { lang: default_language.clone() }

        I18nProvider {
            translations: translations,
            default_language: default_language,
            storage_name: LANG_STORAGE_KEY.to_string(),
            Router::<Route> {}
        }
    }
}

/// Site-wide document metadata rendered into the `<head>` on both the server
/// (so crawlers and link-unfurlers see it in the initial HTML) and the client
/// (where `dioxus-web`'s document provider re-applies it during hydration).
///
/// The per-page `<title>` and `rel="canonical"` live with each page
/// (`document::Title`, [`crate::ui::canonical::Canonical`]); everything here is
/// route-independent and derived from [`CONFIG`], the single source of truth.
/// `lang` is the request-negotiated locale, used only for `og:locale`.
#[component]
fn SiteHead(lang: String) -> Element {
    let keywords = CONFIG.keywords.join(", ");
    // The generated 1200×630 card, not the favicon: every consumer of these tags
    // — Facebook, LinkedIn, Slack, X — refuses SVG, so pointing at one produced a
    // tag that was well-formed and showed nothing anywhere. Absolute, because a
    // relative `og:image` is not resolved by most of them.
    let image = format!("{}/{}", CONFIG.url, OG_IMAGE_FILE);
    let (image_width, image_height) = OG_IMAGE_SIZE;
    let og_locale = match lang.as_str() {
        "de" => "de_DE",
        _ => "en_US",
    };

    rsx! {
        // Core description / indexing hints.
        Meta { name: "description", content: CONFIG.description }
        Meta { name: "keywords", content: keywords }
        Meta { name: "author", content: CONFIG.full_name }
        Meta { name: "theme-color", content: "#0a0d14" }

        // Open Graph (Facebook, LinkedIn, Slack, …).
        Meta { property: "og:type", content: "website" }
        Meta { property: "og:site_name", content: CONFIG.full_name }
        Meta { property: "og:title", content: CONFIG.title }
        Meta { property: "og:description", content: CONFIG.description }
        Meta { property: "og:url", content: CONFIG.url }
        Meta { property: "og:image", content: image.clone() }
        // Declared so a consumer can reserve the space before the bytes arrive;
        // both come from the constant the generator renders to.
        Meta { property: "og:image:width", content: image_width.to_string() }
        Meta { property: "og:image:height", content: image_height.to_string() }
        Meta { property: "og:image:alt", content: CONFIG.title }
        Meta { property: "og:image:type", content: "image/png" }
        Meta { property: "og:locale", content: og_locale }

        // Twitter / X card. `summary_large_image` rather than `summary`: the card
        // is the 1.91:1 shape that variant expects, and the small variant would
        // crop it to a square thumbnail.
        Meta { name: "twitter:card", content: "summary_large_image" }
        Meta { name: "twitter:title", content: CONFIG.title }
        Meta { name: "twitter:description", content: CONFIG.description }
        Meta { name: "twitter:image", content: image }
        Meta { name: "twitter:image:alt", content: CONFIG.title }
    }
}
