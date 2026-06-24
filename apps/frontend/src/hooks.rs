//! Shared hooks and small DOM helpers.

use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use yew::prelude::*;

/// Tracks `window.scrollY`, re-rendering the calling component on change.
/// The value is capped to avoid re-renders once past `cap` pixels.
#[hook]
pub fn use_scroll_y(cap: f64) -> f64 {
    let y = use_state_eq(|| 0.0_f64);
    {
        let y = y.clone();
        use_effect_with((), move |_| {
            let cb = Closure::<dyn Fn()>::wrap(Box::new(move || {
                if let Some(win) = web_sys::window() {
                    let scrolled = win.scroll_y().unwrap_or(0.0);
                    y.set(scrolled.min(cap));
                }
            }));
            let win = web_sys::window().expect("window available");
            win.add_event_listener_with_callback("scroll", cb.as_ref().unchecked_ref())
                .ok();
            move || {
                if let Some(win) = web_sys::window() {
                    win.remove_event_listener_with_callback("scroll", cb.as_ref().unchecked_ref())
                        .ok();
                }
                drop(cb);
            }
        });
    }
    *y
}

/// Smooth-scrolls to the element with the given id (CSS `scroll-behavior`
/// makes this smooth).
pub fn scroll_to(id: &str) {
    if let Some(el) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id(id))
    {
        el.scroll_into_view();
    }
}

/// Scrolls to a section after the next render, for use right after a route
/// change to the home page.
pub fn scroll_to_soon(id: &'static str) {
    wasm_bindgen_futures::spawn_local(async move {
        gloo_timers::future::TimeoutFuture::new(80).await;
        scroll_to(id);
    });
}
