//! Reveal-on-scroll wrapper from the v4 design: children fade/slide in once
//! the block enters the viewport, optionally after a stagger delay.

use yew::platform::spawn_local;
use yew::prelude::*;

use crate::hooks::use_in_view;

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
    let visible = use_in_view(&node, 0.12);
    let on = use_state(|| false);

    {
        let on = on.clone();
        let delay = p.delay;
        use_effect_with(visible, move |&visible| {
            if visible {
                spawn_local(async move {
                    if delay > 0 {
                        gloo_timers::future::TimeoutFuture::new(delay).await;
                    }
                    on.set(true);
                });
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
