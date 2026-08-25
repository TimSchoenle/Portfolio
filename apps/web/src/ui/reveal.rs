//! Reveal-on-scroll wrapper from the v4 design.
//!
//! Renders visible by default so the server-side render and the no-JS view show
//! the content (good for SEO and progressive enhancement). After hydration,
//! blocks still below the fold are hidden and then fade/slide in when they
//! scroll into view; blocks already on screen — and everything under reduced
//! motion — stay visible with no animation, so nothing above the fold flashes.

use dioxus::prelude::*;

/// Wraps `children` so they fade in when scrolled to, `delay` milliseconds after the block
/// above them.
///
/// Stagger a row by giving each item a larger `delay`. Nothing is delayed on the server or under
/// reduced motion, where the children stay visible.
#[component]
pub fn Reveal(#[props(default = 0)] delay: u32, children: Element) -> Element {
    // Empty on the server and on the first client render (identical to the SSR
    // HTML, so hydration matches). The client may switch it to `reveal-pre`
    // (hidden) and then `reveal-in` (animating in) for below-the-fold blocks.
    let phase = use_signal(String::new);

    // The observed DOM node, captured from `onmounted` (client only).
    #[cfg(feature = "web")]
    let mut el = use_signal(|| None::<web_sys::Element>);

    #[cfg(feature = "web")]
    {
        use crate::hooks::{InViewGuard, element_in_view, observe_once, prefers_reduced_motion};
        use std::cell::RefCell;
        use std::rc::Rc;

        // Kept for the component's lifetime; the observer disconnects on drop.
        let guard: Rc<RefCell<Option<InViewGuard>>> = use_hook(|| Rc::new(RefCell::new(None)));
        let mut phase = phase;

        use_effect(move || {
            let Some(el) = el() else { return };
            // Reduced motion or already on screen: leave fully visible, no
            // animation and (for above-the-fold blocks) no flash.
            if prefers_reduced_motion() || element_in_view(&el, 0.9) {
                return;
            }
            phase.set("reveal-pre".to_string());
            *guard.borrow_mut() = observe_once(&el, 0.12, move || {
                phase.set("reveal-in".to_string());
            });
        });
    }

    rsx! {
        div {
            class: "reveal {phase}",
            style: "transition-delay: {delay}ms",
            onmounted: move |_e| {
                #[cfg(feature = "web")]
                {
                    use dioxus::web::WebEventExt;
                    if let Some(node) = _e.try_as_web_event() {
                        el.set(Some(node));
                    }
                }
            },
            {children}
        }
    }
}
