//! Build-time caching for `repos.json`.
//!
//! `repos.json` is generated during the build, so to avoid hitting the GitHub
//! API (and its rate limits) on every rebuild, a freshly generated file is
//! reused while it is still "fresh". The freshness window is longer on CI than
//! on a developer machine.
//!
//! Freshness is derived purely from the file's own `generated_at` timestamp, so
//! it works whether the file was produced locally or restored from a CI cache.

use std::fs;
use std::path::Path;

use portfolio_data::ReposFile;
use time::Duration;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

/// How long a generated `repos.json` stays fresh on CI.
pub const CI_TTL: Duration = Duration::hours(10);
/// How long a generated `repos.json` stays fresh on a developer machine.
pub const LOCAL_TTL: Duration = Duration::minutes(60);

/// Returns the cache time-to-live for the current environment: [`CI_TTL`] when
/// running on CI (the `CI` env var is set to a non-empty value), [`LOCAL_TTL`]
/// otherwise.
pub fn ttl_for_env() -> Duration {
    if is_ci() { CI_TTL } else { LOCAL_TTL }
}

/// `true` when the `CI` environment variable is set to a non-empty value, the
/// de-facto standard signal used by GitHub Actions and most other CI systems.
fn is_ci() -> bool {
    std::env::var("CI").map(|v| !v.is_empty()).unwrap_or(false)
}

/// Reads the cached `repos.json` at `path` and reports whether it is still fresh
/// relative to `now` and `ttl`. A missing or malformed file is treated as stale
/// (returns `false`) so the caller falls back to fetching.
pub fn is_cached_fresh(path: &Path, now: OffsetDateTime, ttl: Duration) -> bool {
    let Ok(contents) = fs::read_to_string(path) else {
        return false;
    };
    let Ok(file) = serde_json::from_str::<ReposFile>(&contents) else {
        return false;
    };
    is_fresh(&file.generated_at, now, ttl)
}

/// Pure freshness check: `true` when `generated_at` (an RFC 3339 timestamp) is
/// within `ttl` of `now`. A missing or unparsable timestamp is treated as stale
/// (returns `false`), forcing a refetch.
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
