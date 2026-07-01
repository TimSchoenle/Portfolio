//! Root component. Phase 1 is a minimal placeholder to validate the fullstack
//! build; the router, i18n provider and real UI are ported in later phases.

use dioxus::prelude::*;

#[component]
pub fn App() -> Element {
    rsx! {
        main {
            h1 { "Portfolio — Dioxus fullstack scaffold" }
        }
    }
}
