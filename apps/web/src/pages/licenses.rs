//! `/licenses` — the third-party attribution page.
//!
//! Rendered from the inventory `cargo about` produced for *this* build (see
//! [`crate::licenses`]), so what the page claims and what the binary links are
//! the same fact. Nothing here is fetched at runtime and nothing is
//! hand-maintained: adding a dependency changes this page on the next build.
//!
//! Two views: which licences are involved, then every dependency — each one
//! expanding to the licence text it ships under. The texts are not a section of
//! their own, because that section could only have listed the same dependency
//! names a second time to say which text belonged to which.

use dioxus::prelude::*;
use portfolio_data::{CONFIG, licenses::LicensesFile};

use crate::i18n::use_i18n;
use crate::licenses::load_licenses;
use crate::routes::Route;
use crate::ui::canonical::Canonical;

/// Renders the inventory `cargo about` produced for this build, or a localized notice when the
/// build produced none.
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

/// The two views over a present inventory.
///
/// A separate component so the page above reads as one decision — there is an
/// inventory or there is not — and so everything below can take the document as
/// a plain `&'static` borrow rather than an `Option` each block has to unwrap.
#[component]
fn Inventory(file: &'static LicensesFile) -> Element {
    let i18n = use_i18n().i18n;
    let t = move |k: &str| i18n.read().t(k);

    // The join, done once: each dependency with the licence texts naming it.
    let dependencies = file.dependencies();

    let unit = t("licenses.dependencyUnit");
    let dependencies_heading = format!(
        "{} ({})",
        t("licenses.dependenciesHeading"),
        dependencies.len()
    );

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
            // One row per dependency, the whole row a disclosure control: the
            // licence text belongs to the dependency that ships it, and a
            // separate list of texts could only have repeated these names to say
            // which text was whose.
            //
            // Collapsed, not deferred — `details` hides its contents, it does not
            // withhold them — so Ctrl+F and a crawler still reach every word of
            // every licence.
            div { class: "license-deps",
                {dependencies.iter().map(|row| {
                    let dep = row.dependency;
                    rsx! {
                        details { key: "{dep.name} {dep.version}", class: "license-dep",
                            summary {
                                span { class: "license-dep-name", "{dep.name}" }
                                span { class: "license-dep-version", "{dep.version}" }
                                span { class: "license-dep-spdx", "{dep.license}" }
                            }
                            div { class: "license-dep-body",
                                // Inside the body rather than in the summary: a
                                // link there would swallow the click on the one
                                // word a reader aims at to open the row.
                                if let Some(url) = dep.repository.as_deref() {
                                    a {
                                        class: "license-dep-repo",
                                        href: "{url}",
                                        target: "_blank",
                                        rel: "noopener noreferrer",
                                        {url.trim_start_matches("https://").trim_start_matches("http://")}
                                    }
                                }
                                {row.texts.iter().enumerate().map(|(i, l)| rsx! {
                                    div { key: "{i}", class: "license-text",
                                        p { class: "license-text-head",
                                            span { class: "license-text-name", "{l.name}" }
                                            span { class: "license-text-id", "{l.id}" }
                                        }
                                        pre { "{l.text}" }
                                    }
                                })}
                                if row.texts.is_empty() {
                                    p { class: "license-text-missing", {t("licenses.noText")} }
                                }
                            }
                        }
                    }
                })}
            }
        }
    }
}
