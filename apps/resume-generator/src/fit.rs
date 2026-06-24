//! Dynamic single-page fitting. Standard one-page practice: recent roles get
//! full bullets, older roles get fewer — content is condensed before type is
//! shrunk below readable sizes. A binary search then finds the largest scale
//! that still renders as a single page.

use genpdf::fonts::{FontData, FontFamily};

use crate::document::build_resume;
use crate::style::Layout;
use crate::translations::Translations;

/// Scales ≥ this keep the body font at ~9 pt — the readability floor
/// recommended for human review. Content is condensed before going lower.
const PREFERRED_MIN_SCALE: f64 = 0.90;
/// Hard floor; below this the generator refuses and fails the build.
const ABSOLUTE_MIN_SCALE: f64 = 0.82;

/// How much detail the experience section carries.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Detail {
    /// Every bullet of every role.
    Full,
    /// Ended roles beyond the two most recent keep their two strongest
    /// bullets; ongoing engagements stay full.
    Condensed,
    /// Every role beyond the two most recent keeps two bullets.
    Compact,
}

impl Detail {
    pub(crate) fn bullet_count(self, entry_index: usize, e: &portfolio_data::Experience) -> u8 {
        let recent = entry_index < 2;
        let ongoing = e.end.is_none();
        match self {
            Detail::Full => e.bullet_count,
            Detail::Condensed if recent || ongoing => e.bullet_count,
            Detail::Compact if recent => e.bullet_count,
            _ => e.bullet_count.min(2),
        }
    }

    pub(crate) fn describe(self) -> &'static str {
        match self {
            Detail::Full => "full detail",
            Detail::Condensed => "older ended roles condensed",
            Detail::Compact => "older roles condensed",
        }
    }
}

pub(crate) struct Fitted {
    pub(crate) bytes: Vec<u8>,
    pub(crate) scale: f64,
    pub(crate) detail: Detail,
}

/// Degradation ladder: full detail at readable scales, then condensed at
/// readable scales, then condensed down to the hard floor.
pub(crate) fn fit_single_page(fonts: &FontFamily<FontData>, t: &Translations) -> Option<Fitted> {
    let attempts = [
        (Detail::Full, PREFERRED_MIN_SCALE, 1.0),
        (Detail::Condensed, PREFERRED_MIN_SCALE, 1.0),
        (Detail::Compact, PREFERRED_MIN_SCALE, 1.0),
        (Detail::Compact, ABSOLUTE_MIN_SCALE, PREFERRED_MIN_SCALE),
    ];
    attempts.into_iter().find_map(|(detail, lo, hi)| {
        largest_fitting_scale(fonts, t, detail, lo, hi).map(|(bytes, scale)| Fitted {
            bytes,
            scale,
            detail,
        })
    })
}

/// Binary-searches the largest scale within `[lo, hi]` that renders as a
/// single page. Returns `None` if even `lo` overflows.
fn largest_fitting_scale(
    fonts: &FontFamily<FontData>,
    t: &Translations,
    detail: Detail,
    lo: f64,
    hi: f64,
) -> Option<(Vec<u8>, f64)> {
    let render_at = |scale: f64| -> Option<(Vec<u8>, bool)> {
        let doc = build_resume(fonts.clone(), t, Layout { scale }, detail);
        let mut bytes = Vec::new();
        doc.render(&mut bytes).ok()?;
        let fits = page_count(&bytes) <= 1;
        Some((bytes, fits))
    };

    let (lo_bytes, lo_fits) = render_at(lo)?;
    if !lo_fits {
        return None;
    }
    if let Some((hi_bytes, true)) = render_at(hi) {
        return Some((hi_bytes, hi));
    }

    // Invariant: `lo` fits, `hi` does not.
    let (mut lo, mut hi) = (lo, hi);
    let mut best = (lo_bytes, lo);
    while hi - lo > 0.01 {
        let mid = (lo + hi) / 2.0;
        match render_at(mid) {
            Some((bytes, true)) => {
                best = (bytes, mid);
                lo = mid;
            }
            _ => hi = mid,
        }
    }
    Some(best)
}

/// Number of pages, read from the `/Count N` entry of the PDF page tree.
fn page_count(bytes: &[u8]) -> usize {
    let needle = b"/Count ";
    bytes
        .windows(needle.len())
        .position(|w| w == needle)
        .map(|i| {
            bytes[i + needle.len()..]
                .iter()
                .take_while(|b| b.is_ascii_digit())
                .fold(0usize, |acc, b| acc * 10 + (b - b'0') as usize)
        })
        .unwrap_or(usize::MAX)
}
