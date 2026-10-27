//! The single ordered source of the numbered "§" section labels.
//!
//! The position of a slug in [`SECTIONS`] *is* its section number, so the mono
//! labels, such as `§ 01 — about`, are derived and stay unique. Reordering or
//! inserting a section here renumbers everything downstream.

/// The section slugs in the order they appear on the home page.
///
/// The array index doubles as the section number rendered in the "§" labels.
pub const SECTIONS: [&str; 6] = [
    "identity",
    "about",
    "stack",
    "work",
    "experience",
    "contact",
];

/// Returns the zero-based index of `slug` in [`SECTIONS`], or `0` for an unknown slug.
pub fn section_index(slug: &str) -> usize {
    SECTIONS.iter().position(|s| *s == slug).unwrap_or(0)
}

/// Formats the section number of `slug` as two zero-padded digits, such as `"01"` for `"about"`.
pub fn section_num(slug: &str) -> String {
    format!("{:02}", section_index(slug))
}

/// Formats the full mono label for a section, such as `"§ 01 — about"`.
pub fn section_label(slug: &str) -> String {
    format!("§ {} — {slug}", section_num(slug))
}

/// Returns the DOM anchor id for a section, such as `"s1"` for `"about"`.
pub fn section_id(slug: &str) -> String {
    format!("s{}", section_index(slug))
}
