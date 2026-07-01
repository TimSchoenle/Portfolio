//! Fixed left-hand chapter rail: roman-numeral dots that reveal the section
//! name. The active-chapter scroll tracking is a client enhancement (Phase 4);
//! for SSR/no-JS the first chapter renders active.

use dioxus::prelude::*;

use crate::i18n::use_i18n;
use crate::sections::{section_id, section_index};

/// (section slug, i18n key or None for the identity hero).
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

/// Rail label: the identity hero keeps "00"; every other chapter uses the roman
/// numeral of its section number.
fn chapter_label(slug: &str) -> String {
    let n = section_index(slug);
    if n == 0 {
        "00".to_string()
    } else {
        to_roman(n)
    }
}

/// The identity hero lives under `top`; everything else uses [`section_id`].
fn chapter_id(slug: &str) -> String {
    if slug == "identity" {
        "top".to_string()
    } else {
        section_id(slug)
    }
}

/// The furthest chapter whose section has scrolled past the 35%-viewport line;
/// `0` until the DOM is laid out. Client-only (reads the live layout).
#[cfg(feature = "web")]
fn active_chapter() -> usize {
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
        return 0;
    };
    let viewport = crate::hooks::viewport_height();
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

#[component]
pub fn ChapterRail() -> Element {
    let i18n = use_i18n().i18n;
    let t = move |k: &str| i18n.read().t(k);
    // First chapter active on the server / no-JS; the client tracks the real
    // active section on scroll.
    let active = use_signal(|| 0usize);

    #[cfg(feature = "web")]
    {
        use crate::hooks::{ListenerGuard, add_window_listener};
        use std::cell::RefCell;
        use std::rc::Rc;

        let mut active = active;
        // Set the initial active chapter once mounted (the layout isn't ready at
        // first render), then keep it in sync on scroll for the rail's lifetime.
        use_effect(move || active.set(active_chapter()));
        let _listener: Rc<RefCell<Option<ListenerGuard>>> = use_hook(|| {
            Rc::new(RefCell::new(add_window_listener("scroll", true, move |_| {
                active.set(active_chapter());
            })))
        });
    }

    let active = active();
    rsx! {
        nav { class: "chapter-rail", "aria-label": "Section navigation",
            {CHAPTERS.iter().enumerate().map(|(i, (slug, key))| {
                let id = chapter_id(slug);
                let label = chapter_label(slug);
                let name = key.map(&t).unwrap_or_else(|| (*slug).to_string());
                let cls = if i == active { "chapter-dot active" } else { "chapter-dot" };
                rsx! {
                    a { key: "{slug}", href: "#{id}", class: "{cls}", title: "{name}",
                        span { class: "chapter-label", "{label}" }
                        span { class: "chapter-name", "{name}" }
                    }
                }
            })}
        }
    }
}
