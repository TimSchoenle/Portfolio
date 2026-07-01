//! Root component: the i18n provider wrapping the router, plus the document
//! head shared across routes.

use std::collections::HashMap;

use dioxus::prelude::*;
use i18nrs::dioxus::I18nProvider;
use portfolio_data::{I18N_DE, I18N_EN};

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

        I18nProvider {
            translations: translations,
            default_language: default_language,
            storage_name: LANG_STORAGE_KEY.to_string(),
            Router::<Route> {}
        }
    }
}
