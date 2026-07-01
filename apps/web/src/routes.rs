//! Application routes and the shared layout shell.

use dioxus::prelude::*;

use crate::github::ReposState;
use crate::pages::{Home, Imprint, NotFound, Privacy};
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

    rsx! {
        div { class: "site",
            Masthead { on_open_palette: move |_| palette_open.set(true) }
            Outlet::<Route> {}
            Footer {}
            if palette_open() {
                CommandPalette {
                    repos: repos.repos(),
                    on_close: move |_| palette_open.set(false),
                }
            }
        }
    }
}
