//! Fixed left-hand chapter rail from the v4 design: roman-numeral dots that
//! track the active section while scrolling and reveal the section name.

use i18nrs::yew::use_translation;
use yew::prelude::*;

use crate::hooks::use_window_event;

use super::sections::{section_id, section_index};

/// (section slug, i18n key or None for the identity hero). The rail label is
/// derived from each slug's [`section_index`] so the numerals always track the
/// same ordering as the "§" section labels.
const CHAPTERS: [(&str, Option<&str>); 6] = [
    ("identity", None),
    ("about", Some("nav.about")),
    ("stack", Some("nav.skills")),
    ("work", Some("nav.projects")),
    ("experience", Some("nav.experience")),
    ("contact", Some("nav.contact")),
];

/// Lowercase roman numeral for `n`, e.g. 1 -> "i", 4 -> "iv", 6 -> "vi".
fn to_roman(mut n: usize) -> String {
    const TABLE: [(usize, &str); 13] = [
        (1000, "m"),
        (900, "cm"),
        (500, "d"),
        (400, "cd"),
        (100, "c"),
        (90, "xc"),
        (50, "l"),
        (40, "xl"),
        (10, "x"),
        (9, "ix"),
        (5, "v"),
        (4, "iv"),
        (1, "i"),
    ];
    let mut out = String::new();
    for (value, symbol) in TABLE {
        while n >= value {
            out.push_str(symbol);
            n -= value;
        }
    }
    out
}

/// Rail label for a chapter: the identity hero keeps the Arabic "00" to match
/// its "§ 00" eyebrow; every other chapter uses the roman numeral of its
/// section number (about -> "i", stack -> "ii", … contact -> "v").
fn chapter_label(slug: &str) -> String {
    let n = section_index(slug);
    if n == 0 {
        "00".to_string()
    } else {
        to_roman(n)
    }
}

/// Resolves a chapter slug to its DOM anchor id. The identity hero lives under
/// the special `top` id; everything else uses [`section_id`].
fn chapter_id(slug: &str) -> String {
    if slug == "identity" {
        "top".to_string()
    } else {
        section_id(slug)
    }
}

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
    for (i, (slug, _)) in CHAPTERS.iter().enumerate() {
        if let Some(el) = doc.get_element_by_id(&chapter_id(slug))
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

    // Set the initial active chapter once mounted (the DOM isn't laid out yet
    // at first render), then keep it in sync on scroll.
    {
        let active = active.clone();
        use_effect_with((), move |_| active.set(active_chapter()));
    }
    {
        let active = active.clone();
        use_window_event("scroll", (), move |_: web_sys::Event| {
            active.set(active_chapter());
        });
    }

    html! {
        <nav class="chapter-rail" aria-label="Section navigation">
            { for CHAPTERS.iter().enumerate().map(|(i, (slug, key))| {
                let id = chapter_id(slug);
                let label = chapter_label(slug);
                let name = key.map(|k| i18n.t(k)).unwrap_or_else(|| (*slug).to_string());
                html! {
                    <a href={format!("#{id}")}
                       class={classes!("chapter-dot", (*active == i).then_some("active"))}
                       title={name.clone()}>
                        <span class="chapter-label">{label}</span>
                        <span class="chapter-name">{name}</span>
                    </a>
                }
            })}
        </nav>
    }
}
