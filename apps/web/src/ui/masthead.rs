//! Top masthead: logo, section nav, command-palette trigger, language toggle.

use dioxus::prelude::*;

use crate::i18n::{other_language, use_i18n};
use crate::routes::Route;
use crate::sections::{section_id, section_num};

/// Home-page sections, in nav order: (section slug, i18n key). Anchors resolve
/// via [`section_id`], which tracks the dynamic numbering.
pub const SECTIONS: [(&str, &str); 5] = [
    ("about", "nav.about"),
    ("stack", "nav.skills"),
    ("work", "nav.projects"),
    ("experience", "nav.experience"),
    ("contact", "nav.contact"),
];

#[component]
pub fn Masthead(on_open_palette: EventHandler<()>) -> Element {
    let ctx = use_i18n();
    let i18n = ctx.i18n;
    let set_language = ctx.set_language;
    let t = move |k: &str| i18n.read().t(k);

    let lang = i18n.read().get_current_language().to_string();
    let next_lang = other_language(&lang).to_string();
    let search_label = t("nav.search");
    let lang_aria = t("nav.languageToggle");

    rsx! {
        header { class: "masthead",
            div { class: "masthead-left",
                Link { to: Route::Home {}, class: "logo-mark",
                    img { class: "logo-img", src: "/favicon.svg", alt: "TS", width: "28", height: "28" }
                }
            }

            nav { class: "masthead-nav",
                {SECTIONS.iter().map(|(slug, key)| {
                    let id = section_id(slug);
                    let num = section_num(slug);
                    let label = t(key);
                    rsx! {
                        a { key: "{slug}", href: "/#{id}",
                            span { class: "mono text-muted", "{num}" }
                            span { class: "mono text-fg ml-1.5", "{label}" }
                        }
                    }
                })}
            }

            div { class: "masthead-right",
                button {
                    class: "cmdk-trigger",
                    onclick: move |_| on_open_palette.call(()),
                    title: "Command palette (⌘K)",
                    span { "⌘K" }
                    span { class: "mono text-muted ml-2", "{search_label}" }
                }
                button {
                    class: "lang-toggle",
                    onclick: move |_| set_language.call(next_lang.clone()),
                    "aria-label": "{lang_aria}",
                    span { class: "mono",
                        if lang == "de" {
                            span { class: "lang-off", "EN" }
                            " · "
                            span { class: "lang-on", "DE" }
                        } else {
                            span { class: "lang-on", "EN" }
                            " · "
                            span { class: "lang-off", "DE" }
                        }
                    }
                }
            }
        }
    }
}
