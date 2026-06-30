//! Stack section from the v4 design: filter chips + radar tooltip in the
//! label column, the radar and the per-category chip list side by side.
//! Hovering a category dims the others; chips carry 5-segment bars.

use i18nrs::yew::use_translation;
use portfolio_data::{Quadrant, Skill, matrix_skills};
use yew::prelude::*;

use super::radar::Radar;
use super::reveal::Reveal;
use super::section_header::SectionHeader;
use super::sections::{section_id, section_label};

#[function_component(Skills)]
pub fn skills_comp() -> Html {
    let (i18n, _) = use_translation();
    let active = use_state(|| None::<Quadrant>);
    let hovered = use_state(|| None::<Skill>);

    let on_hover = {
        let hovered = hovered.clone();
        Callback::from(move |s: Option<Skill>| hovered.set(s))
    };

    let all = matrix_skills();
    let confidence_pct = |s: &Skill| {
        format!(
            "{} {}%",
            i18n.t("skills.confidenceLabel"),
            (s.confidence * 100.0).round()
        )
    };

    html! {
        <section id={ section_id("stack") } class="sec">
            <Reveal>
                <SectionHeader num={ section_label("stack") }
                    title={ i18n.t("skills.title") }
                    intro={ i18n.t("skills.intro") } />
            </Reveal>

            <div class="grid-12">
                <div class="col-label">
                    <div class="filter-controls">
                        <span class="mono text-muted">{ i18n.t("common.filter") }</span>
                        <button class={classes!("filter-chip", active.is_none().then_some("active"))}
                                onclick={{
                                    let active = active.clone();
                                    Callback::from(move |_| active.set(None))
                                }}>
                            <span class="mono">{ i18n.t("common.all") }</span>
                        </button>
                        { for Quadrant::all().iter().map(|q| {
                            let q = *q;
                            let is_active = *active == Some(q);
                            let onclick = {
                                let active = active.clone();
                                Callback::from(move |_| {
                                    active.set(if is_active { None } else { Some(q) })
                                })
                            };
                            html!{
                                <button class={classes!("filter-chip", is_active.then_some("active"))} {onclick}>
                                    <span class="mono">{ i18n.t(q.i18n_key()) }</span>
                                </button>
                            }
                        })}
                        <div class="radar-tooltip" aria-live="polite">
                            if let Some(s) = *hovered {
                                <div class="text-fg text-base font-semibold">{s.name}</div>
                                <span class="mono text-muted">{ confidence_pct(&s) }</span>
                            } else {
                                <span class="mono text-muted opacity-50">{ i18n.t("skills.hoverHint") }</span>
                            }
                        </div>
                    </div>
                </div>

                <div class="col-body stack-split">
                    <Reveal delay={120}>
                        <Radar active={*active} on_hover={on_hover} />
                    </Reveal>
                    <Reveal delay={200}>
                        <div class="stack-list">
                            { for Quadrant::all().iter().enumerate().map(|(i, q)| {
                                let q = *q;
                                let dim = active.is_some() && *active != Some(q);
                                let items: Vec<Skill> = all.iter().filter(|s| s.quadrant == q).copied().collect();
                                let enter = {
                                    let active = active.clone();
                                    Callback::from(move |_: MouseEvent| active.set(Some(q)))
                                };
                                let leave = {
                                    let active = active.clone();
                                    Callback::from(move |_: MouseEvent| active.set(None))
                                };
                                html!{
                                    <div class={classes!("stack-cat", dim.then_some("dim"))}
                                         onmouseenter={enter} onmouseleave={leave}>
                                        <div class="stack-cat-head">
                                            <span class="mono text-muted">{ format!("{:02}", i + 1) }</span>
                                            <span class="mono text-fg">{ i18n.t(q.i18n_key()) }</span>
                                            <span class="stack-cat-rule" />
                                            <span class="mono text-muted">{ items.len() }</span>
                                        </div>
                                        <div class="stack-chips">
                                            { for items.iter().map(|s| html!{
                                                <span class="stack-chip" title={ confidence_pct(s) }>
                                                    {s.name}
                                                    <span class="chip-bar">
                                                        { for (1..=5).map(|n| html!{
                                                            <span class={classes!("chip-bar-seg", (n <= s.level()).then_some("on"))} />
                                                        })}
                                                    </span>
                                                </span>
                                            })}
                                        </div>
                                    </div>
                                }
                            })}
                        </div>
                    </Reveal>
                </div>
            </div>
        </section>
    }
}
