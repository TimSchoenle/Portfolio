use dioxus::prelude::*;

#[component]
pub fn NotFound(segments: Vec<String>) -> Element {
    let _ = segments;
    rsx! {
        section { class: "notfound", "404 (port in progress)" }
    }
}
