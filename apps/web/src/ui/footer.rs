//! Site footer: identity, social/legal/meta columns, sign-off line.

use dioxus::prelude::*;
use portfolio_data::CONFIG;

use crate::i18n::use_i18n;
use crate::routes::Route;
use crate::util::current_year;

#[component]
pub fn Footer() -> Element {
    let i18n = use_i18n().i18n;
    let t = move |k: &str| i18n.read().t(k);

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
                        Link { to: Route::Imprint {}, {t("footer.imprint")} }
                        Link { to: Route::Privacy {}, {t("footer.privacy")} }
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
