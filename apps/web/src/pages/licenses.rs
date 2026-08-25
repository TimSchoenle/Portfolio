//! `/licenses` — the third-party attribution page.
//!
//! Rendered from the inventory `cargo about` produced for *this* build (see
//! [`crate::licenses`]), so what the page claims and what the binary links are
//! the same fact. Nothing here is fetched at runtime and nothing is
//! hand-maintained: adding a dependency changes this page on the next build.
//!
//! Three views, in the order the question is usually asked: which licences are
//! involved, then the notices themselves, then every dependency with the terms
//! it offers.
//!
//! **The notices are grouped by licence, not by dependency.** The first version
//! of this page gave each dependency a row and reproduced its licence texts
//! inside it, which served the same MIT paragraph once per crate carrying it —
//! 499 KB of licence text for 215 KB of distinct notices, in a 738 KB document.
//! A notice is reproduced once here and names the dependencies it covers, which
//! is both smaller and the shape the licences themselves ask for.
//! [`LicensesFile::notices`] does the grouping and holds the tests for it.

use std::collections::BTreeSet;

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
            p { class: "licenses-note", {t("licenses.note")} }

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
///
/// Everything under it is plain markup rather than further components on
/// purpose. A `LicenseNotices` prop would be compared for equality on every
/// re-render — a language switch is one — and that comparison is a memcmp over
/// the 215 KB of licence text the props would carry.
#[component]
fn Inventory(file: &'static LicensesFile) -> Element {
    let i18n = use_i18n().i18n;
    let t = move |k: &str| i18n.read().t(k);

    // Both groupings, once per render rather than once per section.
    let licences = file.notices();
    let dependencies: Vec<_> = file.third_party().collect();

    // The dependencies some notice names. A crate outside it is one cargo-about
    // found no licence file for — rare, and worth saying so on its row rather
    // than leaving a reader to conclude the page simply forgot it.
    let attributed: BTreeSet<(&str, &str)> = licences
        .iter()
        .flat_map(|l| l.notices.iter())
        .flat_map(|n| n.crates.iter())
        .map(|c| (c.name.as_str(), c.version.as_str()))
        .collect();

    let unit = move |count: usize, singular: &str, plural: &str| {
        format!("{count} {}", t(if count == 1 { singular } else { plural }))
    };
    let dependency_unit =
        move |count: usize| unit(count, "licenses.dependencyOne", "licenses.dependencyMany");
    let notice_unit = move |count: usize| unit(count, "licenses.noticeOne", "licenses.noticeMany");

    let notice_total: usize = licences.iter().map(|l| l.notices.len()).sum();
    let notices_heading = format!("{} ({notice_total})", t("licenses.noticesHeading"));
    let dependencies_heading = format!(
        "{} ({})",
        t("licenses.dependenciesHeading"),
        dependencies.len()
    );

    rsx! {
        section { class: "licenses-block",
            h2 { {t("licenses.summaryHeading")} }
            ul { class: "license-summary",
                {licences.iter().map(|licence| rsx! {
                    li { key: "{licence.id}", class: "license-chip",
                        span { class: "license-chip-id", "{licence.id}" }
                        span { class: "license-chip-name", "{licence.name}" }
                        span { class: "license-chip-count",
                            {dependency_unit(licence.crates)}
                            // Only when there is more than one: "1 licence text"
                            // beside a licence invites the reader to look for a
                            // distinction that is not there.
                            if licence.notices.len() > 1 {
                                " · "
                                {notice_unit(licence.notices.len())}
                            }
                        }
                    }
                })}
            }
        }

        section { class: "licenses-block",
            h2 { "{notices_heading}" }
            // One block per licence, each holding its distinct notices. A notice
            // is a disclosure row whose summary names the dependencies shipping
            // it, so the collapsed section reads as an index of the graph and
            // only the licence file itself has to be opened.
            //
            // Collapsed, not deferred — `details` hides its contents, it does not
            // withhold them — so Ctrl+F and a crawler still reach every word of
            // every licence.
            {licences.iter().map(|licence| rsx! {
                section { key: "{licence.id}", class: "license-group",
                    h3 { class: "license-group-head",
                        span { class: "license-group-name", "{licence.name} ({licence.id})" }
                        span { class: "license-group-count", {dependency_unit(licence.crates)} }
                    }
                    {licence.notices.iter().enumerate().map(|(i, notice)| {
                        let covered = notice.crate_names().join(", ");
                        rsx! {
                            details { key: "{licence.id}-{i}", class: "license-notice",
                                summary {
                                    span { class: "license-notice-crates", "{covered}" }
                                    span { class: "license-notice-count",
                                        {dependency_unit(notice.crates.len())}
                                    }
                                }
                                pre { "{notice.text}" }
                            }
                        }
                    })}
                }
            })}
        }

        section { class: "licenses-block",
            h2 { "{dependencies_heading}" }
            // A flat inventory, not a disclosure list: the terms a dependency
            // *offers* are one line, and the text it was resolved under is above,
            // under the licence that owns it. Reproducing it here again is what
            // made this page 738 KB.
            div { class: "license-deps",
                {dependencies.iter().map(|dep| {
                    let unattributed =
                        !attributed.contains(&(dep.name.as_str(), dep.version.as_str()));
                    rsx! {
                        div { key: "{dep.name} {dep.version}", class: "license-dep",
                            // The name is the repository link when the crate
                            // declares one; a crate that declares none renders
                            // unlinked rather than as a dead anchor.
                            if let Some(url) = dep.repository.as_deref() {
                                a {
                                    class: "license-dep-name",
                                    href: "{url}",
                                    target: "_blank",
                                    rel: "noopener noreferrer",
                                    "{dep.name}"
                                }
                            } else {
                                span { class: "license-dep-name", "{dep.name}" }
                            }
                            span { class: "license-dep-version", "{dep.version}" }
                            span { class: "license-dep-spdx", "{dep.license}" }
                            if unattributed {
                                span { class: "license-dep-note", {t("licenses.noText")} }
                            }
                        }
                    }
                })}
            }
        }
    }
}
