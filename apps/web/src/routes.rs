//! Application routes and the shared layout shell.

use dioxus::prelude::*;
use terrace_legal_dioxus::{LegalProvider, Shared, SharedTransport};

use crate::github::ReposState;
use crate::i18n::use_i18n;
use crate::legal::{LegalIndexes, SiteRouting, SiteSkin, SiteText, SiteTransport, legal_indexes};
use crate::pages::{Home, LegalDocument, Licenses, NotFound};
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
    /// A hosted legal document. Which slugs exist is configuration, so an unknown one renders
    /// the 404 page from inside the route rather than being refused by the router.
    #[route("/legal/:slug")]
    LegalDocument { slug: String },
    #[route("/licenses")]
    Licenses {},
    #[route("/:..segments")]
    NotFound { segments: Vec<String> },
}

/// Shared chrome around every page: the masthead, the routed page content
/// (`Outlet`), the footer and (client-side) the command palette.
///
/// It also resolves the legal index, because the footer on every page links each published
/// document and those links have to be in the server render: an imprint a reader without
/// JavaScript cannot reach is not "easily recognizable, directly accessible" (§ 5 DDG). The
/// fetch has no reactive input, so it suspends once, on the server, and hydrates from the
/// payload. The index is shared as context and handed to [`LegalProvider`], which the document
/// page's component reads its adapters from.
#[component]
fn Shell() -> Element {
    let mut palette_open = use_signal(|| false);
    let repos = use_context::<ReposState>();
    let i18n = use_i18n().i18n;

    let indexes = use_server_future(legal_indexes)?;
    // A failed fetch is only possible on the client, after hydration already had the answer;
    // an empty index then degrades to a footer without legal links rather than a broken shell.
    let indexes: LegalIndexes = indexes
        .read()
        .as_ref()
        .and_then(|result| result.as_ref().ok())
        .cloned()
        .unwrap_or(LegalIndexes::default());
    use_context_provider(|| indexes.clone());

    // Created once: the provider reads its handles on first render and compares them by
    // identity afterwards.
    let transport = use_hook(|| SharedTransport::new(SiteTransport::new(indexes.clone())));
    let text = use_hook(|| Shared::<dyn terrace_legal_dioxus::LegalText>::new(SiteText::new(i18n)));
    let routing = use_hook(|| Shared::<dyn terrace_legal_dioxus::LegalRouting>::new(SiteRouting));
    let skin = use_hook(|| Shared::<dyn terrace_legal_dioxus::LegalSkin>::new(SiteSkin));
    let language = use_memo(move || i18n.read().get_current_language().to_string());

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
        //
        // Not on the first run, which is hydration rather than a navigation: the
        // browser has already placed the page where the URL asked — a `#s4`
        // deep link, or the position restored after a reload — and resetting it
        // would throw that away.
        let router = router();
        let hydrated: Rc<std::cell::Cell<bool>> = use_hook(|| Rc::new(std::cell::Cell::new(false)));
        use_effect(move || {
            let _: Route = router.current();
            if !hydrated.replace(true) {
                return;
            }
            if let Some(win) = web_sys::window() {
                win.scroll_to_with_x_and_y(0.0, 0.0);
            }
        });
    }

    rsx! {
        LegalProvider {
            transport,
            text,
            routing,
            skin,
            language,
            div { class: "site",
                Masthead { on_open_palette: move |()| palette_open.set(true) }
                Outlet::<Route> {}
                Footer {}
                if palette_open() {
                    CommandPalette {
                        repos: repos.clone(),
                        on_close: move |()| palette_open.set(false),
                    }
                }
            }
        }
    }
}
