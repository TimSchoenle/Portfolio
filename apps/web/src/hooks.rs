//! Client-side interaction primitives (scroll offset, viewport queries, one-shot
//! in-view detection, global listeners) and small DOM helpers.
//!
//! The whole module is `web`-only: it compiles solely into the wasm client,
//! never the SSR server binary. Components call it from inside their own
//! `#[cfg(feature = "web")]` effects and keep their reactive state at the
//! "final, fully visible" value on the server, so the SSR HTML (and the no-JS
//! view) always shows complete content and hydration never mismatches — the
//! entrance animations are armed only after the client has hydrated.

use std::cell::RefCell;
use std::rc::Rc;

use web_sys::wasm_bindgen::closure::Closure;
use web_sys::wasm_bindgen::{JsCast, JsValue};

/// True when the user has requested reduced motion at the OS/browser level. The
/// scroll-driven hero motion and the reveal / type-in entrance animations are
/// skipped when this holds.
pub fn prefers_reduced_motion() -> bool {
    web_sys::window()
        .and_then(|w| {
            w.match_media("(prefers-reduced-motion: reduce)")
                .ok()
                .flatten()
        })
        .map(|m| m.matches())
        .unwrap_or(false)
}

/// The current viewport height in CSS pixels (fallback `800.0`).
pub fn viewport_height() -> f64 {
    web_sys::window()
        .and_then(|w| w.inner_height().ok())
        .and_then(|v| v.as_f64())
        .unwrap_or(800.0)
}

/// The current vertical scroll offset in CSS pixels.
pub fn scroll_y() -> f64 {
    web_sys::window()
        .and_then(|w| w.scroll_y().ok())
        .unwrap_or(0.0)
}

/// True if the element's top edge is above `ratio` of the viewport height, i.e.
/// it is already on screen (or nearly so) at call time. Used to decide whether a
/// block should animate in (below the fold) or render statically (already seen).
pub fn element_in_view(el: &web_sys::Element, ratio: f64) -> bool {
    el.get_bounding_client_rect().top() <= viewport_height() * ratio
}

/// Smooth-scrolls to the element with the given id (CSS `scroll-behavior:
/// smooth` on `<html>` makes the motion smooth).
pub fn scroll_to(id: &str) {
    if let Some(el) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id(id))
    {
        el.scroll_into_view();
    }
}

/// Scrolls to a section shortly after the next paint — used right after a route
/// change to the home page, once the target section exists in the DOM.
pub fn scroll_to_soon(id: String) {
    wasm_bindgen_futures::spawn_local(async move {
        gloo_timers::future::TimeoutFuture::new(80).await;
        scroll_to(&id);
    });
}

/// Guard returned by [`observe_once`]; disconnects the observer when dropped
/// (i.e. when the owning component unmounts).
pub struct InViewGuard {
    observer: web_sys::IntersectionObserver,
    _on_intersect: Closure<dyn FnMut(js_sys::Array)>,
}

impl Drop for InViewGuard {
    fn drop(&mut self) {
        self.observer.disconnect();
    }
}

/// Fires `on_enter` the first time `el` reaches the given intersection
/// `threshold`, then disconnects. Returns a guard that also disconnects on drop;
/// store it for the element's lifetime.
pub fn observe_once(
    el: &web_sys::Element,
    threshold: f64,
    mut on_enter: impl FnMut() + 'static,
) -> Option<InViewGuard> {
    // Shared so the callback can disconnect the observer the moment it fires,
    // making the trigger strictly one-shot even before the guard drops.
    let slot: Rc<RefCell<Option<web_sys::IntersectionObserver>>> = Rc::new(RefCell::new(None));
    let slot_in_cb = slot.clone();
    let on_intersect = Closure::<dyn FnMut(js_sys::Array)>::new(move |entries: js_sys::Array| {
        let entry: web_sys::IntersectionObserverEntry = entries.get(0).unchecked_into();
        if entry.is_intersecting() {
            if let Some(obs) = slot_in_cb.borrow_mut().take() {
                obs.disconnect();
            }
            on_enter();
        }
    });

    let init = web_sys::IntersectionObserverInit::new();
    init.set_threshold(&JsValue::from_f64(threshold));
    let observer =
        web_sys::IntersectionObserver::new_with_options(on_intersect.as_ref().unchecked_ref(), &init)
            .ok()?;
    observer.observe(el);
    *slot.borrow_mut() = Some(observer.clone());
    Some(InViewGuard {
        observer,
        _on_intersect: on_intersect,
    })
}

/// Guard returned by [`add_window_listener`]; removes the listener on drop.
pub struct ListenerGuard {
    event: &'static str,
    closure: Closure<dyn FnMut(web_sys::Event)>,
}

impl Drop for ListenerGuard {
    fn drop(&mut self) {
        if let Some(win) = web_sys::window() {
            win.remove_event_listener_with_callback(
                self.event,
                self.closure.as_ref().unchecked_ref(),
            )
            .ok();
        }
    }
}

/// Registers `handler` as a `window` listener for `event`, returning a guard
/// that removes it on drop. `passive` must be `false` for handlers that call
/// `prevent_default` (e.g. the wheel-hijack). The handler reads live component
/// state through captured signals, so it never needs re-registering.
pub fn add_window_listener(
    event: &'static str,
    passive: bool,
    handler: impl FnMut(web_sys::Event) + 'static,
) -> Option<ListenerGuard> {
    let closure = Closure::<dyn FnMut(web_sys::Event)>::new(handler);
    let win = web_sys::window()?;
    let options = web_sys::AddEventListenerOptions::new();
    options.set_passive(passive);
    win.add_event_listener_with_callback_and_add_event_listener_options(
        event,
        closure.as_ref().unchecked_ref(),
        &options,
    )
    .ok()?;
    Some(ListenerGuard { event, closure })
}
