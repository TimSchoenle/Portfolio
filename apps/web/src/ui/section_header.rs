//! The heading every home-page section opens with.

use dioxus::prelude::*;

/// A `grid-12` row: the mono section label and the title in the label column, the intro
/// sentence in the body column.
///
/// `num` is a whole rendered label such as `"§ 02 — about"`, from
/// [`crate::sections::section_label`], rather than a number this component formats. Section
/// numbering is one derivation and it lives there.
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
