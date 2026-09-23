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

/// How many months a role lasted, counting its first and last month both, the way a resume or
/// LinkedIn states a tenure: Oct 2021 – Jan 2023 is 16 months. An ongoing role runs to the
/// current month.
fn tenure_months(start: YearMonth, end: Option<YearMonth>) -> u32 {
    let (end_year, end_month) = match end {
        Some(end) => (i32::from(end.year), end.month),
        None => (current_year(), current_month()),
    };
    let months =
        (end_year - i32::from(start.year)) * 12 + i32::from(end_month) - i32::from(start.month) + 1;
    months.max(1).unsigned_abs()
}

/// The duration badge, e.g. "4y 3m", "2y" or "5m" ("4 J. 3 M." in German).
///
/// `years_unit` and `months_unit` each carry an `{n}` placeholder for the count, and a zero part
/// is left out, so a whole number of years reads as years alone.
fn duration_badge(months: u32, years_unit: &str, months_unit: &str) -> String {
    let (years, rest) = (months / 12, months % 12);
    let years_part = (years > 0).then(|| years_unit.replace("{n}", &years.to_string()));
    let months_part = (rest > 0).then(|| months_unit.replace("{n}", &rest.to_string()));
    [years_part, months_part]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" ")
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
    let years_unit = t("experience.yearsShort");
    let months_unit = t("experience.monthsShort");
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
                            let badge = duration_badge(tenure_months(e.start, e.end), &years_unit, &months_unit);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn ym(year: u16, month: u8) -> YearMonth {
        YearMonth { year, month }
    }

    #[test]
    fn tenure_counts_the_first_and_last_month() {
        assert_eq!(tenure_months(ym(2021, 10), Some(ym(2023, 1))), 16);
        assert_eq!(tenure_months(ym(2018, 11), Some(ym(2023, 1))), 51);
        assert_eq!(tenure_months(ym(2026, 3), Some(ym(2026, 3))), 1);
    }

    #[test]
    fn the_badge_shows_years_and_months_and_drops_a_zero_part() {
        assert_eq!(duration_badge(16, "{n}y", "{n}m"), "1y 4m");
        assert_eq!(duration_badge(24, "{n}y", "{n}m"), "2y");
        assert_eq!(duration_badge(5, "{n}y", "{n}m"), "5m");
        assert_eq!(duration_badge(51, "{n} J.", "{n} M."), "4 J. 3 M.");
    }
}
