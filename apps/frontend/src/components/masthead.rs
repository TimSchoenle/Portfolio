use i18nrs::yew::use_translation;
use yew::prelude::*;
use yew_router::prelude::*;

use super::sections::section_id;
use crate::hooks::{scroll_to, scroll_to_soon};
use crate::i18n::other_language;
use crate::router::Route;

/// Home-page sections, in nav order: (section slug, i18n key). The anchor id is
/// derived from the slug via [`section_id`] so it tracks the dynamic numbering.
pub const SECTIONS: [(&str, &str); 5] = [
    ("about", "nav.about"),
    ("stack", "nav.skills"),
    ("work", "nav.projects"),
    ("experience", "nav.experience"),
    ("contact", "nav.contact"),
];

/// Navigates to a home-page section from anywhere in the app.
pub fn goto_section(navigator: &Navigator, route: Route, id: String) {
    if route == Route::Home {
        scroll_to(&id);
    } else {
        navigator.push(&Route::Home);
        scroll_to_soon(id);
    }
}

#[derive(Properties, PartialEq)]
pub struct MastheadProps {
    pub on_open_palette: Callback<MouseEvent>,
}

#[function_component(Masthead)]
pub fn masthead(p: &MastheadProps) -> Html {
    let (i18n, set_language) = use_translation();
    let navigator = use_navigator().expect("masthead rendered inside router");
    let route = use_route::<Route>().unwrap_or(Route::Home);

    let lang = i18n.get_current_language().to_string();
    let toggle_lang = {
        let lang = lang.clone();
        Callback::from(move |_: MouseEvent| {
            set_language.emit(other_language(&lang).to_string());
        })
    };

    html! {
        <header class="masthead">
            <div class="masthead-left">
                <Link<Route> to={Route::Home} classes="logo-mark">
                    <img class="logo-img" src="/favicon.svg" alt="TS" width="28" height="28" />
                </Link<Route>>
            </div>

            <nav class="masthead-nav">
                { for SECTIONS.iter().map(|(slug, key)| {
                    let id = section_id(slug);
                    let navigator = navigator.clone();
                    let onclick = {
                        let id = id.clone();
                        Callback::from(move |e: MouseEvent| {
                            e.prevent_default();
                            goto_section(&navigator, route, id.clone());
                        })
                    };
                    html!{
                        <a href={format!("/#{id}")} {onclick}>
                            <span class="mono text-muted">{ super::sections::section_num(slug) }</span>
                            <span class="mono text-fg ml-1.5">{ i18n.t(key) }</span>
                        </a>
                    }
                })}
            </nav>

            <div class="masthead-right">
                <button class="cmdk-trigger" onclick={p.on_open_palette.clone()} title="Command palette (⌘K)">
                    <span>{"⌘K"}</span>
                    <span class="mono text-muted ml-2">{ i18n.t("nav.search") }</span>
                </button>
                <button class="lang-toggle" onclick={toggle_lang} aria-label={ i18n.t("nav.languageToggle") }>
                    <span class="mono">
                        if lang == "de" {
                            <span class="lang-off">{"EN"}</span>{" · "}<span class="lang-on">{"DE"}</span>
                        } else {
                            <span class="lang-on">{"EN"}</span>{" · "}<span class="lang-off">{"DE"}</span>
                        }
                    </span>
                </button>
            </div>
        </header>
    }
}
