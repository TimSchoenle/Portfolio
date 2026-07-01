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

    rsx! {
        document::Link {
            rel: "icon",
            r#type: "image/svg+xml",
            href: asset!("/assets/favicon.svg"),
        }

        I18nProvider {
            translations: translations,
            default_language: "en".to_string(),
            storage_name: LANG_STORAGE_KEY.to_string(),
            Router::<Route> {}
        }
    }
}
