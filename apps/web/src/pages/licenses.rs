//! `/licenses` — the third-party attribution page.
//!
//! Rendered from the inventory `cargo about` produced for *this* build (see
//! [`crate::licenses`]), so what the page claims and what the binary links are
//! the same fact. Nothing here is fetched at runtime and nothing is
//! hand-maintained: adding a dependency changes this page on the next build.
//!
//! Three views, in the order a reader needs them — which licences are involved,
//! which dependencies are involved, and the texts themselves.

use dioxus::prelude::*;
use portfolio_data::{CONFIG, licenses::LicensesFile};

use crate::i18n::use_i18n;
use crate::licenses::load_licenses;
use crate::routes::Route;
use crate::ui::canonical::Canonical;

#[component]
pub fn Licenses() -> Element {
    let i18n = use_i18n().i18n;
    let t = move |k: &str| i18n.read().t(k);

    let title = t("licenses.title");
    let full_title = format!("{} · {}", title, CONFIG.full_name);

    rsx! {
        document::Title { "{full_title}" }
        Canonical { path: "/licenses" }
        section { class: "licenses-page",
            Link { to: Route::Home {}, class: "legal-back mono",
                "← "
                {t("common.backToHome")}
            }
            h1 { class: "legal-title", "{title}" }
            p { class: "licenses-intro", {t("licenses.intro")} }

            if let Some(file) = load_licenses() {
                Inventory { file }
            } else {
                p { class: "licenses-unavailable", {t("licenses.unavailable")} }
            }
        }
    }
}

/// The three views over a present inventory.
///
/// A separate component so the page above reads as one decision — there is an
/// inventory or there is not — and so everything below can take the document as
/// a plain `&'static` borrow rather than an `Option` each block has to unwrap.
#[component]
fn Inventory(file: &'static LicensesFile) -> Element {
    let i18n = use_i18n().i18n;
    let t = move |k: &str| i18n.read().t(k);

    // Collected once: the heading needs the count and the list needs the items,
    // and `third_party` is a filter over the whole graph rather than a field.
    let dependencies: Vec<_> = file.third_party().collect();

    let unit = t("licenses.dependencyUnit");
    let dependencies_heading = format!(
        "{} ({})",
        t("licenses.dependenciesHeading"),
        dependencies.len()
    );
    let texts_heading = format!("{} ({})", t("licenses.textsHeading"), file.texts.len());

    rsx! {
        section { class: "licenses-block",
            h2 { {t("licenses.summaryHeading")} }
            ul { class: "license-summary",
                {file.summary.iter().map(|l| rsx! {
                    li { key: "{l.id}", class: "license-chip",
                        span { class: "license-chip-id", "{l.id}" }
                        span { class: "license-chip-name", "{l.name}" }
                        span { class: "license-chip-count", "{l.crates} {unit}" }
                    }
                })}
            }
        }

        section { class: "licenses-block",
            h2 { "{dependencies_heading}" }
            ul { class: "license-deps",
                {dependencies.iter().map(|c| rsx! {
                    li { key: "{c.name} {c.version}", class: "license-dep",
                        // Unlinked when the crate declares no repository, rather
                        // than an anchor with nowhere to go.
                        if let Some(url) = c.repository.as_deref() {
                            a {
                                class: "license-dep-name",
                                href: "{url}",
                                target: "_blank",
                                rel: "noopener noreferrer",
                                "{c.name}"
                            }
                        } else {
                            span { class: "license-dep-name", "{c.name}" }
                        }
                        span { class: "license-dep-version", "{c.version}" }
                        span { class: "license-dep-spdx", "{c.license}" }
                    }
                })}
            }
        }

        section { class: "licenses-block",
            h2 { "{texts_heading}" }
            // Collapsed by default: the texts are the reason the page exists, but
            // a hundred-odd of them expanded is not a page anyone can read. They
            // are in the document either way — `details` hides them, it does not
            // defer them — so Ctrl+F and a crawler still find every word.
            {file.texts.iter().enumerate().map(|(i, l)| {
                let used_by = l.used_by
                    .iter()
                    .map(|c| format!("{} {}", c.name, c.version))
                    .collect::<Vec<_>>()
                    .join(", ");
                rsx! {
                    details { key: "{i}", class: "license-text",
                        summary {
                            span { class: "license-text-name", "{l.name}" }
                            span { class: "license-text-id", "{l.id}" }
                        }
                        div { class: "license-text-body",
                            p { class: "license-text-usedby", "{used_by}" }
                            pre { "{l.text}" }
                        }
                    }
                }
            })}
        }
    }
}
