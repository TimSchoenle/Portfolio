//! Stack section: filter chips + radar tooltip in the label column, the radar
//! and the per-category chip list side by side.

use dioxus::prelude::*;
use portfolio_data::{Quadrant, Skill, matrix_skills};

use crate::i18n::use_i18n;
use crate::sections::{section_id, section_label};
use crate::ui::radar::Radar;
use crate::ui::reveal::Reveal;
use crate::ui::section_header::SectionHeader;

#[component]
pub fn Skills() -> Element {
    let i18n = use_i18n().i18n;
    let t = move |k: &str| i18n.read().t(k);
    let mut active = use_signal(|| None::<Quadrant>);
    let mut hovered = use_signal(|| None::<Skill>);

    let all = matrix_skills();
    let confidence_pct = move |s: &Skill| {
        format!(
            "{} {}%",
            i18n.read().t("skills.confidenceLabel"),
            (s.confidence * 100.0).round()
        )
    };

    let all_cls = if active().is_none() {
        "filter-chip active"
    } else {
        "filter-chip"
    };

    rsx! {
        section { id: section_id("stack"), class: "sec",
            Reveal {
                SectionHeader {
                    num: section_label("stack"),
                    title: t("skills.title"),
                    intro: t("skills.intro"),
                }
            }

            div { class: "grid-12",
                div { class: "col-label",
                    div { class: "filter-controls",
                        span { class: "mono text-muted", {t("common.filter")} }
                        button { class: "{all_cls}", onclick: move |_| active.set(None),
                            span { class: "mono", {t("common.all")} }
                        }
                        {Quadrant::all().into_iter().map(|q| {
                            let is_active = active() == Some(q);
                            let cls = if is_active { "filter-chip active" } else { "filter-chip" };
                            let label = t(q.i18n_key());
                            rsx! {
                                button {
                                    key: "{label}",
                                    class: "{cls}",
                                    onclick: move |_| active.set(if is_active { None } else { Some(q) }),
                                    span { class: "mono", "{label}" }
                                }
                            }
                        })}
                        div { class: "radar-tooltip", "aria-live": "polite",
                            if let Some(s) = hovered() {
                                div { class: "text-fg text-base font-semibold", "{s.name}" }
                                span { class: "mono text-muted", {confidence_pct(&s)} }
                            } else {
                                span { class: "mono text-muted opacity-50", {t("skills.hoverHint")} }
                            }
                        }
                    }
                }

                div { class: "col-body stack-split",
                    Reveal { delay: 120,
                        Radar { active: active(), on_hover: move |s| hovered.set(s) }
                    }
                    Reveal { delay: 200,
                        div { class: "stack-list",
                            {Quadrant::all().into_iter().enumerate().map(|(i, q)| {
                                let dim = active().is_some() && active() != Some(q);
                                let items: Vec<Skill> = all.iter().filter(|s| s.quadrant == q).copied().collect();
                                let cat_cls = if dim { "stack-cat dim" } else { "stack-cat" };
                                let num = format!("{:02}", i + 1);
                                let name = t(q.i18n_key());
                                let count = items.len();
                                rsx! {
                                    div {
                                        key: "{name}",
                                        class: "{cat_cls}",
                                        onmouseenter: move |_| active.set(Some(q)),
                                        onmouseleave: move |_| active.set(None),
                                        div { class: "stack-cat-head",
                                            span { class: "mono text-muted", "{num}" }
                                            span { class: "mono text-fg", "{name}" }
                                            span { class: "stack-cat-rule" }
                                            span { class: "mono text-muted", "{count}" }
                                        }
                                        div { class: "stack-chips",
                                            {items.iter().map(|s| {
                                                let title = confidence_pct(s);
                                                let level = s.level();
                                                rsx! {
                                                    span { key: "{s.name}", class: "stack-chip", title: "{title}",
                                                        "{s.name}"
                                                        span { class: "chip-bar",
                                                            {(1..=5).map(|n| {
                                                                let seg = if n <= level { "chip-bar-seg on" } else { "chip-bar-seg" };
                                                                rsx! { span { key: "{n}", class: "{seg}" } }
                                                            })}
                                                        }
                                                    }
                                                }
                                            })}
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
}
