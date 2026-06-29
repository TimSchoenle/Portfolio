//! Design tokens (colors, typography, spacing, two-column geometry) for the
//! Typst resume, following the house style guide. The template references these
//! roles; no size or color is hard-coded outside this module.
//!
//! Everything scales together through [`Layout::scale`] so the visual hierarchy
//! survives the fit-to-one-page reduction (§9: density adjusts spacing, never
//! type below the readability floors), while [`Layout::dense`] selects the
//! Compact density preset (tighter margins, gutter, spacing and line-height).

// ---- color tokens (§2.1, light theme — the only theme used for PDF) ----

/// `--c-accent` (default slate blue): name, header rule, section headers, links.
pub(crate) const ACCENT: &str = "#2D4A77";
/// `--c-ink`: primary text.
pub(crate) const INK: &str = "#16181D";
/// `--c-ink-soft`: secondary prose (the headline).
pub(crate) const INK_SOFT: &str = "#3A3F47";
/// `--c-ink-muted`: meta — dates, location, context, contact, bullet marker.
///
/// Darkened from the original `#6B7280` to meet the readability floor (F5 / §8):
/// ~7:1 on white `--c-bg` and ~6.6:1 on the `#F4F6F9` sidebar tint, while
/// staying visibly quieter than near-black `--c-ink`.
pub(crate) const INK_MUTED: &str = "#4B5563";
/// `--c-rule`: hairline beneath main section headers.
pub(crate) const RULE: &str = "#E3E6EA";
/// `--c-sidebar-bg`: the subtle sidebar tint.
pub(crate) const SIDEBAR_BG: &str = "#F4F6F9";
/// `--c-link` (§8 / F9): every link uses the accent so none drifts to muted.
pub(crate) const LINK: &str = ACCENT;
/// `--c-tag-bg`: skill-chip background. White on the tinted sidebar (§6.3 / N3).
pub(crate) const TAG_BG: &str = "#FFFFFF";
/// `--c-tag-ink`: skill-chip text.
pub(crate) const TAG_INK: &str = "#3A3F47";
/// Skill-chip / status-tag hairline border (R6): a touch darker than `--c-rule`
/// so the white chip's edges register against the `#F4F6F9` sidebar tint
/// without looking heavy.
pub(crate) const TAG_BORDER: &str = "#D2D8E0";
/// `--radius-tag`: skill-chip corner radius.
pub(crate) const RADIUS_TAG_PT: f64 = 2.25; // ~3px

// ---- typography bases in points (§2.2 / §4) ----

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
/// Skill-chip text size (§2.2 `--fs-tag`).
const FS_TAG: f64 = 8.5;

// ---- spacing bases in points (§2.3, 4 pt base) ----

pub(crate) const SP_1: f64 = 2.0; // between bullets (legacy small gap)
/// Dedicated inter-bullet gap (§2.3 `--bullet-gap`, N7): wider than `SP_1` so
/// multi-line bullets don't read as one block.
pub(crate) const BULLET_GAP: f64 = 3.0;
pub(crate) const SP_2: f64 = 4.0; // title row → first bullet; header text → rule
pub(crate) const SP_3: f64 = 6.0; // inside blocks; sidebar item / contact-row gaps
pub(crate) const SP_4: f64 = 8.0; // section rule → first entry
pub(crate) const SP_5: f64 = 11.0; // between entries
pub(crate) const SP_6: f64 = 14.0; // header band → grid
pub(crate) const SP_7: f64 = 18.0; // above each section header (within a column)

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

    // -- two-column geometry (§2.4) --

    /// Page side / top-bottom margins (Comfortable default, Compact preset, §9).
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

    /// Body line-height (`--lh-body`, N7): Comfortable 1.40 / Compact 1.32 →
    /// Typst `leading` (the gap added on top of the font size, i.e. `lh - 1`).
    /// The compact floor is held at 1.32 (never the old, clipping 1.22) so a
    /// line's descenders keep clear space above the next line's ascenders.
    pub(crate) fn leading_em(&self) -> f64 {
        if self.dense { 0.32 } else { 0.40 }
    }
}
