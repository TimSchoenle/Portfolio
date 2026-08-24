//! When a `repos.json` already on disk is good enough to skip the API call.
//!
//! Freshness comes from the file's own `generated_at` stamp and nothing else — not the mtime,
//! not a marker file — so a listing restored from a CI cache is judged by when it was generated
//! rather than by when it was unpacked.

use std::fs;
use std::path::Path;

use portfolio_data::ReposFile;
use time::Duration;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

/// How long a generated `repos.json` stays fresh on CI: ten hours, so one listing serves the
/// image builds of a working day.
pub const CI_TTL: Duration = Duration::hours(10);
/// How long a generated `repos.json` stays fresh on a developer machine.
pub const LOCAL_TTL: Duration = Duration::minutes(60);

/// [`CI_TTL`] on CI, [`LOCAL_TTL`] anywhere else.
pub fn ttl_for_env() -> Duration {
    if is_ci() { CI_TTL } else { LOCAL_TTL }
}

/// `true` when the `CI` environment variable is set to a non-empty value, the
/// de-facto standard signal used by GitHub Actions and most other CI systems.
fn is_ci() -> bool {
    std::env::var("CI").map(|v| !v.is_empty()).unwrap_or(false)
}

/// Whether the `repos.json` at `path` was generated within `ttl` of `now`.
///
/// Missing, unreadable and malformed all answer `false`, so every way of having no usable
/// listing leads to the same fetch.
pub fn is_cached_fresh(path: &Path, now: OffsetDateTime, ttl: Duration) -> bool {
    let Ok(contents) = fs::read_to_string(path) else {
        return false;
    };
    let Ok(file) = serde_json::from_str::<ReposFile>(&contents) else {
        return false;
    };
    is_fresh(&file.generated_at, now, ttl)
}

/// Whether `generated_at`, an RFC 3339 timestamp, is less than `ttl` before `now`.
///
/// Exactly `ttl` old is stale. A timestamp in the future is fresh, so clock skew between the
/// machine that wrote the file and the one reading it does not cost an API call.
///
/// An unparsable stamp is stale, which is the reading that refetches rather than the one that
/// trusts a file nothing can date.
pub fn is_fresh(generated_at: &str, now: OffsetDateTime, ttl: Duration) -> bool {
    match OffsetDateTime::parse(generated_at, &Rfc3339) {
        Ok(generated) => now - generated < ttl,
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> OffsetDateTime {
        OffsetDateTime::parse("2026-06-28T12:00:00Z", &Rfc3339).unwrap()
    }

    #[test]
    fn recent_timestamp_is_fresh_within_ttl() {
        // 30 minutes old, well inside the 60-minute local window.
        let stamp = "2026-06-28T11:30:00Z";
        assert!(is_fresh(stamp, now(), LOCAL_TTL));
        assert!(is_fresh(stamp, now(), CI_TTL));
    }

    #[test]
    fn old_timestamp_is_stale_past_ttl() {
        // 2 hours old: stale for the 60-minute local window, fresh for CI's 10h.
        let stamp = "2026-06-28T10:00:00Z";
        assert!(!is_fresh(stamp, now(), LOCAL_TTL));
        assert!(is_fresh(stamp, now(), CI_TTL));
    }

    #[test]
    fn timestamp_exactly_at_ttl_is_stale() {
        // Exactly 60 minutes old: not strictly within the window.
        let stamp = "2026-06-28T11:00:00Z";
        assert!(!is_fresh(stamp, now(), LOCAL_TTL));
    }

    #[test]
    fn unparsable_or_empty_timestamp_is_stale() {
        assert!(!is_fresh("", now(), CI_TTL));
        assert!(!is_fresh("not-a-timestamp", now(), CI_TTL));
    }

    #[test]
    fn future_timestamp_is_treated_as_fresh() {
        // Clock skew should not trigger an unnecessary refetch.
        let stamp = "2026-06-28T12:30:00Z";
        assert!(is_fresh(stamp, now(), LOCAL_TTL));
    }
}
