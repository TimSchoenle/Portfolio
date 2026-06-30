//! Design tokens (colors, typography, spacing, two-column geometry) for the
//! Typst resume. The template references these tokens; no size or color is
//! hard-coded outside this module.
//!
//! Everything scales together through [`Layout::scale`] so the visual hierarchy
//! survives the fit-to-one-page reduction, while [`Layout::dense`] selects the
//! tighter Compact density preset.

// ---- color tokens (light theme — the only theme used for PDF) ----

/// Accent (slate blue): name, header rule, section headers, links.
pub(crate) const ACCENT: &str = "#2D4A77";
/// Primary text.
pub(crate) const INK: &str = "#16181D";
/// Secondary prose (the headline).
pub(crate) const INK_SOFT: &str = "#3A3F47";
/// Meta text: dates, location, context, contact, bullet marker. Dark enough to
/// stay readable on both white and the sidebar tint, yet quieter than `INK`.
pub(crate) const INK_MUTED: &str = "#4B5563";
/// Hairline beneath main section headers.
pub(crate) const RULE: &str = "#E3E6EA";
/// The subtle sidebar tint.
pub(crate) const SIDEBAR_BG: &str = "#F4F6F9";
/// Links use the accent color.
pub(crate) const LINK: &str = ACCENT;
/// Skill-chip background (white, on the tinted sidebar).
pub(crate) const TAG_BG: &str = "#FFFFFF";
/// Skill-chip text.
pub(crate) const TAG_INK: &str = "#3A3F47";
/// Skill-chip border: slightly darker than `RULE` so white chips register
/// against the sidebar tint without looking heavy.
pub(crate) const TAG_BORDER: &str = "#D2D8E0";
/// Skill-chip corner radius.
pub(crate) const RADIUS_TAG_PT: f64 = 2.25; // ~3px
/// Hairline weight for the divider between experience entries; kept thin so it
/// reads as a quiet divider, not a heavy line.
pub(crate) const RULE_WEIGHT_PT: f64 = 0.5;

// ---- typography bases in points ----

const FS_NAME: f64 = 23.0;
const FS_HEADLINE: f64 = 11.0;
const FS_CONTACT: f64 = 9.0;
const FS_SECTION: f64 = 10.5;
const FS_SECTION_SM: f64 = 9.5;
const FS_TITLE: f64 = 10.5;
const FS_META: f64 = 9.5;
const FS_BODY: f64 = 10.0;
const FS_SIDEBAR: f64 = 9.5;
/// Small-caps contact label of the no-icon contact variant (`EMAIL`, `WEB`, …).
const FS_LABEL: f64 = 7.5;
/// Skill-chip text size.
const FS_TAG: f64 = 8.5;

// ---- spacing bases in points ----
//
// Each vertical level is visibly larger than the one beneath it so the eye can
// separate them: line < bullet < entry < section.

pub(crate) const SP_1: f64 = 2.0; // title ↔ company; small gap
/// Inter-bullet gap: wide enough that multi-line bullets don't read as one
/// block.
pub(crate) const BULLET_GAP: f64 = 5.0;
pub(crate) const SP_2: f64 = 4.0; // header text → rule
pub(crate) const SP_3: f64 = 6.0; // company line → first bullet; inside blocks
/// Skills group label → its chips: a small step that keeps the label with its
/// chip rows.
pub(crate) const SP_LABEL: f64 = 3.0;
pub(crate) const SP_4: f64 = 8.0; // section rule → first entry; between sidebar groups
/// Above the `Stack:` run: detaches it from the last bullet so it never reads
/// as another bullet.
pub(crate) const STACK_GAP: f64 = 5.0;
pub(crate) const SP_5: f64 = 15.0; // between entries — roles read as distinct blocks
pub(crate) const SP_6: f64 = 14.0; // header band → grid
pub(crate) const SP_7: f64 = 20.0; // above each section header — the largest step

/// Scaled token resolver. Sizes derive from the point bases times `scale`;
/// spacing additionally tightens under the Compact density.
#[derive(Clone, Copy)]
pub(crate) struct Layout {
    pub(crate) scale: f64,
    pub(crate) dense: bool,
}

impl Layout {
    /// A scaled length literal in points, e.g. `"9.45pt"`.
    fn pt(&self, base: f64) -> String {
        format!("{:.2}pt", base * self.scale)
    }

    /// A scaled, density-aware spacing literal in points.
    pub(crate) fn sp(&self, base: f64) -> String {
        let factor = if self.dense { 0.78 } else { 1.0 };
        format!("{:.2}pt", base * self.scale * factor)
    }

    // -- type roles (size literals) --

    pub(crate) fn fs_name(&self) -> String {
        self.pt(FS_NAME)
    }
    pub(crate) fn fs_headline(&self) -> String {
        self.pt(FS_HEADLINE)
    }
    pub(crate) fn fs_contact(&self) -> String {
        self.pt(FS_CONTACT)
    }
    pub(crate) fn fs_section(&self) -> String {
        self.pt(FS_SECTION)
    }
    pub(crate) fn fs_section_sm(&self) -> String {
        self.pt(FS_SECTION_SM)
    }
    pub(crate) fn fs_title(&self) -> String {
        self.pt(FS_TITLE)
    }
    pub(crate) fn fs_meta(&self) -> String {
        self.pt(FS_META)
    }
    pub(crate) fn fs_body(&self) -> String {
        self.pt(FS_BODY)
    }
    pub(crate) fn fs_sidebar(&self) -> String {
        self.pt(FS_SIDEBAR)
    }
    pub(crate) fn fs_label(&self) -> String {
        self.pt(FS_LABEL)
    }
    pub(crate) fn fs_tag(&self) -> String {
        self.pt(FS_TAG)
    }

    /// Skill-chip corner radius (scaled).
    pub(crate) fn radius_tag(&self) -> String {
        format!("{:.2}pt", RADIUS_TAG_PT * self.scale)
    }

    // -- two-column geometry --

    /// Page side / top-bottom margins (Comfortable default, Compact preset).
    pub(crate) fn margin_x_mm(&self) -> f64 {
        if self.dense { 11.0 } else { 14.0 }
    }
    pub(crate) fn margin_y_mm(&self) -> f64 {
        if self.dense { 10.0 } else { 13.0 }
    }
    /// Column gutter (`--col-gap`).
    pub(crate) fn col_gap_mm(&self) -> f64 {
        if self.dense { 7.0 } else { 9.0 }
    }
    /// Sidebar inner padding (`--sidebar-pad`).
    pub(crate) fn sidebar_pad_mm(&self) -> f64 {
        if self.dense { 5.5 } else { 7.0 }
    }

    /// Body line-height as Typst `leading` (the gap added on top of the font
    /// size, i.e. `line-height - 1`): Comfortable 1.45, Compact 1.32.
    pub(crate) fn leading_em(&self) -> f64 {
        if self.dense { 0.32 } else { 0.45 }
    }
}
