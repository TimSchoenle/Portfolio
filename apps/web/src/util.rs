//! Reading the wall clock, which the two build targets do differently.
//!
//! The "current date" is read from the JS `Date` on the wasm client and from the
//! system clock (`time`) on the server. The two use different zones — the client
//! reports local time, the server UTC — so around midnight, and for a few hours
//! either side of New Year, they can disagree by a day or a year. Only the
//! footer's copyright year, the hero's "years of experience" and the duration
//! badges of ongoing roles are derived from them; all are cosmetic, and the worst case is that a freshly hydrated page
//! corrects itself by one. Anything where the exact date matters must not be
//! built on these.

/// Reports a recoverable problem on the browser console. A no-op outside the wasm client,
/// where the server's own logging covers the same data at start-up.
pub fn console_warn(message: std::fmt::Arguments<'_>) {
    #[cfg(feature = "web")]
    web_sys::console::warn_1(&message.to_string().into());
    #[cfg(not(feature = "web"))]
    let _ = message;
}

/// The current calendar year (e.g. `2026`).
pub fn current_year() -> i32 {
    #[cfg(feature = "web")]
    {
        i32::try_from(js_sys::Date::new_0().get_full_year()).unwrap_or(i32::MAX)
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
        u8::try_from(js_sys::Date::new_0().get_month()).map_or(1, |month| month + 1)
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
