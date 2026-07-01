//! Reveal-on-scroll wrapper from the v4 design.
//!
//! Renders its children **visible by default** so the server-side render and the
//! no-JS view show the content (good for SEO and progressive enhancement). The
//! IntersectionObserver-driven fade/slide entrance is re-added as a client-only
//! enhancement in the hydration phase.

use dioxus::prelude::*;

#[component]
pub fn Reveal(#[props(default = 0)] delay: u32, children: Element) -> Element {
    let _ = delay;
    rsx! {
        div { class: "reveal", {children} }
    }
}
