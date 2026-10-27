//! Client-side interaction primitives (viewport queries, one-shot
//! in-view detection, global listeners) and small DOM helpers.
//!
//! The whole module is `web`-only: it compiles solely into the wasm client,
//! never the SSR server binary. Components call it from inside their own
//! `#[cfg(feature = "web")]` effects and keep their reactive state at the
//! "final, fully visible" value on the server, so the SSR HTML (and the no-JS
//! view) always shows complete content and hydration never mismatches. The
//! entrance animations are armed only after the client has hydrated.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use web_sys::wasm_bindgen::closure::Closure;
use web_sys::wasm_bindgen::{JsCast, JsValue};

/// Reports whether the user has requested reduced motion in the OS or browser.
///
/// The scroll-driven hero motion and the reveal and type-in entrance animations are skipped when
/// this holds.
pub fn prefers_reduced_motion() -> bool {
    web_sys::window()
        .and_then(|w| {
            w.match_media("(prefers-reduced-motion: reduce)")
                .ok()
                .flatten()
        })
        .is_some_and(|m| m.matches())
}

/// Returns the current viewport height in CSS pixels, or `800.0` when it cannot be read.
pub fn viewport_height() -> f64 {
    web_sys::window()
        .and_then(|w| w.inner_height().ok())
        .and_then(|v| v.as_f64())
        .unwrap_or(800.0)
}

/// Reports whether the element's top edge is above `ratio` of the viewport height, so it is
/// already on screen or nearly so.
///
/// A block that is already in view renders statically instead of animating in.
pub fn element_in_view(el: &web_sys::Element, ratio: f64) -> bool {
    el.get_bounding_client_rect().top() <= viewport_height() * ratio
}

/// Scrolls to the element with the given id, smoothly through the page's CSS `scroll-behavior`.
pub fn scroll_to(id: &str) {
    if let Some(el) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id(id))
    {
        el.scroll_into_view();
    }
}

/// Scrolls to a section shortly after the next paint, once the target exists in the DOM.
///
/// Call it right after a route change to the home page.
pub fn scroll_to_soon(id: String) {
    wasm_bindgen_futures::spawn_local(async move {
        gloo_timers::future::TimeoutFuture::new(80).await;
        scroll_to(&id);
    });
}

/// Disconnects the observer [`observe_once`] created when dropped, typically on unmount.
pub struct InViewGuard {
    observer: web_sys::IntersectionObserver,
    _on_intersect: Closure<dyn FnMut(js_sys::Array)>,
}

impl Drop for InViewGuard {
    fn drop(&mut self) {
        self.observer.disconnect();
    }
}

/// Calls `on_enter` the first time `el` reaches the given intersection `threshold`, then
/// disconnects.
///
/// The returned guard also disconnects on drop, so store it for the element's lifetime.
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
    let observer = web_sys::IntersectionObserver::new_with_options(
        on_intersect.as_ref().unchecked_ref(),
        &init,
    )
    .ok()?;
    observer.observe(el);
    *slot.borrow_mut() = Some(observer.clone());
    Some(InViewGuard {
        observer,
        _on_intersect: on_intersect,
    })
}

/// Removes the listener [`add_window_listener`] registered when dropped.
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

/// Registers `handler` as a `window` listener for `event` until the returned guard drops.
///
/// `passive` must be `false` for a handler that calls `prevent_default`, such as the ⌘K shortcut.
/// A handler that reads component state through captured signals never needs re-registering.
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

/// The one animation frame [`add_window_listener_per_frame`] may have requested and not yet
/// seen run.
// Kept apart from the `window` calls it brackets so the bookkeeping (request at most one frame at
// a time, forget it once it ran, cancel it if it never did) is testable without a browser.
#[derive(Default)]
struct FrameSlot(Cell<Option<i32>>);

impl FrameSlot {
    /// Calls `request` unless a frame is already pending, and records the id it returns.
    fn schedule(&self, request: impl FnOnce() -> Option<i32>) {
        if self.0.get().is_none() {
            self.0.set(request());
        }
    }

    /// Records that the pending frame ran.
    fn ran(&self) {
        self.0.set(None);
    }

    /// Hands a frame that never ran to `cancel`.
    fn cancel(&self, cancel: impl FnOnce(i32)) {
        if let Some(id) = self.0.take() {
            cancel(id);
        }
    }
}

/// Removes the listener [`add_window_listener_per_frame`] registered, and cancels a frame callback
/// that is still pending, when dropped.
pub struct FrameListenerGuard {
    _listener: ListenerGuard,
    _frame: Rc<Closure<dyn FnMut()>>,
    pending: Rc<FrameSlot>,
}

impl Drop for FrameListenerGuard {
    // Runs before the fields drop. A frame requested by the last event and not yet painted would
    // otherwise call into the closure after `_frame` freed it, which wasm-bindgen reports as an
    // uncaught "closure invoked … after being dropped" error on every unmount mid-scroll.
    fn drop(&mut self) {
        self.pending.cancel(|id| {
            if let Some(win) = web_sys::window() {
                let _ = win.cancel_animation_frame(id);
            }
        });
    }
}

/// Registers a passive `window` listener for `event` that runs `handler` at most
/// once per animation frame, no matter how many events arrive in between.
///
/// Only for handlers that observe: a handler calling `prevent_default` must use
/// [`add_window_listener`], because a deferred handler runs too late to cancel the event. Scrolling
/// allocates nothing, because the frame callback is created once and reused.
// Scroll fires far faster than the screen refreshes, and handlers that read layout
// (`getBoundingClientRect`, `scrollY`) or write signals force a synchronous reflow and a re-render
// each time they do. Coalescing onto the frame means at most one such pass per painted frame,
// which is the most the user can perceive anyway.
pub fn add_window_listener_per_frame(
    event: &'static str,
    handler: impl FnMut() + 'static,
) -> Option<FrameListenerGuard> {
    let pending = Rc::new(FrameSlot::default());
    let handler = Rc::new(RefCell::new(handler));

    let frame: Rc<Closure<dyn FnMut()>> = Rc::new(Closure::new({
        let pending = pending.clone();
        let handler = handler.clone();
        move || {
            pending.ran();
            (handler.borrow_mut())();
        }
    }));

    let listener = add_window_listener(event, true, {
        let frame = frame.clone();
        let pending = pending.clone();
        move |_| {
            pending.schedule(|| {
                web_sys::window()?
                    .request_animation_frame(frame.as_ref().as_ref().unchecked_ref())
                    .ok()
            });
        }
    })?;

    Some(FrameListenerGuard {
        _listener: listener,
        _frame: frame,
        pending,
    })
}

/// The CSS selector matching the elements a user can Tab to.
///
/// Every clause excludes `tabindex="-1"`, which is how the palette's options stay reachable by the
/// arrow keys only.
const FOCUSABLE: &str = "a[href]:not([tabindex='-1']), \
                         button:not([disabled]):not([tabindex='-1']), \
                         input:not([disabled]):not([tabindex='-1']), \
                         select:not([disabled]):not([tabindex='-1']), \
                         textarea:not([disabled]):not([tabindex='-1']), \
                         [tabindex]:not([tabindex='-1'])";

/// Keeps Tab focus inside `container` by wrapping from the last focusable element to the first,
/// and back with `shift`.
///
/// Call it from a modal's handler for a keydown already known to be Tab. It returns `true` when it
/// moved focus, in which case the caller must prevent the browser's default.
// A dialog that lets focus escape into the page behind it strands keyboard and screen-reader
// users, who have no way to see that they have left it.
pub fn trap_tab_focus(container: &web_sys::Element, shift: bool) -> bool {
    let Ok(nodes) = container.query_selector_all(FOCUSABLE) else {
        return false;
    };
    let focusable: Vec<web_sys::HtmlElement> = (0..nodes.length())
        .filter_map(|i| nodes.get(i))
        .filter_map(|node| node.dyn_into::<web_sys::HtmlElement>().ok())
        .collect();

    let (Some(first), Some(last)) = (focusable.first(), focusable.last()) else {
        return false;
    };
    let active = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.active_element());

    // Wrap only at the ends; in between, the browser's own order is correct.
    let target = match active {
        Some(active) if shift && active == **first => last,
        Some(active) if !shift && active == **last => first,
        _ => return false,
    };
    target.focus().is_ok()
}

#[cfg(test)]
mod tests {
    use super::FrameSlot;
    use std::cell::Cell;

    #[test]
    fn a_frame_is_requested_once_until_it_runs() {
        let slot = FrameSlot::default();
        let requests = Cell::new(0);
        let request = || {
            requests.set(requests.get() + 1);
            Some(requests.get())
        };

        slot.schedule(request);
        slot.schedule(request);
        assert_eq!(requests.get(), 1, "a second event requested another frame");

        slot.ran();
        slot.schedule(request);
        assert_eq!(
            requests.get(),
            2,
            "the next event after the frame ran requested none"
        );
    }

    #[test]
    fn only_a_frame_that_never_ran_is_cancelled() {
        let slot = FrameSlot::default();
        let cancelled = Cell::new(None);

        slot.schedule(|| Some(7));
        slot.cancel(|id| cancelled.set(Some(id)));
        assert_eq!(cancelled.get(), Some(7));

        cancelled.set(None);
        slot.schedule(|| Some(8));
        slot.ran();
        slot.cancel(|id| cancelled.set(Some(id)));
        assert_eq!(
            cancelled.get(),
            None,
            "a frame that already ran was cancelled"
        );
    }

    /// A refused request leaves nothing pending, so the next event tries again.
    #[test]
    fn a_failed_request_is_retried() {
        let slot = FrameSlot::default();
        let requests = Cell::new(0);

        slot.schedule(|| {
            requests.set(requests.get() + 1);
            None
        });
        slot.schedule(|| {
            requests.set(requests.get() + 1);
            Some(1)
        });
        assert_eq!(requests.get(), 2);
    }
}
