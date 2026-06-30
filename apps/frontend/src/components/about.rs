use i18nrs::yew::use_translation;
use yew::prelude::*;

use super::reveal::Reveal;
use super::section_header::SectionHeader;
use super::sections::{section_id, section_label};

#[function_component(About)]
pub fn about() -> Html {
    let (i18n, _) = use_translation();

    html! {
        <section id={ section_id("about") } class="sec">
            <Reveal>
                <SectionHeader num={ section_label("about") }
                    title={ i18n.t("about.title") }
                    intro={ i18n.t("about.intro") } />
            </Reveal>

            <div class="grid-12">
                <div class="col-label" />
                <div class="col-body">
                    <Reveal delay={100}>
                        <p class="about-body">{ i18n.t("about.body") }</p>
                    </Reveal>
                    <Reveal delay={200}>
                        <div class="competencies">
                            <span class="mono text-muted">{ i18n.t("about.focusLabel") }</span>
                            <div class="competency-list">
                                { for (1..=4).map(|n| html!{
                                    <div class="competency-row">
                                        <span class="mono text-accent min-w-[28px]">{ format!("{n:02}") }</span>
                                        <div class="competency-text">
                                            <div class="competency-k">{ i18n.t(&format!("about.focus.f{n}.k")) }</div>
                                            <div class="competency-v">{ i18n.t(&format!("about.focus.f{n}.v")) }</div>
                                        </div>
                                        <span class="competency-line" />
                                    </div>
                                })}
                            </div>
                        </div>
                    </Reveal>
                </div>
            </div>
        </section>
    }
}
