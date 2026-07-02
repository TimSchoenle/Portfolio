//! Experience accordion: period + years badge, role row with a plus/minus
//! indicator, and a max-height-animated body with bullet arrows and tech tags.

use dioxus::prelude::*;
use portfolio_data::{experiences_sorted, format_period_years};

use crate::i18n::use_i18n;
use crate::sections::{section_id, section_label};
use crate::ui::reveal::Reveal;
use crate::ui::section_header::SectionHeader;
use crate::util::current_year;

/// "7y" badge; sub-year stints render as "<1y".
fn years_badge(start: u16, end: Option<u16>) -> String {
    let end = end.unwrap_or_else(|| current_year() as u16);
    let years = end.saturating_sub(start);
    if years == 0 {
        "<1y".to_string()
    } else {
        format!("{years}y")
    }
}

#[component]
pub fn Experience() -> Element {
    let i18n = use_i18n().i18n;
    let t = move |k: &str| i18n.read().t(k);
    // `usize::MAX` = all collapsed.
    let mut open = use_signal(|| 0usize);
    let now = t("common.now");

    rsx! {
        section { id: section_id("experience"), class: "sec",
            Reveal {
                SectionHeader {
                    num: section_label("experience"),
                    title: t("experience.title"),
                    intro: t("experience.intro"),
                }
            }

            div { class: "grid-12",
                div { class: "col-label" }
                div { class: "col-body",
                    div { class: "experience-list",
                        {experiences_sorted().into_iter().enumerate().map(|(i, e)| {
                            let is_open = open() == i;
                            let row_cls = if is_open { "experience-row open" } else { "experience-row" };
                            let icon_cls = if is_open { "exp-icon open" } else { "exp-icon" };
                            let body_style = if is_open { "max-height: 1000px" } else { "max-height: 0" };
                            let key = |field: &str| format!("experience.entries.{}.{field}", e.id);
                            let period = format_period_years(e.start, e.end, &now);
                            let badge = years_badge(e.start.year, e.end.map(|d| d.year));
                            let role = t(&key("role"));
                            let sub = format!("{} · {}", t(&key("org")), e.location);
                            let bullet_count = e.bullet_count;
                            let tech = e.tech;
                            let entry_id = e.id;
                            rsx! {
                                div { key: "{entry_id}", class: "{row_cls}",
                                    button {
                                        class: "experience-head",
                                        "aria-expanded": "{is_open}",
                                        onclick: move |_| open.set(if is_open { usize::MAX } else { i }),
                                        div { class: "exp-period",
                                            span { class: "mono text-muted", "{period}" }
                                            span { class: "mono text-accent mt-1", "{badge}" }
                                        }
                                        div { class: "exp-title-col",
                                            div { class: "exp-role", "{role}" }
                                            div { class: "exp-sub",
                                                span { class: "mono text-muted", "{sub}" }
                                            }
                                        }
                                        div { class: "exp-indicator",
                                            span { class: "{icon_cls}",
                                                span {}
                                                span {}
                                            }
                                        }
                                    }
                                    div { class: "experience-body", style: "{body_style}",
                                        div { class: "experience-body-inner",
                                            ul { class: "exp-bullets",
                                                {(1..=bullet_count).map(|n| {
                                                    let bullet = t(&key(&format!("bullets.b{n}")));
                                                    rsx! {
                                                        li { key: "{n}",
                                                            span { class: "bullet-arrow", "›" }
                                                            "{bullet}"
                                                        }
                                                    }
                                                })}
                                            }
                                            div { class: "exp-tech",
                                                {tech.iter().map(|tag| rsx! {
                                                    span { key: "{tag}", class: "tech-tag", "{tag}" }
                                                })}
                                            }
                                        }
                                    }
                                }
                            }
                        })}
                    }
                }
            }
        }
    }
}
