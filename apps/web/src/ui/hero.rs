//! Identity hero: eyebrow meta card, oversized name, tagline, scroll cue.
//!
//! Both motion effects are CSS: the name parallax is a scroll-driven animation and the
//! hero → about hand-off is `scroll-snap` (`input.css`). Neither needs the wasm client, and
//! neither takes the wheel away from the reader.

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
    let mut years = current_year() - i32::from(earliest.start.year);
    if current_month() < earliest.start.month {
        years -= 1;
    }
    u32::try_from(years).unwrap_or(0)
}

/// The full name split into hero lines, coloring "ö" and the trailing dot.
///
/// The first whitespace-separated token is the given name; everything after it
/// forms the second line, so a middle name or a multi-word family name is
/// rendered rather than silently dropped.
fn hero_name_lines() -> Element {
    let mut parts = CONFIG.full_name.split_whitespace();
    let first = parts.next().unwrap_or_default();
    let last = parts.collect::<Vec<_>>().join(" ");
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

/// Renders the hero. The years-of-experience figure comes from [`EXPERIENCE`] against the current
/// date, so it advances without an edit here.
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
                // Not a translation key: a tool's name is the same in both languages, and when
                // this was one the two files held identical values that could still drift.
                span { class: "mono text-muted", {CONFIG.headline_tech.join(" · ")} }

                div { class: "hero-meta",
                    div { class: "hero-meta-card",
                        span { class: "mono text-muted", "§ {identity_num}.a" }
                        dl { class: "meta-dl",
                            dt { span { class: "mono text-muted", {t("hero.roleLabel")} } }
                            dd { {t("common.jobTitle")} }
                            dt { span { class: "mono text-muted", {t("hero.locationLabel")} } }
                            dd { {t("common.country")} }
                            dt { span { class: "mono text-muted", {t("hero.yearsLabel")} } }
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

            // The parallax on the name is a CSS scroll-driven animation (`.hero-name` in
            // `input.css`): it runs on the compositor, costs the wasm client nothing, and is off
            // under reduced motion and in browsers without scroll timelines.
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

            a { href: "#{about_id}", class: "scroll-cue", "aria-label": t("hero.scrollLabel"),
                span { class: "mono text-muted", {t("hero.scroll")} }
                span { class: "scroll-cue-line" }
            }
        }
    }
}
