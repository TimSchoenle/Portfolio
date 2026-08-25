//! Dynamic single-page fitting. Recent roles keep full bullets while older
//! roles get fewer, so content is condensed before the type is shrunk below
//! readable sizes. The order is: prefer detail, then tighten density
//! (Comfortable → Compact), and only then ease the type scale.
//!
//! Page count comes straight from Typst's compiled [`typst_layout::PagedDocument`]
//! (`pages.len()`), so fitting needs no PDF parsing; the PDF is exported only
//! for the scale that actually fits.

use crate::style::Layout;
use crate::template::{Photo, build_typ};
use crate::translations::Translations;
use crate::world::{self, Asset};

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
    /// The three leading roles keep every bullet; the rest keep two.
    Condensed,
    /// The two leading roles keep every bullet; the rest keep two.
    Compact,
}

impl Detail {
    /// How many leading entries of the sorted work history keep every bullet at this level.
    ///
    /// The two condensing levels differed by whether a role was ongoing rather than by how many
    /// roles they protected. That exemption never fired — both ongoing entries already sort into
    /// the first two positions — and it ranked the wrong role: the freelance engagement is the
    /// ongoing one, and the senior employment trimmed in its place is not. Protecting a count
    /// instead makes the two levels a gradient, and moves the most recent senior role out of the
    /// first thing the ladder cuts.
    fn protected(self) -> usize {
        match self {
            Detail::Full => usize::MAX,
            Detail::Condensed => 3,
            Detail::Compact => 2,
        }
    }

    /// How many bullets the entry at `entry_index` of the sorted work history renders at this
    /// detail level.
    ///
    /// `entry_index` is the position in [`portfolio_data::experiences_sorted`], so "leading" means
    /// the front of that order rather than the most recent by date.
    ///
    /// [`Experience::resume_bullet_cap`](portfolio_data::Experience::resume_bullet_cap) is applied
    /// last and only ever lowers the result, so a capped entry stays capped at every level.
    pub(crate) fn bullet_count(self, entry_index: usize, e: &portfolio_data::Experience) -> u8 {
        let n = if entry_index < self.protected() {
            e.bullet_count
        } else {
            e.bullet_count.min(2)
        };
        // Apply the resume-only bullet cap from the shared config: some older
        // roles render fewer bullets in the PDF, while the website still shows
        // every bullet via `Experience::bullet_count`.
        n.min(e.resume_bullet_cap.unwrap_or(u8::MAX))
    }

    /// The phrase the generator prints for this level, for the line naming what it took to fit.
    pub(crate) fn describe(self) -> &'static str {
        match self {
            Detail::Full => "full detail",
            Detail::Condensed => "roles after the third condensed",
            Detail::Compact => "roles after the second condensed",
        }
    }
}

/// A resume that fit on one page, and what it cost to get there.
pub(crate) struct Fitted {
    /// The exported PDF.
    pub(crate) bytes: Vec<u8>,
    /// The type scale it fit at, between [`ABSOLUTE_MIN_SCALE`] and `1.0`.
    pub(crate) scale: f64,
    /// Whether the Compact spacing preset was needed.
    pub(crate) dense: bool,
    /// How much of the experience section survived.
    pub(crate) detail: Detail,
}

/// Fitting ladder: full detail at Comfortable density, then condensed, then
/// Compact density, condensing further, and only last letting the scale drift
/// toward the floor.
pub(crate) fn fit_single_page(
    t: &Translations,
    lang: &str,
    photo: Option<Photo<'_>>,
    assets: &[Asset],
) -> Result<Fitted, String> {
    // (dense, detail, lo, hi)
    let attempts = [
        (false, Detail::Full, PREFERRED_MIN_SCALE, 1.0),
        (false, Detail::Condensed, PREFERRED_MIN_SCALE, 1.0),
        (true, Detail::Full, PREFERRED_MIN_SCALE, 1.0),
        (true, Detail::Condensed, PREFERRED_MIN_SCALE, 1.0),
        (true, Detail::Compact, PREFERRED_MIN_SCALE, 1.0),
        (
            true,
            Detail::Compact,
            ABSOLUTE_MIN_SCALE,
            PREFERRED_MIN_SCALE,
        ),
    ];
    for (dense, detail, lo, hi) in attempts {
        if let Some((bytes, scale)) =
            largest_fitting_scale(t, lang, dense, detail, lo, hi, photo, assets)?
        {
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
#[expect(
    clippy::too_many_arguments,
    reason = "one search over the ladder's own coordinates, all of which vary per attempt"
)]
fn largest_fitting_scale(
    t: &Translations,
    lang: &str,
    dense: bool,
    detail: Detail,
    lo: f64,
    hi: f64,
    photo: Option<Photo<'_>>,
    assets: &[Asset],
) -> Result<Option<(Vec<u8>, f64)>, String> {
    // Compile + export at `scale`, returning whether it fit on one page.
    let render_at = |scale: f64| -> Result<(bool, Vec<u8>), String> {
        let source = build_typ(t, Layout { scale, dense }, detail, lang, photo);
        let (pages, bytes) = world::render(source, assets)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use portfolio_data::{Experience, YearMonth};

    fn entry(bullet_count: u8, resume_bullet_cap: Option<u8>, ongoing: bool) -> Experience {
        Experience {
            id: "test",
            location: "Remote",
            start: YearMonth {
                year: 2020,
                month: 1,
            },
            end: if ongoing {
                None
            } else {
                Some(YearMonth {
                    year: 2022,
                    month: 1,
                })
            },
            bullet_count,
            resume_bullet_cap,
            tech: &[],
        }
    }

    #[test]
    fn full_detail_keeps_every_bullet_regardless_of_recency() {
        let detail = Detail::Full;
        let e = entry(3, None, false);
        // Index well past the two most recent roles: still full.
        assert_eq!(detail.bullet_count(7, &e), 3);
    }

    #[test]
    fn condensed_protects_the_first_three_roles() {
        let detail = Detail::Condensed;
        assert_eq!(detail.bullet_count(0, &entry(3, None, false)), 3);
        // The third entry is the most recent senior employment, and it is what Compact cuts
        // first. Condensed is the rung that still keeps it whole.
        assert_eq!(detail.bullet_count(2, &entry(3, None, false)), 3);
        assert_eq!(detail.bullet_count(3, &entry(3, None, false)), 2);
    }

    #[test]
    fn compact_protects_only_the_first_two_roles() {
        let detail = Detail::Compact;
        assert_eq!(detail.bullet_count(1, &entry(3, None, false)), 3);
        assert_eq!(detail.bullet_count(2, &entry(3, None, false)), 2);
    }

    /// Whether a role is ongoing no longer changes how much of it survives; only its position
    /// does. Both levels are checked at the index where they differ.
    #[test]
    fn condensing_ignores_whether_a_role_is_ongoing() {
        for detail in [Detail::Condensed, Detail::Compact] {
            for index in [0, 2, 5] {
                assert_eq!(
                    detail.bullet_count(index, &entry(3, None, true)),
                    detail.bullet_count(index, &entry(3, None, false)),
                    "ongoing changed the count at index {index}",
                );
            }
        }
    }

    #[test]
    fn resume_bullet_cap_is_always_honored() {
        // Even at full detail for a recent role, the per-entry cap wins.
        assert_eq!(Detail::Full.bullet_count(0, &entry(3, Some(2), false)), 2);
        // The cap never raises the count above what the detail level allows.
        assert_eq!(
            Detail::Compact.bullet_count(9, &entry(3, Some(2), false)),
            2
        );
    }

    #[test]
    fn describe_is_distinct_per_level() {
        let labels = [
            Detail::Full.describe(),
            Detail::Condensed.describe(),
            Detail::Compact.describe(),
        ];
        for label in labels {
            assert!(!label.is_empty());
        }
        assert_ne!(labels[0], labels[1]);
        assert_ne!(labels[1], labels[2]);
    }

    #[test]
    fn preferred_floor_stays_above_absolute_floor() {
        const {
            assert!(PREFERRED_MIN_SCALE > ABSOLUTE_MIN_SCALE);
            assert!(ABSOLUTE_MIN_SCALE > 0.0);
            assert!(PREFERRED_MIN_SCALE <= 1.0);
        }
    }
}
