//! The v4 section header: a `grid-12` row with the section number + title in
//! the label column and the intro sentence in the body column.

use dioxus::prelude::*;

#[component]
pub fn SectionHeader(num: String, title: String, intro: String) -> Element {
    rsx! {
        div { class: "section-header grid-12",
            div { class: "col-label",
                span { class: "mono text-muted", "{num}" }
                h2 { class: "section-title", "{title}" }
            }
            div { class: "col-body section-intro", "{intro}" }
        }
    }
}
