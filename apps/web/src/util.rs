//! Small cross-target helpers.
//!
//! The "current date" is read from the JS `Date` on the wasm client and from
//! the system clock (`time`) on the server. Both resolve to the same real
//! calendar date, so values derived from them match across hydration.

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
