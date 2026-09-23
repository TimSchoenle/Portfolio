//! Experience accordion: period + years badge, role row with a plus/minus
//! indicator, and a max-height-animated body with bullet arrows and tech tags.

use std::rc::Rc;

use dioxus::prelude::*;
use portfolio_data::{YearMonth, experiences_sorted, format_period_years};

use crate::i18n::use_i18n;
use crate::sections::{section_id, section_label};
use crate::ui::reveal::Reveal;
use crate::ui::section_header::SectionHeader;
use crate::util::{current_month, current_year};

/// Whole years a role lasted, counted in months so that Nov 2018 – Jan 2023 is four years,
/// not five. An ongoing role runs to the current month.
fn whole_years(start: YearMonth, end: Option<YearMonth>) -> u32 {
    let (end_year, end_month) = match end {
        Some(end) => (i32::from(end.year), end.month),
        None => (current_year(), current_month()),
    };
    let months =
        (end_year - i32::from(start.year)) * 12 + i32::from(end_month) - i32::from(start.month);
    months.max(0).unsigned_abs() / 12
}

/// The duration badge, e.g. "4y" or "4 J.", with `under_one_year` for a role shorter than a year.
/// `years_short` carries an `{n}` placeholder for the count.
fn years_badge(years: u32, years_short: &str, under_one_year: &str) -> String {
    if years == 0 {
        under_one_year.to_string()
    } else {
        years_short.replace("{n}", &years.to_string())
    }
}

/// Renders the accordion with the most recent role open. One row is open at a time, and clicking
/// the open one collapses it.
#[component]
pub fn Experience() -> Element {
    let i18n = use_i18n().i18n;
    let t = move |k: &str| i18n.read().t(k);
    // `usize::MAX` = all collapsed.
    let mut open = use_signal(|| 0usize);
    let now = t("common.now");
    let years_short = t("experience.yearsShort");
    let under_one_year = t("experience.underOneYear");
    // Sorted once per mounted section rather than on every render: the ordering
    // is a property of the compile-time data, while `open` above changes on every
    // row the reader expands.
    let entries = use_hook(|| Rc::new(experiences_sorted()));

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
                        {entries.iter().enumerate().map(|(i, e)| {
                            let is_open = open() == i;
                            let row_cls = if is_open { "experience-row open" } else { "experience-row" };
                            let icon_cls = if is_open { "exp-icon open" } else { "exp-icon" };
                            let body_style = if is_open { "max-height: 1000px" } else { "max-height: 0" };
                            let key = |field: &str| format!("experience.entries.{}.{field}", e.id);
                            let period = format_period_years(e.start, e.end, &now);
                            let badge = years_badge(whole_years(e.start, e.end), &years_short, &under_one_year);
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
