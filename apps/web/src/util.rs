//! Small cross-target helpers.
//!
//! The "current date" is read from the JS `Date` on the wasm client and from the
//! system clock (`time`) on the server. The two use different zones — the client
//! reports local time, the server UTC — so around midnight, and for a few hours
//! either side of New Year, they can disagree by a day or a year. Only the
//! footer's copyright year and the hero's "years of experience" are derived from
//! them; both are cosmetic, and the worst case is that a freshly hydrated page
//! corrects itself by one. Anything where the exact date matters must not be
//! built on these.

/// The current calendar year (e.g. `2026`).
pub fn current_year() -> i32 {
    #[cfg(feature = "web")]
    {
        js_sys::Date::new_0().get_full_year() as i32
    }
    #[cfg(all(not(feature = "web"), feature = "server"))]
    {
        time::OffsetDateTime::now_utc().year()
    }
    #[cfg(not(any(feature = "web", feature = "server")))]
    {
        2026
    }
}

/// The current calendar month, `1..=12`.
pub fn current_month() -> u8 {
    #[cfg(feature = "web")]
    {
        js_sys::Date::new_0().get_month() as u8 + 1
    }
    #[cfg(all(not(feature = "web"), feature = "server"))]
    {
        u8::from(time::OffsetDateTime::now_utc().month())
    }
    #[cfg(not(any(feature = "web", feature = "server")))]
    {
        1
    }
}
