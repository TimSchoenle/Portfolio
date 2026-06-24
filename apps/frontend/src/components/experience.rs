//! Experience accordion from the v4 design: period + years badge, role row
//! with plus/minus indicator, max-height-animated body with bullet arrows
//! and a tech-tag column.

use i18nrs::yew::use_translation;
use portfolio_data::{EXPERIENCE, format_period_years};
use yew::prelude::*;

use super::reveal::Reveal;
use super::section_header::SectionHeader;

/// "7y" badge; sub-year stints render as "<1y".
fn years_badge(start: u16, end: Option<u16>) -> String {
    let end = end.unwrap_or_else(|| js_sys::Date::new_0().get_full_year() as u16);
    let years = end.saturating_sub(start);
    if years == 0 {
        "<1y".to_string()
    } else {
        format!("{years}y")
    }
}

#[function_component(Experience)]
pub fn experience_comp() -> Html {
    let (i18n, _) = use_translation();
    let open = use_state(|| 0_usize);

    html! {
        <section id="s4" class="sec">
            <Reveal>
                <SectionHeader num="§ 04 — experience"
                    title={ i18n.t("experience.title") }
                    intro={ i18n.t("experience.intro") } />
            </Reveal>

            <div class="grid-12">
                <div class="col-label" />
                <div class="col-body">
                    <div class="experience-list">
                        { for EXPERIENCE.iter().enumerate().map(|(i, e)| {
                            let is_open = *open == i;
                            let toggle = {
                                let open = open.clone();
                                Callback::from(move |_| open.set(if is_open { usize::MAX } else { i }))
                            };
                            let key = |field: &str| format!("experience.entries.{}.{field}", e.id);
                            let period = format_period_years(e.start, e.end, &i18n.t("common.now"));

                            html!{
                                <div class={classes!("experience-row", is_open.then_some("open"))}>
                                    <button onclick={toggle} class="experience-head" aria-expanded={is_open.to_string()}>
                                        <div class="exp-period">
                                            <span class="mono text-muted">{period}</span>
                                            <span class="mono text-accent mt-1">{ years_badge(e.start.year, e.end.map(|d| d.year)) }</span>
                                        </div>
                                        <div class="exp-title-col">
                                            <div class="exp-role">{ i18n.t(&key("role")) }</div>
                                            <div class="exp-sub">
                                                <span class="mono text-muted">
                                                    { format!("{} · {}", i18n.t(&key("org")), e.location) }
                                                </span>
                                            </div>
                                        </div>
                                        <div class="exp-indicator">
                                            <span class={classes!("exp-icon", is_open.then_some("open"))}>
                                                <span /><span />
                                            </span>
                                        </div>
                                    </button>
                                    <div class="experience-body"
                                         style={ if is_open { "max-height: 1000px" } else { "max-height: 0" } }>
                                        <div class="experience-body-inner">
                                            <ul class="exp-bullets">
                                                { for (1..=e.bullet_count).map(|n| html!{
                                                    <li>
                                                        <span class="bullet-arrow">{"›"}</span>
                                                        { i18n.t(&key(&format!("bullets.b{n}"))) }
                                                    </li>
                                                })}
                                            </ul>
                                            <div class="exp-tech">
                                                { for e.tech.iter().map(|t| html!{
                                                    <span class="tech-tag">{*t}</span>
                                                })}
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            }
                        })}
                    </div>
                </div>
            </div>
        </section>
    }
}
