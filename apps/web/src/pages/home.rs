use dioxus::prelude::*;
use portfolio_data::CONFIG;

use crate::embedded::repos::ReposState;
use crate::ui::about::About;
use crate::ui::canonical::Canonical;
use crate::ui::chapter_rail::ChapterRail;
use crate::ui::contact::Contact;
use crate::ui::experience::Experience;
use crate::ui::hero::Hero;
use crate::ui::projects::Projects;
use crate::ui::skills::Skills;

/// Renders the `/` route: the chapter rail beside the six stacked sections.
///
/// It reads the repository listing from context, so it renders only under [`crate::app::App`].
#[component]
pub fn Home() -> Element {
    let repos = use_context::<ReposState>();

    rsx! {
        document::Title { "{CONFIG.title}" }
        Canonical { path: "/" }
        ChapterRail {}
        main { class: "main-container",
            Hero {}
            About {}
            Skills {}
            Projects { state: repos.clone() }
            Experience {}
            Contact {}
        }
    }
}
