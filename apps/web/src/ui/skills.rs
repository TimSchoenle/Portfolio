//! Stack section: filter chips + radar tooltip in the label column, the radar
//! and the per-category chip list side by side.

use std::sync::LazyLock;

use dioxus::prelude::*;
use i18nrs::I18n;
use portfolio_data::{Quadrant, Skill, matrix_skills};

use crate::i18n::use_i18n;
use crate::sections::{section_id, section_label};
use crate::ui::radar::Radar;
use crate::ui::reveal::Reveal;
use crate::ui::section_header::SectionHeader;

/// The skills the chip list shows, a pure function of compile-time data built once per process.
static MATRIX: LazyLock<Vec<Skill>> = LazyLock::new(matrix_skills);

/// Renders the stack section, with the quadrant chips and the radar sharing one selection.
#[component]
pub fn Skills() -> Element {
    let i18n = use_i18n().i18n;
    let t = move |k: &str| i18n.read().t(k);
    let active = use_signal(|| None::<Quadrant>);
    let mut hovered = use_signal(|| None::<Skill>);

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
                        QuadrantChips { active }
                        RadarReadout { hovered }
                    }
                }

                div { class: "col-body stack-split",
                    Reveal { delay: 120,
                        Radar { active: active(), on_hover: move |s| hovered.set(s) }
                    }
                    Reveal { delay: 200,
                        div { class: "stack-list",
                            {Quadrant::all().into_iter().enumerate().map(|(i, q)| rsx! {
                                StackCategory { key: "{q.i18n_key()}", index: i, quadrant: q, active }
                            })}
                        }
                    }
                }
            }
        }
    }
}

/// "All" and one chip per quadrant. Choosing the active quadrant again clears the selection.
#[component]
fn QuadrantChips(active: Signal<Option<Quadrant>>) -> Element {
    let i18n = use_i18n().i18n;
    let t = move |k: &str| i18n.read().t(k);
    let all_cls = if active().is_none() {
        "filter-chip active"
    } else {
        "filter-chip"
    };
    rsx! {
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
    }
}

/// The live region naming the skill under the cursor or focus, or a hint when there is none.
#[component]
fn RadarReadout(hovered: Signal<Option<Skill>>) -> Element {
    let i18n = use_i18n().i18n;
    rsx! {
        div { class: "radar-tooltip", "aria-live": "polite",
            if let Some(s) = hovered() {
                div { class: "text-fg text-base font-semibold", "{s.name}" }
                span { class: "mono text-muted", {confidence_label(i18n, &s)} }
            } else {
                span { class: "mono text-muted opacity-50", {i18n.read().t("skills.hoverHint")} }
            }
        }
    }
}

/// One quadrant's skills as chips under a numbered heading. Hovering it makes `quadrant` the
/// active one; it dims while another quadrant is active.
#[component]
fn StackCategory(index: usize, quadrant: Quadrant, active: Signal<Option<Quadrant>>) -> Element {
    let i18n = use_i18n().i18n;
    let dim = active().is_some() && active() != Some(quadrant);
    let items: Vec<Skill> = MATRIX
        .iter()
        .filter(|s| s.quadrant == quadrant)
        .copied()
        .collect();
    let cat_cls = if dim { "stack-cat dim" } else { "stack-cat" };
    let num = format!("{:02}", index + 1);
    let name = i18n.read().t(quadrant.i18n_key());
    let count = items.len();
    rsx! {
        div {
            class: "{cat_cls}",
            onmouseenter: move |_| active.set(Some(quadrant)),
            onmouseleave: move |_| active.set(None),
            div { class: "stack-cat-head",
                span { class: "mono text-muted", "{num}" }
                span { class: "mono text-fg", "{name}" }
                span { class: "stack-cat-rule" }
                span { class: "mono text-muted", "{count}" }
            }
            div { class: "stack-chips",
                {items.iter().map(|s| {
                    let title = confidence_label(i18n, s);
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
}

/// The localized confidence readout for `skill`, such as `Depth 85%`.
fn confidence_label(i18n: Signal<I18n>, skill: &Skill) -> String {
    format!(
        "{} {}%",
        i18n.read().t("skills.confidenceLabel"),
        (skill.confidence * 100.0).round()
    )
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::*;
    use crate::ui::test_render::{render, text};

    #[test]
    fn renders_one_category_per_quadrant_holding_every_matrix_skill() {
        let html = render("en", || rsx! { Skills {} });
        assert!(html.contains(&format!(r#"id="{}""#, section_id("stack"))));
        assert_eq!(
            html.matches(r#"class="stack-cat""#).count(),
            Quadrant::all().len()
        );
        assert_eq!(html.matches(r#"class="stack-chip""#).count(), MATRIX.len());
        for skill in MATRIX.iter() {
            assert!(html.contains(skill.name), "{} is missing", skill.name);
        }
    }

    #[test]
    fn opens_with_no_quadrant_selected_and_the_hover_hint_showing() {
        let html = render("en", || rsx! { Skills {} });
        // "All" is the only active chip, no category is dimmed, and the readout shows the hint.
        assert_eq!(html.matches("filter-chip active").count(), 1);
        assert!(!html.contains("stack-cat dim"));
        assert!(html.contains(&text("en", "skills.hoverHint")));
    }

    #[test]
    fn labels_each_chip_with_its_skill_s_localized_confidence() {
        for lang in ["en", "de"] {
            let html = render(lang, || rsx! { Skills {} });
            let skill = MATRIX[0];
            let expected = format!(
                "{} {}%",
                text(lang, "skills.confidenceLabel"),
                (skill.confidence * 100.0).round()
            );
            assert!(html.contains(&format!(r#"title="{expected}""#)), "{lang}");
            assert!(html.contains(&text(lang, "skills.title")), "{lang}");
        }
    }

    #[test]
    fn fills_each_level_bar_with_as_many_segments_as_the_skill_s_level() {
        let html = render("en", || rsx! { Skills {} });
        let lit: usize = MATRIX.iter().map(|s| usize::from(s.level())).sum();
        assert_eq!(html.matches(r#"class="chip-bar-seg on""#).count(), lit);
        assert_eq!(
            html.matches(r#"class="chip-bar-seg"#).count(),
            MATRIX.len() * 5
        );
    }
}
