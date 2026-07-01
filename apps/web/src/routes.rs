//! Application routes and the shared layout shell.

use dioxus::prelude::*;

use crate::pages::{Home, Imprint, NotFound, Privacy};
use crate::ui::footer::Footer;

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
    rsx! {
        div { class: "site",
            Outlet::<Route> {}
            Footer {}
        }
    }
}
