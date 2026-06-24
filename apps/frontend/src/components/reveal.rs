//! Reveal-on-scroll wrapper from the v4 design: children fade/slide in once
//! the block enters the viewport, optionally after a stagger delay.

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen::closure::Closure;
use yew::platform::spawn_local;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct RevealProps {
    /// Stagger delay in milliseconds before the transition starts.
    #[prop_or(0)]
    pub delay: u32,
    #[prop_or_default]
    pub children: Html,
}

#[function_component(Reveal)]
pub fn reveal(p: &RevealProps) -> Html {
    let node = use_node_ref();
    let on = use_state(|| false);

    {
        let node = node.clone();
        let on = on.clone();
        let delay = p.delay;
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
                        let on = on.clone();
                        spawn_local(async move {
                            if delay > 0 {
                                gloo_timers::future::TimeoutFuture::new(delay).await;
                            }
                            on.set(true);
                        });
                    }
                }));

            let init = web_sys::IntersectionObserverInit::new();
            init.set_threshold(&JsValue::from_f64(0.12));
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

    let style = if *on {
        "opacity: 1; transform: translateY(0); \
         transition: opacity 0.7s cubic-bezier(.2,.8,.2,1), transform 0.7s cubic-bezier(.2,.8,.2,1);"
    } else {
        "opacity: 0; transform: translateY(16px); \
         transition: opacity 0.7s cubic-bezier(.2,.8,.2,1), transform 0.7s cubic-bezier(.2,.8,.2,1);"
    };

    html! {
        <div ref={node} style={style}>
            { p.children.clone() }
        </div>
    }
}
