//! Experience and education timelines, and how their periods are written.

/// A month on the experience and education timelines.
///
/// Deliberately no day. Every date this site renders is `Mon YYYY` or `YYYY`, so a start day
/// would be a fact nobody publishes and nothing corrects.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct YearMonth {
    /// Four-digit calendar year.
    pub year: u16,
    /// `1..=12`. [`format_period`] indexes the localized month names with it.
    pub month: u8,
}

pub(crate) const fn ym(year: u16, month: u8) -> YearMonth {
    YearMonth { year, month }
}

/// One role in the work history.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Experience {
    /// Stable id used as translation key segment (`experience.entries.<id>`).
    /// The localized role, organization and bullets live in the i18n files.
    pub id: &'static str,
    /// Where the role is worked from, e.g. `Remote`. Not localized.
    pub location: &'static str,
    /// First month of the role.
    pub start: YearMonth,
    /// Last month, or `None` while the role is ongoing. Ongoing roles sort ahead of every ended
    /// one in [`experiences_sorted`], however recently the ended one started.
    pub end: Option<YearMonth>,
    /// Number of localized bullets (`…bullets.b1` ..= `…bullets.b<n>`).
    pub bullet_count: u8,
    /// Caps how many localized bullets the generated resume renders
    /// (`None` = uncapped). Only trims the PDF; the website always shows every
    /// bullet via [`bullet_count`](Self::bullet_count). Used to drop a redundant
    /// third bullet from the two oldest roles on the resume.
    pub resume_bullet_cap: Option<u8>,
    /// The chip run under the entry, in the order given.
    pub tech: &'static [&'static str],
}

/// Raw work-history entries, in no meaningful order.
///
/// The site and the resume never read this slice directly. Both go through
/// [`experiences_sorted`], which derives the rendering order from the dates, so an entry appended
/// here lands wherever its dates put it. Localized roles and bullets live in the i18n files.
pub const EXPERIENCE: &[Experience] = &[
    Experience {
        id: "sixtwenty",
        location: "Remote",
        start: ym(2026, 3),
        end: None,
        bullet_count: 2,
        resume_bullet_cap: None,
        tech: &["Java", "TypeScript", "Rust", "Kubernetes", "GitOps"],
    },
    Experience {
        id: "mineplex-studios",
        location: "Remote",
        start: ym(2023, 8),
        end: Some(ym(2026, 3)),
        bullet_count: 3,
        resume_bullet_cap: None,
        tech: &["Java", "TypeScript", "Rust", "Kubernetes", "GitOps"],
    },
    Experience {
        id: "self-employed",
        location: "Remote",
        start: ym(2021, 10),
        end: None,
        bullet_count: 3,
        resume_bullet_cap: None,
        tech: &[
            "Java",
            "Spring Boot",
            "TypeScript",
            "Node.js",
            "Rust",
            "gRPC",
            "Kubernetes",
            "Talos Linux",
        ],
    },
    Experience {
        id: "mineplex-dev",
        location: "Remote",
        start: ym(2021, 10),
        end: Some(ym(2023, 1)),
        bullet_count: 3,
        resume_bullet_cap: Some(2),
        tech: &["Java", "PaperMC", "Bukkit API", "Spigot API"],
    },
    Experience {
        id: "mineplex-qa",
        location: "Remote",
        start: ym(2018, 11),
        end: Some(ym(2023, 1)),
        bullet_count: 3,
        resume_bullet_cap: Some(2),
        tech: &["Java"],
    },
];

/// Work history in the order the site and the resume render it.
///
/// Ongoing roles first, then the rest by descending start date, with the most recent end date
/// breaking equal starts. All of that comes from the dates, so an ongoing role outranks an ended
/// one that started later.
#[must_use]
pub fn experiences_sorted() -> Vec<&'static Experience> {
    let mut out: Vec<&'static Experience> = EXPERIENCE.iter().collect();
    out.sort_by(|a, b| {
        // Ongoing roles (no end) come first.
        a.end
            .is_some()
            .cmp(&b.end.is_some())
            // Then most recent start first.
            .then_with(|| (b.start.year, b.start.month).cmp(&(a.start.year, a.start.month)))
            // Finally, the most recent end breaks equal-start ties.
            .then_with(|| match (a.end, b.end) {
                (Some(ae), Some(be)) => (be.year, be.month).cmp(&(ae.year, ae.month)),
                _ => std::cmp::Ordering::Equal,
            })
    });
    out
}

/// One entry of the education history. Degree and institution are localized.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Education {
    /// Stable id used as translation key segment (`resume.education.<id>`).
    pub id: &'static str,
    /// First month of the programme.
    pub start: YearMonth,
    /// Last month, or `None` while it is ongoing.
    pub end: Option<YearMonth>,
}

/// Education history, newest first. Degrees/institutions are localized.
pub const EDUCATION: &[Education] = &[
    Education {
        id: "uni-konstanz",
        start: ym(2019, 10),
        end: Some(ym(2021, 9)),
    },
    Education {
        id: "abitur",
        start: ym(2016, 9),
        end: Some(ym(2019, 7)),
    },
];

/// A date range in the caller's language, e.g. `Nov 2018 – Jan 2023`.
///
/// `months` are the twelve month abbreviations in order, and `present` is the label an
/// open-ended range gets in place of an end date. A month above twelve, or a `months` shorter
/// than twelve, renders the month blank; month `0` renders the first entry of `months`.
///
/// ```
/// # use portfolio_data::{YearMonth, format_period};
/// let months: Vec<String> = ["Jan", "Feb", "Mar", "Apr", "May", "Jun",
///                            "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"]
///     .iter().map(|m| (*m).to_owned()).collect();
/// let ym = |year, month| YearMonth { year, month };
///
/// assert_eq!(
///     format_period(ym(2018, 11), Some(ym(2023, 1)), &months, "Present"),
///     "Nov 2018 – Jan 2023",
/// );
/// assert_eq!(
///     format_period(ym(2026, 3), None, &months, "Present"),
///     "Mar 2026 – Present",
/// );
/// ```
#[must_use]
pub fn format_period(
    start: YearMonth,
    end: Option<YearMonth>,
    months: &[String],
    present: &str,
) -> String {
    let fmt = |d: YearMonth| {
        let month = months
            .get((d.month as usize).saturating_sub(1))
            .map_or("", String::as_str);
        format!("{month} {year}", year = d.year)
    };
    match end {
        Some(end) => format!("{} – {}", fmt(start), fmt(end)),
        None => format!("{} – {present}", fmt(start)),
    }
}

/// A year-only range, e.g. `2018–2023`, `2026–now`, or a single year when both ends share it.
///
/// Used by the web experience accordion and the resume's education lines; the resume's
/// experience dates use the month-precise [`format_period`]. The en dash is unspaced, as it is
/// between two numbers.
///
/// ```
/// # use portfolio_data::{YearMonth, format_period_years};
/// let ym = |year, month| YearMonth { year, month };
/// assert_eq!(format_period_years(ym(2018, 11), Some(ym(2023, 1)), "now"), "2018–2023");
/// assert_eq!(format_period_years(ym(2026, 3), None, "now"), "2026–now");
/// assert_eq!(format_period_years(ym(2026, 3), Some(ym(2026, 9)), "now"), "2026");
/// ```
#[must_use]
pub fn format_period_years(start: YearMonth, end: Option<YearMonth>, now: &str) -> String {
    match end {
        Some(end) if end.year == start.year => format!("{}", start.year),
        Some(end) => format!("{}–{}", start.year, end.year),
        None => format!("{}–{now}", start.year),
    }
}
