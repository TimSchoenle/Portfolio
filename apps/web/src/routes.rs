//! Application routes and the shared layout shell.

use dioxus::prelude::*;

use crate::github::ReposState;
use crate::pages::{Home, Imprint, Licenses, NotFound, Privacy};
use crate::ui::footer::Footer;
use crate::ui::masthead::Masthead;
use crate::ui::palette::CommandPalette;

/// The site's routes. `#[layout(Shell)]` wraps every page in the masthead +
/// footer chrome; the trailing catch-all renders the 404 page.
#[derive(Routable, Clone, PartialEq)]
pub enum Route {
    #[layout(Shell)]
    #[route("/")]
    Home {},
    #[route("/imprint")]
    Imprint {},
    #[route("/privacy")]
    Privacy {},
    #[route("/licenses")]
    Licenses {},
    #[route("/:..segments")]
    NotFound { segments: Vec<String> },
}

/// Shared chrome around every page: the masthead, the routed page content
/// (`Outlet`), the footer and (client-side) the command palette. The masthead
/// and palette are wired in as their components are ported.
#[component]
fn Shell() -> Element {
    let mut palette_open = use_signal(|| false);
    let repos = use_context::<ReposState>();

    #[cfg(feature = "web")]
    {
        use crate::hooks::{ListenerGuard, add_window_listener};
        use std::cell::RefCell;
        use std::rc::Rc;
        use web_sys::wasm_bindgen::JsCast;

        // Global ⌘K / Ctrl+K toggles the palette; Escape closes it. The handler
        // reads the live open-state through the captured signal.
        let mut palette_open = palette_open;
        let _keys: Rc<RefCell<Option<ListenerGuard>>> = use_hook(|| {
            Rc::new(RefCell::new(add_window_listener(
                "keydown",
                false,
                move |e| {
                    let Some(key) = e.dyn_ref::<web_sys::KeyboardEvent>() else {
                        return;
                    };
                    if (key.meta_key() || key.ctrl_key()) && key.key() == "k" {
                        key.prevent_default();
                        let open = palette_open();
                        palette_open.set(!open);
                    } else if key.key() == "Escape" {
                        palette_open.set(false);
                    }
                },
            )))
        });

        // Reset scroll to the top on every route change (an SPA keeps the old
        // position otherwise). Reading `current()` subscribes this effect to
        // navigations, so it re-runs whenever the route changes.
        let router = router();
        use_effect(move || {
            let _: Route = router.current();
            if let Some(win) = web_sys::window() {
                win.scroll_to_with_x_and_y(0.0, 0.0);
            }
        });
    }

    rsx! {
        div { class: "site",
            Masthead { on_open_palette: move |_| palette_open.set(true) }
            Outlet::<Route> {}
            Footer {}
            if palette_open() {
                CommandPalette {
                    repos: repos.clone(),
                    on_close: move |_| palette_open.set(false),
                }
            }
        }
    }
}
