//! Identity hero: eyebrow meta card, oversized name, tagline, scroll cue.
//!
//! The wheel-hijack that snaps between the hero and the about section, and the
//! scroll-driven name parallax, are client-only enhancements re-added in the
//! hydration phase.

use dioxus::prelude::*;
use portfolio_data::{CONFIG, EXPERIENCE};

use crate::i18n::use_i18n;
use crate::sections::{section_id, section_label, section_num};
use crate::util::{current_month, current_year};

/// Full years since the earliest experience entry, e.g. 7 -> "7+".
fn years_of_experience() -> u32 {
    let Some(earliest) = EXPERIENCE
        .iter()
        .min_by_key(|e| (e.start.year, e.start.month))
    else {
        return 0;
    };
    let mut years = current_year() - earliest.start.year as i32;
    if current_month() < earliest.start.month {
        years -= 1;
    }
    years.max(0) as u32
}

/// The full name split into hero lines, coloring "ö" and the trailing dot.
fn hero_name_lines() -> Element {
    let mut parts = CONFIG.full_name.split_whitespace();
    let first = parts.next().unwrap_or_default();
    let last = parts.next().unwrap_or_default();
    rsx! {
        span { class: "hero-name-line", "{first}" }
        span { class: "hero-name-line",
            {last.chars().map(|c| {
                if c == 'ö' {
                    rsx! { span { class: "hero-accent-char", "ö" } }
                } else {
                    rsx! { "{c}" }
                }
            })}
            span { class: "hero-accent-char", "." }
        }
    }
}

#[component]
pub fn Hero() -> Element {
    let i18n = use_i18n().i18n;
    let t = move |k: &str| i18n.read().t(k);
    let years = years_of_experience();
    let identity_num = section_num("identity");
    let about_id = section_id("about");

    rsx! {
        section { id: "top", class: "hero",
            div { class: "hero-eyebrow",
                div { class: "bracket-line" }
                span { class: "mono text-accent", {section_label("identity")} }
                span { class: "mono text-muted", {t("hero.eyebrow")} }

                div { class: "hero-meta",
                    div { class: "hero-meta-card",
                        span { class: "mono text-muted", "§ {identity_num}.a" }
                        dl { class: "meta-dl",
                            dt { span { class: "mono text-muted", "ROLE" } }
                            dd { {t("hero.jobTitle")} }
                            dt { span { class: "mono text-muted", "LOC" } }
                            dd { {t("common.country")} }
                            dt { span { class: "mono text-muted", "YRS" } }
                            dd { "{years}+" }
                            dt { span { class: "mono text-muted", {t("hero.statusLabel")} } }
                            dd { class: "text-accent flex items-center gap-2",
                                span { class: "pulse-dot" }
                                {t("hero.status")}
                            }
                        }
                    }
                }
            }

            h1 { class: "hero-name", {hero_name_lines()} }

            div { class: "hero-tagline",
                div { class: "tagline-label",
                    span { class: "mono text-muted", "§ {identity_num}.b" }
                }
                div { class: "tagline-body",
                    p { class: "tagline-main", {t("hero.tagline")} }
                    p { class: "tagline-sub", {t("hero.taglineSub")} }
                }
            }

            a { href: "#{about_id}", class: "scroll-cue", "aria-label": "Scroll",
                span { class: "mono text-muted", "SCROLL" }
                span { class: "scroll-cue-line" }
            }
        }
    }
}
