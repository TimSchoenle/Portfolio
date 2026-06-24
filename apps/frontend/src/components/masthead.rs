use i18nrs::yew::use_translation;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::hooks::{scroll_to, scroll_to_soon};
use crate::i18n::other_language;
use crate::router::Route;

/// Home-page sections, in nav order: (element id, i18n key).
pub const SECTIONS: [(&str, &str); 5] = [
    ("s1", "nav.about"),
    ("s2", "nav.skills"),
    ("s3", "nav.projects"),
    ("s4", "nav.experience"),
    ("s5", "nav.contact"),
];

/// Navigates to a home-page section from anywhere in the app.
pub fn goto_section(navigator: &Navigator, route: Route, id: &'static str) {
    if route == Route::Home {
        scroll_to(id);
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
                    <span class="logo-glyph">{"◇"}</span>
                    <span class="mono text-fg">{"TS"}</span>
                    <span class="mono text-muted ml-1.5">{"/ portfolio.v4"}</span>
                </Link<Route>>
            </div>

            <nav class="masthead-nav">
                { for SECTIONS.iter().enumerate().map(|(i, (id, key))| {
                    let navigator = navigator.clone();
                    let onclick = Callback::from(move |e: MouseEvent| {
                        e.prevent_default();
                        goto_section(&navigator, route, id);
                    });
                    html!{
                        <a href={format!("/#{id}")} {onclick}>
                            <span class="mono text-muted">{format!("{:02}", i+1)}</span>
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
