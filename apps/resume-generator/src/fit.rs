//! Dynamic single-page fitting. Standard one-page practice: recent roles get
//! full bullets, older roles get fewer — content is condensed before type is
//! shrunk below readable sizes. The §9 ordering is honored: prefer detail, then
//! tighten density (Comfortable → Compact), and only then ease the type scale.
//!
//! Page count comes straight from Typst's compiled [`PagedDocument`]
//! (`pages.len()`), so fitting needs no PDF parsing; the PDF is exported only
//! for the scale that actually fits.

use crate::style::Layout;
use crate::template::build_typ;
use crate::translations::Translations;
use crate::world;

/// Preferred lower bound while at Comfortable density: keeps the body type close
/// to its 10 pt design size. The ladder switches to Compact density before
/// pushing the scale below this.
const PREFERRED_MIN_SCALE: f64 = 0.90;
/// Hard floor; below this the generator refuses and fails the build.
const ABSOLUTE_MIN_SCALE: f64 = 0.80;

/// How much detail the experience section carries.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Detail {
    /// Every bullet of every role.
    Full,
    /// Ended roles beyond the two most recent keep their two strongest bullets;
    /// ongoing engagements stay full.
    Condensed,
    /// Every role beyond the two most recent keeps two bullets.
    Compact,
}

impl Detail {
    pub(crate) fn bullet_count(self, entry_index: usize, e: &portfolio_data::Experience) -> u8 {
        let recent = entry_index < 2;
        let ongoing = e.end.is_none();
        let n = match self {
            Detail::Full => e.bullet_count,
            Detail::Condensed if recent || ongoing => e.bullet_count,
            Detail::Compact if recent => e.bullet_count,
            _ => e.bullet_count.min(2),
        };
        // Apply the resume-exclusive bullet cap defined in the shared config:
        // older roles with redundant bullets render fewer in the PDF, while the
        // website still shows every bullet via `Experience::bullet_count`.
        n.min(e.resume_bullet_cap.unwrap_or(u8::MAX))
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
    pub(crate) dense: bool,
    pub(crate) detail: Detail,
}

/// Degradation ladder (§9): full detail at Comfortable density, then condensed,
/// then switch to Compact density before condensing the content further and,
/// last, letting the scale drift toward the floor.
pub(crate) fn fit_single_page(t: &Translations, lang: &str) -> Result<Fitted, String> {
    // (dense, detail, lo, hi)
    let attempts = [
        (false, Detail::Full, PREFERRED_MIN_SCALE, 1.0),
        (false, Detail::Condensed, PREFERRED_MIN_SCALE, 1.0),
        (true, Detail::Full, PREFERRED_MIN_SCALE, 1.0),
        (true, Detail::Condensed, PREFERRED_MIN_SCALE, 1.0),
        (true, Detail::Compact, PREFERRED_MIN_SCALE, 1.0),
        (true, Detail::Compact, ABSOLUTE_MIN_SCALE, PREFERRED_MIN_SCALE),
    ];
    for (dense, detail, lo, hi) in attempts {
        if let Some((bytes, scale)) = largest_fitting_scale(t, lang, dense, detail, lo, hi)? {
            return Ok(Fitted {
                bytes,
                scale,
                dense,
                detail,
            });
        }
    }
    Err("does not fit one page even condensed".to_string())
}

/// Binary-searches the largest scale within `[lo, hi]` that renders as a single
/// page. Returns `Ok(None)` if even `lo` overflows; `Err` on a compile failure.
fn largest_fitting_scale(
    t: &Translations,
    lang: &str,
    dense: bool,
    detail: Detail,
    lo: f64,
    hi: f64,
) -> Result<Option<(Vec<u8>, f64)>, String> {
    // Compile + export at `scale`, returning whether it fit on one page.
    let render_at = |scale: f64| -> Result<(bool, Vec<u8>), String> {
        let source = build_typ(t, Layout { scale, dense }, detail, lang);
        let (pages, bytes) = world::render(source)?;
        Ok((pages <= 1, bytes))
    };

    let (lo_fits, lo_bytes) = render_at(lo)?;
    if !lo_fits {
        return Ok(None);
    }
    let (hi_fits, hi_bytes) = render_at(hi)?;
    if hi_fits {
        return Ok(Some((hi_bytes, hi)));
    }

    // Invariant: `lo` fits, `hi` does not.
    let (mut lo, mut hi) = (lo, hi);
    let mut best = (lo_bytes, lo);
    while hi - lo > 0.01 {
        let mid = (lo + hi) / 2.0;
        let (fits, bytes) = render_at(mid)?;
        if fits {
            best = (bytes, mid);
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Ok(Some(best))
}
