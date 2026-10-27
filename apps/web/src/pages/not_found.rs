//! The catch-all page every unrouted URL lands on.

use dioxus::prelude::*;

use crate::i18n::use_i18n;
use crate::routes::Route;

/// Renders the 404 page for any path no other route claimed.
///
/// The page is identical for every unknown URL, so `segments`, which the catch-all captured, is
/// never read.
// Being identical is also what makes it safe to render for a URL a visitor did not type.
#[component]
pub fn NotFound(segments: Vec<String>) -> Element {
    let _ = segments;
    // Answered as a `404`, not as the `200` a rendered route otherwise gets: a crawler treats a
    // `200` here as a real, thin page at every mistyped URL (a "soft 404"), and a request for an
    // asset this replica does not have — a client of a newer deploy, mid-rollout — gets an error
    // it can recognize instead of a page of HTML where its script should be.
    #[cfg(feature = "server")]
    dioxus_fullstack_core::FullstackContext::commit_http_status(http::StatusCode::NOT_FOUND, None);
    let i18n = use_i18n().i18n;
    let t = move |k: &str| i18n.read().t(k);

    rsx! {
        document::Title { "404" }
        // The catch-all route answers every unknown URL with this page, so it
        // must not be indexed: without it each mistyped or stale link becomes a
        // separate thin page competing with the real ones. Deliberately carries
        // no `rel="canonical"`, because there is no canonical URL for "not found".
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
