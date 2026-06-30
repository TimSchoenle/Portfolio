//! Shared hooks and small DOM helpers.

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::convert::FromWasmAbi;
use wasm_bindgen::{JsCast, JsValue};
use yew::prelude::*;

/// Registers `handler` as a `window` event listener and removes it on cleanup.
///
/// `E` is the concrete `web_sys` event type the handler accepts (use
/// `web_sys::Event` when the payload is unused). The listener is re-registered
/// whenever `deps` changes — pass `()` to register once for the component's
/// lifetime, or a value the handler closes over so it never reads stale state.
#[hook]
pub fn use_window_event<E, F, D>(event: &'static str, deps: D, handler: F)
where
    E: FromWasmAbi + 'static,
    F: Fn(E) + 'static,
    D: PartialEq + 'static,
{
    use_effect_with(deps, move |_| {
        let cb = Closure::<dyn Fn(E)>::wrap(Box::new(handler));
        let win = web_sys::window().expect("window available");
        win.add_event_listener_with_callback(event, cb.as_ref().unchecked_ref())
            .ok();
        move || {
            if let Some(win) = web_sys::window() {
                win.remove_event_listener_with_callback(event, cb.as_ref().unchecked_ref())
                    .ok();
            }
            drop(cb);
        }
    });
}

/// Tracks `window.scrollY`, re-rendering the calling component on change.
/// The value is capped at `cap` pixels to stop re-renders once well past it.
#[hook]
pub fn use_scroll_y(cap: f64) -> f64 {
    let y = use_state_eq(|| 0.0_f64);
    {
        let y = y.clone();
        use_window_event("scroll", (), move |_: web_sys::Event| {
            if let Some(win) = web_sys::window() {
                y.set(win.scroll_y().unwrap_or(0.0).min(cap));
            }
        });
    }
    *y
}

/// Returns `true` once the referenced element first scrolls into view (at
/// `threshold` visibility) and stays `true` afterwards. The underlying
/// `IntersectionObserver` disconnects after the first intersection.
#[hook]
pub fn use_in_view(node: &NodeRef, threshold: f64) -> bool {
    let visible = use_state(|| false);
    {
        let visible = visible.clone();
        let node = node.clone();
        use_effect_with((), move |_| {
            let observer: Rc<RefCell<Option<web_sys::IntersectionObserver>>> =
                Rc::new(RefCell::new(None));
            let observer_in_cb = observer.clone();
            let cb =
                Closure::<dyn Fn(js_sys::Array)>::wrap(Box::new(move |entries: js_sys::Array| {
                    let entry: web_sys::IntersectionObserverEntry = entries.get(0).unchecked_into();
                    if entry.is_intersecting() {
                        if let Some(obs) = observer_in_cb.borrow_mut().take() {
                            obs.disconnect();
                        }
                        visible.set(true);
                    }
                }));

            let init = web_sys::IntersectionObserverInit::new();
            init.set_threshold(&JsValue::from_f64(threshold));
            if let (Some(el), Ok(obs)) = (
                node.cast::<web_sys::Element>(),
                web_sys::IntersectionObserver::new_with_options(cb.as_ref().unchecked_ref(), &init),
            ) {
                obs.observe(&el);
                *observer.borrow_mut() = Some(obs);
            }

            move || {
                if let Some(obs) = observer.borrow_mut().take() {
                    obs.disconnect();
                }
                drop(cb);
            }
        });
    }
    *visible
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
pub fn scroll_to_soon(id: String) {
    wasm_bindgen_futures::spawn_local(async move {
        gloo_timers::future::TimeoutFuture::new(80).await;
        scroll_to(&id);
    });
}
