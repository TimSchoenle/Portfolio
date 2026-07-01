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
    if n == 0 { "00".to_string() } else { to_roman(n) }
}

/// The identity hero lives under `top`; everything else uses [`section_id`].
fn chapter_id(slug: &str) -> String {
    if slug == "identity" {
        "top".to_string()
    } else {
        section_id(slug)
    }
}

#[component]
pub fn ChapterRail() -> Element {
    let i18n = use_i18n().i18n;
    let t = move |k: &str| i18n.read().t(k);
    let active = 0usize;

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
