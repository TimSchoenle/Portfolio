//! The catch-all page every unrouted URL lands on.

use dioxus::prelude::*;

use crate::i18n::use_i18n;
use crate::routes::Route;

/// Answers any path no other route claimed.
///
/// `segments` is what the catch-all captured, and nothing reads it. The page is deliberately
/// identical for every unknown URL, which is also what makes it safe to render one that a
/// visitor did not type.
#[component]
pub fn NotFound(segments: Vec<String>) -> Element {
    let _ = segments;
    let i18n = use_i18n().i18n;
    let t = move |k: &str| i18n.read().t(k);

    rsx! {
        document::Title { "404" }
        // The catch-all route answers every unknown URL with this page, so it
        // must not be indexed: without it each mistyped or stale link becomes a
        // separate thin page competing with the real ones. Deliberately carries
        // no `rel="canonical"` — there is no canonical URL for "not found".
        Meta { name: "robots", content: "noindex, follow" }
        section { class: "notfound",
            div {
                span { class: "mono text-muted", "// signal_lost.404" }
                h1 { {t("notFound.title")} }
                p { {t("notFound.description")} }
                Link { to: Route::Home {}, class: "btn-accent",
                    span { class: "mono", {t("common.backToHome")} }
                }
            }
        }
    }
}
