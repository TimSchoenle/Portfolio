//! Fixed left-hand chapter rail from the v4 design: roman-numeral dots that
//! track the active section while scrolling and reveal the section name.

use i18nrs::yew::use_translation;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use yew::prelude::*;

/// (rail label, element id, i18n key or None for the intro chapter).
const CHAPTERS: [(&str, &str, Option<&str>); 6] = [
    ("00", "top", None),
    ("i", "s1", Some("nav.about")),
    ("ii", "s2", Some("nav.skills")),
    ("iii", "s3", Some("nav.projects")),
    ("iv", "s4", Some("nav.experience")),
    ("v", "s5", Some("nav.contact")),
];

fn active_chapter() -> usize {
    let Some(win) = web_sys::window() else {
        return 0;
    };
    let Some(doc) = win.document() else { return 0 };
    let viewport = win
        .inner_height()
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(800.0);
    let mut current = 0;
    for (i, (_, id, _)) in CHAPTERS.iter().enumerate() {
        if let Some(el) = doc.get_element_by_id(id)
            && el.get_bounding_client_rect().top() <= viewport * 0.35
        {
            current = i;
        }
    }
    current
}

#[function_component(ChapterRail)]
pub fn chapter_rail() -> Html {
    let (i18n, _) = use_translation();
    let active = use_state_eq(|| 0_usize);

    {
        let active = active.clone();
        use_effect_with((), move |_| {
            active.set(active_chapter());
            let cb = Closure::<dyn Fn()>::wrap(Box::new(move || {
                active.set(active_chapter());
            }));
            let win = web_sys::window().expect("window available");
            win.add_event_listener_with_callback("scroll", cb.as_ref().unchecked_ref())
                .ok();
            move || {
                if let Some(win) = web_sys::window() {
                    win.remove_event_listener_with_callback("scroll", cb.as_ref().unchecked_ref())
                        .ok();
                }
                drop(cb);
            }
        });
    }

    html! {
        <nav class="chapter-rail" aria-label="Section navigation">
            { for CHAPTERS.iter().enumerate().map(|(i, (label, id, key))| {
                let name = key.map(|k| i18n.t(k)).unwrap_or_else(|| "intro".to_string());
                html! {
                    <a href={format!("#{id}")}
                       class={classes!("chapter-dot", (*active == i).then_some("active"))}
                       title={name.clone()}>
                        <span class="chapter-label">{*label}</span>
                        <span class="chapter-name">{name}</span>
                    </a>
                }
            })}
        </nav>
    }
}
