//! Design tokens (colors, typography, spacing, two-column geometry) for the
//! Typst resume. The template references these tokens; no size or color is
//! hard-coded outside this module.
//!
//! # Design space
//!
//! The print design is authored on an A4 sheet measured in CSS pixels at 96 dpi
//! (794 × 1123 px), so every length here is a [`Px`] and [`Layout::pt`] is the
//! single place that converts one to the typographic points Typst wants
//! ([`PT_PER_PX`]). Keeping the numbers in the design's own unit is what makes
//! a token auditable against the source design: `34.0` here is `34px` there.
//!
//! # Two knobs
//!
//! Everything scales together through [`Layout::scale`] so the visual hierarchy
//! survives the fit-to-one-page reduction, while [`Layout::dense`] selects the
//! [`COMPACT`] spacing table over [`COMFORTABLE`]. The two tables are the two
//! sheets of the design, not a single factor applied to one of them: the
//! designer tightened some gaps far more than others (the divider band halves
//! while the chip gutter loses a quarter), and a uniform multiplier reproduces
//! neither sheet.

/// A length in the design's own unit: one CSS pixel of the A4 print mock at
/// 96 dpi.
pub(crate) type Px = f64;

/// One design pixel in typographic points — 96 dpi against Typst's 72 dpi.
const PT_PER_PX: f64 = 0.75;

/// Inter's ascender as a multiple of the font size.
const FONT_ASCENDER_EM: f64 = 0.968_750;
/// Inter's descender as a multiple of the font size, unsigned.
const FONT_DESCENDER_EM: f64 = 0.241_211;
/// Inter's line box as a multiple of the font size: ascender + descender, the
/// height a browser gives a line at `line-height: normal`.
///
/// The design is written in CSS line heights, and [`Layout::edges`] is what
/// turns one into the pair of Typst text edges that reproduces it — see there
/// for why the document runs at zero leading.
const FONT_LINE_EM: f64 = FONT_ASCENDER_EM + FONT_DESCENDER_EM;

// ---- color tokens (light theme — the only theme used for PDF) ----

/// Sheet color. The design's near-white paper, a shade off pure white so the
/// tinted sidebar and the white skill chips both register against it.
pub(crate) const PAPER: &str = "#FDFDFE";
/// Accent: job title, section headers, links, the start of the header rule.
/// The print-side reading of the site's `#9184d9` — same OKLCH hue family,
/// darkened until it holds contrast on paper.
pub(crate) const ACCENT: &str = "#4A3E96";
/// Softer accent: bullet markers and the midpoint of the header rule's fade.
pub(crate) const ACCENT_SOFT: &str = "#8B7FD4";
/// Primary text: the name, role titles, section labels.
pub(crate) const INK: &str = "#191A21";
/// Body prose: the summary and the bullet text. A step down from [`INK`] so a
/// paragraph reads quieter than the headings around it.
pub(crate) const INK_BODY: &str = "#2A2C34";
/// Secondary text: the organisation line, the `Stack` run, sidebar values and
/// chip text.
pub(crate) const INK_SOFT: &str = "#3C3F4A";
/// Meta text: dates, locations, sidebar micro-labels. Dark enough to stay
/// readable on both the paper and the sidebar tint, yet quieter than [`INK`].
pub(crate) const INK_MUTED: &str = "#5A5E6B";
/// Hairline beneath main section headers.
pub(crate) const RULE: &str = "#E2E0EC";
/// Hairline between experience entries — lighter than [`RULE`], so an entry
/// break never competes with a section break.
pub(crate) const RULE_ENTRY: &str = "#ECEAF3";
/// The subtle sidebar tint, also the availability pill's fill.
pub(crate) const SIDEBAR_BG: &str = "#F4F3FA";
/// Links use the accent color.
pub(crate) const LINK: &str = ACCENT;
/// Skill-chip and QR/photo frame background (white, on the tinted sidebar).
pub(crate) const TAG_BG: &str = "#FFFFFF";
/// Skill-chip text.
pub(crate) const TAG_INK: &str = INK_SOFT;
/// Skill-chip, QR-frame and photo-frame border: slightly darker than [`RULE`]
/// so white boxes register against the sidebar tint without looking heavy.
pub(crate) const TAG_BORDER: &str = "#DBD7EA";
/// Availability-pill border — a touch more present than [`TAG_BORDER`], since
/// the pill sits on the paper rather than on the tint.
pub(crate) const PILL_BORDER: &str = "#D5D0E8";

// ---- font weights ----
//
// The design uses three weights above the body, and [`W_TITLE`] is not a value
// any static cut of Inter carries. The generator therefore ships the variable
// face and lets Typst instantiate the `wght` axis, so 650 is a real 650 and
// stays visibly lighter than the 700 section headers it has to be distinct
// from.

/// Regular (400): body prose, chips, sidebar values.
pub(crate) const W_REGULAR: u16 = 400;
/// Medium (500): the job title under the name.
pub(crate) const W_MEDIUM: u16 = 500;
/// 650: the name, role titles, education degrees, skill-group labels.
pub(crate) const W_TITLE: u16 = 650;
/// Bold (700): section headers, micro-labels, the `Stack` label.
pub(crate) const W_BOLD: u16 = 700;

/// The `opsz` (optical size) axis is pinned rather than tracked to the type
/// size, at the value the reference rendering was set in: the web Inter the
/// design was drawn with exposes only `wght`, leaving `opsz` at its 14 default
/// for every size on the sheet. Left to itself Typst would derive the axis from
/// each run's point size, which sets the display sizes on a different drawing
/// of the face than the design has.
pub(crate) const OPTICAL_SIZE: f64 = 14.0;

// ---- typography sizes, in design pixels ----

/// Name in the header band.
pub(crate) const FS_NAME: Px = 27.0;
/// Job title beneath the name.
pub(crate) const FS_HEADLINE: Px = 13.0;
/// Availability pill.
pub(crate) const FS_AVAIL: Px = 9.5;
/// Main-column section headers.
pub(crate) const FS_SECTION: Px = 10.0;
/// Sidebar section headers.
pub(crate) const FS_SECTION_SM: Px = 9.5;
/// Experience role titles.
pub(crate) const FS_TITLE: Px = 12.0;
/// Dates, and the institution line under an education degree.
pub(crate) const FS_META: Px = 10.0;
/// The organisation · location line under a role title.
pub(crate) const FS_ORG: Px = 10.5;
/// The professional summary.
pub(crate) const FS_BODY: Px = 11.0;
/// Experience bullets.
pub(crate) const FS_BULLET: Px = 10.0;
/// The `Stack:` run under an entry.
pub(crate) const FS_STACK: Px = 9.5;
/// Sidebar values: contact rows, education degrees, languages.
pub(crate) const FS_SIDEBAR: Px = 10.5;
/// Skill-group labels inside the sidebar.
pub(crate) const FS_SKILL_LABEL: Px = 10.0;
/// Uppercase contact micro-labels (`EMAIL`, `WEB`, …).
pub(crate) const FS_LABEL: Px = 8.0;
/// Skill-chip text.
pub(crate) const FS_TAG: Px = 9.5;

// ---- letter spacing, in em ----

/// The name is set slightly tight, the way a display size wants to be.
pub(crate) const TRACK_NAME: f64 = -0.015;
/// Uppercase section headers open up, so the caps read as a label and not as a
/// word.
pub(crate) const TRACK_SECTION: f64 = 0.09;
/// Uppercase contact micro-labels, a shade tighter than the section headers.
pub(crate) const TRACK_LABEL: f64 = 0.07;

// ---- line heights, as CSS `line-height` multiples ----

/// The name, set tighter than its own font box the way a display size wants.
pub(crate) const LH_NAME: f64 = 1.1;
/// The summary paragraph — the airiest text on the sheet.
pub(crate) const LH_BODY: f64 = 1.45;
/// Everything the design leaves at `line-height: normal`.
pub(crate) const LH_NORMAL: f64 = FONT_LINE_EM;

// ---- fixed geometry, in design pixels ----

/// Page trim: A4.
pub(crate) const PAGE_W: Px = 794.0;
/// Page trim: A4.
pub(crate) const PAGE_H: Px = 1123.0;
/// Page margins. Fixed rather than density-driven: the design sets the same
/// trim on both sheets, and the ladder buys its room from the rhythm inside.
pub(crate) const PAGE_PAD_X: Px = 44.0;
/// Top page margin.
pub(crate) const PAGE_PAD_TOP: Px = 34.0;
/// Bottom page margin.
pub(crate) const PAGE_PAD_BOTTOM: Px = 28.0;
/// Sidebar column width.
pub(crate) const SIDEBAR_W: Px = 214.0;

/// Every hairline the design draws: chip, pill, QR and photo borders, and the
/// section and entry rules.
pub(crate) const HAIRLINE: Px = 1.0;
/// The header rule is the one heavier line on the sheet.
pub(crate) const HEADER_RULE_H: Px = 1.5;

/// Sidebar block corner radius.
pub(crate) const RADIUS_SIDEBAR: Px = 8.0;
/// Skill-chip, QR-frame and photo-frame corner radius.
pub(crate) const RADIUS_TAG: Px = 4.0;

/// Availability pill: horizontal inset.
pub(crate) const PILL_PAD_X: Px = 10.0;
/// Availability pill: vertical inset.
pub(crate) const PILL_PAD_Y: Px = 4.0;

/// Side of the QR module field, excluding its white frame.
pub(crate) const QR_SIZE: Px = 50.0;
/// White margin between the QR modules and their frame.
pub(crate) const QR_PAD: Px = 3.0;
/// Quiet-zone width, in QR modules, rendered inside [`QR_SIZE`].
pub(crate) const QR_QUIET_MODULES: u32 = 1;

/// Application-photo frame width (German sheet only).
pub(crate) const PHOTO_W: Px = 120.0;
/// Application-photo frame height — the 3:4 portrait a German application
/// expects.
pub(crate) const PHOTO_H: Px = 156.0;

/// Gap between the header's QR frame and the name stack.
pub(crate) const HEADER_GAP_QR: Px = 14.0;
/// Minimum gap between the name stack and the availability pill.
pub(crate) const HEADER_GAP_PILL: Px = 16.0;
/// Gap between the name and the job title beneath it.
pub(crate) const NAME_TO_ROLE: Px = 4.0;
/// Gap between a contact micro-label and its value, and between an education
/// degree and its institution line.
pub(crate) const LABEL_TO_VALUE: Px = 1.0;
/// Gap between a role title row and the organisation line beneath it.
pub(crate) const TITLE_TO_ORG: Px = 1.0;
/// Column gutter between the sidebar and the main column.
pub(crate) const COL_GAP: Px = 22.0;
/// Gap between a skill-group label and its chips.
pub(crate) const SKILL_LABEL_GAP: Px = 4.0;
/// Gap between a bullet's `–` marker and its text. Bullets hang, so this is
/// also the indent every wrapped line of that bullet keeps.
pub(crate) const BULLET_INDENT: Px = 7.0;
/// Gutter between a role title and the date range opposite it.
pub(crate) const ENTRY_DATE_GAP: Px = 12.0;
/// Gap below the application photo.
pub(crate) const PHOTO_GAP: Px = 10.0;
/// Gap between the two language rows.
pub(crate) const LANG_GAP: Px = 3.0;
/// Gap above a main-column section header (the first one has none).
pub(crate) const MAIN_SECTION_TOP: Px = 13.0;

/// The gaps the two sheets of the design set differently — the Comfortable
/// rhythm of the English sheet against the Compact rhythm of the German one.
///
/// Every field is a design pixel; [`Layout::sp`] picks the table and
/// [`Layout::pt`] converts.
pub(crate) struct Spacing {
    /// Above the header rule.
    pub(crate) header_rule_above: Px,
    /// Below the header rule, before the two-column grid.
    pub(crate) header_rule_below: Px,
    /// Sidebar block vertical inset.
    pub(crate) sidebar_pad_y: Px,
    /// Sidebar block horizontal inset.
    pub(crate) sidebar_pad_x: Px,
    /// Above every sidebar section header but the first.
    pub(crate) side_section_top: Px,
    /// Contact header to its first row.
    pub(crate) contact_head_gap: Px,
    /// Between contact rows.
    pub(crate) contact_row_gap: Px,
    /// Skills header to its first group.
    pub(crate) skills_head_gap: Px,
    /// Between skill groups.
    pub(crate) skill_group_gap: Px,
    /// Between skill chips, across and down.
    pub(crate) chip_gap: Px,
    /// Skill-chip horizontal inset.
    pub(crate) chip_pad_x: Px,
    /// Skill-chip vertical inset.
    pub(crate) chip_pad_y: Px,
    /// Education header to its first entry.
    pub(crate) edu_head_gap: Px,
    /// Between education entries.
    pub(crate) edu_gap: Px,
    /// Languages header to its first row.
    pub(crate) lang_head_gap: Px,
    /// Above a main-column section rule.
    pub(crate) main_rule_above: Px,
    /// Below a main-column section rule.
    pub(crate) main_rule_below: Px,
    /// Organisation line to the first bullet.
    pub(crate) bullets_top: Px,
    /// Between bullets.
    pub(crate) bullet_gap: Px,
    /// Above the `Stack:` run.
    pub(crate) stack_top: Px,
    /// Above *and* below the hairline between experience entries.
    pub(crate) divider_band: Px,
    /// Bullet line height, as a CSS `line-height` multiple.
    pub(crate) lh_bullet: f64,
}

/// The English sheet's rhythm: the design at rest.
pub(crate) const COMFORTABLE: Spacing = Spacing {
    header_rule_above: 10.0,
    header_rule_below: 14.0,
    sidebar_pad_y: 16.0,
    sidebar_pad_x: 14.0,
    side_section_top: 18.0,
    contact_head_gap: 10.0,
    contact_row_gap: 9.0,
    skills_head_gap: 9.0,
    skill_group_gap: 9.0,
    chip_gap: 4.0,
    chip_pad_x: 6.0,
    chip_pad_y: 2.0,
    edu_head_gap: 9.0,
    edu_gap: 7.0,
    lang_head_gap: 8.0,
    main_rule_above: 5.0,
    main_rule_below: 8.0,
    bullets_top: 5.0,
    bullet_gap: 3.0,
    stack_top: 5.0,
    divider_band: 7.0,
    lh_bullet: 1.36,
};

/// The German sheet's rhythm: the same design squeezed by one step, which is
/// what the longer German prose (and, when supplied, the application photo)
/// needs to stay on one page.
pub(crate) const COMPACT: Spacing = Spacing {
    header_rule_above: 8.0,
    header_rule_below: 12.0,
    sidebar_pad_y: 14.0,
    sidebar_pad_x: 12.0,
    side_section_top: 13.0,
    contact_head_gap: 8.0,
    contact_row_gap: 7.0,
    skills_head_gap: 7.0,
    skill_group_gap: 7.0,
    chip_gap: 3.0,
    chip_pad_x: 5.0,
    chip_pad_y: 1.5,
    edu_head_gap: 7.0,
    edu_gap: 5.0,
    lang_head_gap: 8.0,
    main_rule_above: 4.0,
    main_rule_below: 7.0,
    bullets_top: 4.0,
    bullet_gap: 2.0,
    stack_top: 4.0,
    divider_band: 4.0,
    lh_bullet: 1.33,
};

/// Scaled token resolver: converts design pixels to Typst lengths and selects
/// the density table.
#[derive(Clone, Copy)]
pub(crate) struct Layout {
    /// Uniform type and spacing multiplier, `1.0` at the design size.
    pub(crate) scale: f64,
    /// Whether the [`COMPACT`] spacing table is in force.
    pub(crate) dense: bool,
}

impl Layout {
    /// A scaled length literal in points, e.g. `"7.500pt"`.
    pub(crate) fn pt(&self, px: Px) -> String {
        format!("{:.3}pt", px * PT_PER_PX * self.scale)
    }

    /// An *unscaled* length literal in points, for the page trim: the sheet is
    /// A4 whatever the type does.
    pub(crate) fn page_pt(px: Px) -> String {
        format!("{:.3}pt", px * PT_PER_PX)
    }

    /// The spacing table this density selects.
    pub(crate) fn sp(&self) -> &'static Spacing {
        if self.dense { &COMPACT } else { &COMFORTABLE }
    }

    /// The `(top-edge, bottom-edge)` pair that gives a run of text the CSS line
    /// box `line_height` would give it, as em literals.
    ///
    /// A browser centres the font box inside the line box and splits the
    /// difference into half-leading above and below; Typst instead stacks lines
    /// between a top edge and a bottom edge with `leading` in between. Folding
    /// the half-leading into the two edges and running the document at zero
    /// leading makes the two models agree exactly — a line box is then
    /// `line_height` tall, consecutive baselines sit `line_height` apart, and
    /// every `#v()` in the template means the CSS margin it was copied from.
    pub(crate) fn edges(line_height: f64) -> (String, String) {
        let half_leading = (line_height - FONT_LINE_EM) / 2.0;
        (
            format!("{:.4}em", FONT_ASCENDER_EM + half_leading),
            // Below the baseline is negative, the sign Typst's own descender
            // metric carries.
            format!("{:.4}em", -(FONT_DESCENDER_EM + half_leading)),
        )
    }

    /// The gutter between two skill chips, across and down.
    ///
    /// Typst centres a border on the box edge, so a chip paints
    /// [`HAIRLINE`]`/2` past its layout box on every side and two neighbours
    /// eat one whole hairline of whatever sits between them. Adding it back is
    /// what makes the *painted* gap the one the design asks for — and, since
    /// the same correction goes into the horizontal gutter, what makes a chip's
    /// advance match the CSS border box it was drawn from, so the rows wrap
    /// where the design wraps them.
    ///
    /// A chip box rests on the baseline and contributes no descender of its
    /// own, which is why the vertical case is the plain gutter rather than a
    /// gutter minus a descender.
    pub(crate) fn chip_gutter(&self, gap: Px) -> String {
        self.pt(gap + HAIRLINE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pt_converts_design_pixels_at_96_dpi() {
        let l = Layout {
            scale: 1.0,
            dense: false,
        };
        // The A4 sheet is 794 × 1123 px, i.e. 595.5 × 842.25 pt.
        assert_eq!(l.pt(PAGE_W), "595.500pt");
        assert_eq!(l.pt(16.0), "12.000pt");
    }

    #[test]
    fn scale_applies_to_type_but_not_to_the_page_trim() {
        let l = Layout {
            scale: 0.5,
            dense: false,
        };
        assert_eq!(l.pt(16.0), "6.000pt");
        assert_eq!(Layout::page_pt(16.0), "12.000pt");
    }

    #[test]
    fn normal_line_height_leaves_the_font_box_untouched() {
        let (top, bottom) = Layout::edges(LH_NORMAL);
        assert_eq!(top, "0.9688em");
        assert_eq!(bottom, "-0.2412em");
    }

    #[test]
    fn edges_span_exactly_the_requested_line_height() {
        for line_height in [LH_NAME, LH_BODY, LH_NORMAL, 1.36, 1.33] {
            let (top, bottom) = Layout::edges(line_height);
            let em = |s: String| s.trim_end_matches("em").parse::<f64>().expect("em literal");
            let span = em(top) - em(bottom);
            assert!(
                (span - line_height).abs() < 1e-3,
                "{line_height} spans {span}"
            );
        }
    }

    #[test]
    fn a_tight_line_height_pulls_the_edges_inside_the_font_box() {
        // The name is set below `normal`, so both edges move toward the
        // baseline rather than away from it.
        let (top, bottom) = Layout::edges(LH_NAME);
        assert!(top.trim_end_matches("em").parse::<f64>().expect("em") < FONT_ASCENDER_EM);
        assert!(bottom.trim_end_matches("em").parse::<f64>().expect("em") > -FONT_DESCENDER_EM);
    }

    #[test]
    fn density_selects_the_matching_spacing_table() {
        let comfortable = Layout {
            scale: 1.0,
            dense: false,
        };
        let compact = Layout {
            scale: 1.0,
            dense: true,
        };
        assert_eq!(
            comfortable.pt(comfortable.sp().divider_band),
            comfortable.pt(COMFORTABLE.divider_band),
        );
        assert_eq!(
            compact.pt(compact.sp().divider_band),
            compact.pt(COMPACT.divider_band),
        );
        // Compact is tighter everywhere it differs at all.
        const {
            assert!(COMPACT.divider_band < COMFORTABLE.divider_band);
            assert!(COMPACT.lh_bullet < COMFORTABLE.lh_bullet);
        }
    }

    #[test]
    fn chip_gutter_covers_the_gap_plus_the_two_stroke_halves() {
        let l = Layout {
            scale: 1.0,
            dense: false,
        };
        assert_eq!(l.chip_gutter(COMFORTABLE.chip_gap), l.pt(4.0 + HAIRLINE));
        assert_eq!(l.chip_gutter(COMPACT.chip_gap), l.pt(3.0 + HAIRLINE));
    }
}
