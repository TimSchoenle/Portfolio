//! Root component: the i18n provider wrapping the router, plus the document
//! head shared across routes.

use std::collections::HashMap;

use dioxus::prelude::*;
use i18nrs::dioxus::I18nProvider;
use portfolio_data::{CONFIG, I18N_DE, I18N_EN};

use crate::i18n::LANG_STORAGE_KEY;
use crate::routes::Route;

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
    let image = format!("{}/favicon.svg", CONFIG.url);
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
        Meta { property: "og:locale", content: og_locale }

        // Twitter / X card.
        Meta { name: "twitter:card", content: "summary" }
        Meta { name: "twitter:title", content: CONFIG.title }
        Meta { name: "twitter:description", content: CONFIG.description }
        Meta { name: "twitter:image", content: image }
    }
}
