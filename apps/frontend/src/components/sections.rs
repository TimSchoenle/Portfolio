//! Single ordered source of truth for the numbered "§" section labels.
//!
//! The position of a slug in [`SECTIONS`] *is* its section number, so the mono
//! labels (e.g. "§ 02 — about") are always derived dynamically and stay unique.
//! Reordering or inserting a section here renumbers everything downstream
//! automatically.

/// Section slugs in the order they appear on the home page. The array index
/// doubles as the section number rendered in the "§" labels.
pub const SECTIONS: [&str; 6] = [
    "identity",
    "about",
    "stack",
    "work",
    "experience",
    "contact",
];

/// Zero-based number of the section with the given slug (its index in
/// [`SECTIONS`]). Unknown slugs fall back to `0` so a typo can never silently
/// collide with another section's number.
pub fn section_index(slug: &str) -> usize {
    SECTIONS.iter().position(|s| *s == slug).unwrap_or(0)
}

/// Two-digit, zero-padded section number, e.g. `"02"`.
pub fn section_num(slug: &str) -> String {
    format!("{:02}", section_index(slug))
}

/// Full mono label for a section, e.g. `"§ 02 — about"`.
pub fn section_label(slug: &str) -> String {
    format!("§ {} — {slug}", section_num(slug))
}

/// DOM anchor id for a section, derived from its number so the id always
/// reflects the same ordering as the "§" labels, e.g. `"about"` -> `"s2"`.
pub fn section_id(slug: &str) -> String {
    format!("s{}", section_index(slug))
}
