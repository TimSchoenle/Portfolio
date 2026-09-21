//! Site footer: identity, social/legal/meta columns, sign-off line.

use dioxus::prelude::*;
use portfolio_data::CONFIG;

use terrace_legal_dioxus::legal_title;
use terrace_legal_model::LegalIndexEntry;

use crate::i18n::use_i18n;
use crate::legal::{SiteText, is_hosted, use_legal_entries};
use crate::routes::Route;
use crate::util::current_year;

/// Renders the footer. The copyright year is read from the clock at render time.
///
/// The legal column lists every document the operator published, in their order and under their
/// titles, followed by the site's own licence inventory and colophon.
#[component]
pub fn Footer() -> Element {
    let i18n = use_i18n().i18n;
    let t = move |k: &str| i18n.read().t(k);
    let lang = i18n.read().get_current_language().to_string();
    let legal = use_legal_entries(&lang);
    let text = SiteText::new(i18n);

    let url_display = CONFIG
        .url
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    let year = current_year();
    let identity = format!("{} · PORTFOLIO", CONFIG.full_name.to_uppercase());

    rsx! {
        footer { class: "site-footer",
            div { class: "footer-top",
                div {
                    span { class: "mono text-fg", "{identity}" }
                    div { class: "footer-tagline", {t("footer.credit")} }
                }
                div { class: "footer-cols",
                    div { class: "footer-col",
                        span { class: "mono text-muted", "SOCIAL" }
                        a { href: CONFIG.github, target: "_blank", rel: "noreferrer", "GitHub" }
                        a { href: CONFIG.linkedin, target: "_blank", rel: "noreferrer", "LinkedIn" }
                        a { href: CONFIG.url, target: "_blank", rel: "noreferrer", "{url_display}" }
                    }
                    div { class: "footer-col",
                        span { class: "mono text-muted", "LEGAL" }
                        for entry in legal.iter() {
                            {legal_link(entry, &legal_title(&text, entry))}
                        }
                        Link { to: Route::Licenses {}, {t("footer.licenses")} }
                        a { href: CONFIG.repository, target: "_blank", rel: "noreferrer", {t("footer.colophon")} }
                    }
                    div { class: "footer-col",
                        span { class: "mono text-muted", "META" }
                        span { "© {year}" }
                        span { {t("common.country")} }
                        span { {t("footer.built")} }
                    }
                }
            }
            div { class: "footer-bottom",
                span { class: "mono text-muted", "— END OF TRANSMISSION —" }
            }
        }
    }
}

/// One legal document's footer link: through the router for a document this site hosts, and as
/// a plain anchor that leaves the site for one hosted elsewhere.
fn legal_link(entry: &LegalIndexEntry, title: &str) -> Element {
    match entry.url.as_deref().filter(|_| !is_hosted(entry)) {
        Some(url) => rsx! {
            a { key: "{entry.slug}", href: "{url}", target: "_blank", rel: "noopener noreferrer", "{title}" }
        },
        None => rsx! {
            Link { key: "{entry.slug}", to: Route::LegalDocument { slug: entry.slug.clone() }, "{title}" }
        },
    }
}
