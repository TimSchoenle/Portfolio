use i18nrs::yew::use_translation;
use portfolio_data::CONFIG;
use yew::prelude::*;

use crate::components::about::About;
use crate::components::chapter_rail::ChapterRail;
use crate::components::contact::Contact;
use crate::components::experience::Experience;
use crate::components::hero::Hero;
use crate::components::projects::Projects;
use crate::components::skills::Skills;
use crate::github::ReposState;
use crate::i18n::set_document_title;

#[derive(Properties, PartialEq)]
pub struct HomeProps {
    pub repos: ReposState,
}

#[function_component(Home)]
pub fn home(p: &HomeProps) -> Html {
    let (i18n, _) = use_translation();

    {
        let lang = i18n.get_current_language().to_string();
        use_effect_with(lang, |_| set_document_title(CONFIG.title));
    }

    html! {
        <>
            <ChapterRail />
            <main class="main-container">
                <Hero />
                <About />
                <Skills />
                <Projects state={p.repos.clone()} />
                <Experience />
                <Contact />
            </main>
        </>
    }
}
