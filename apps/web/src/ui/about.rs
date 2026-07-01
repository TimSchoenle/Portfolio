//! About section: intro paragraph and the four-focus competency list.

use dioxus::prelude::*;

use crate::i18n::use_i18n;
use crate::sections::{section_id, section_label};
use crate::ui::reveal::Reveal;
use crate::ui::section_header::SectionHeader;

#[component]
pub fn About() -> Element {
    let i18n = use_i18n().i18n;
    let t = move |k: &str| i18n.read().t(k);

    rsx! {
        section { id: section_id("about"), class: "sec",
            Reveal {
                SectionHeader {
                    num: section_label("about"),
                    title: t("about.title"),
                    intro: t("about.intro"),
                }
            }

            div { class: "grid-12",
                div { class: "col-label" }
                div { class: "col-body",
                    Reveal { delay: 100,
                        p { class: "about-body", {t("about.body")} }
                    }
                    Reveal { delay: 200,
                        div { class: "competencies",
                            span { class: "mono text-muted", {t("about.focusLabel")} }
                            div { class: "competency-list",
                                {(1..=4).map(|n| {
                                    let num = format!("{n:02}");
                                    let k = t(&format!("about.focus.f{n}.k"));
                                    let v = t(&format!("about.focus.f{n}.v"));
                                    rsx! {
                                        div { key: "{n}", class: "competency-row",
                                            span { class: "mono text-accent min-w-[28px]", "{num}" }
                                            div { class: "competency-text",
                                                div { class: "competency-k", "{k}" }
                                                div { class: "competency-v", "{v}" }
                                            }
                                            span { class: "competency-line" }
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
}
