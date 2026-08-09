//! Incremental static regeneration: where rendered pages are cached, and for how long.

use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::Deserialize;

/// The SSR server's incremental render cache.
///
/// Off unless [`cache_dir`](Self::cache_dir) names a directory, and off again at runtime if
/// that directory turns out not to be writable — the server then renders every request fresh
/// rather than failing to start.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct IsrConfig {
    /// Writable directory rendered HTML is cached into. Unset (or empty) disables ISR.
    ///
    /// Keep it *outside* the bundled `public/` asset tree so those content-hashed assets stay
    /// immutable. The image points it at a sub-directory of `/tmp`, which the deployment
    /// already provides as a writable mount even under a read-only root filesystem.
    #[serde(default)]
    pub cache_dir: Option<PathBuf>,
    /// Revalidation interval in seconds. `0` — the default — means a permanent cache.
    ///
    /// Every page renders from compile-time data, so the only thing that changes the output is
    /// a new build, which starts from an empty cache anyway. A positive value opts into a
    /// finite, time-based TTL, which is useful only when a *persistent* cache volume is shared
    /// across deploys.
    #[serde(default)]
    pub ttl_secs: u64,
}

impl IsrConfig {
    /// The cache directory, or `None` when ISR is off.
    ///
    /// An empty value counts as unset: container platforms routinely inject `KEY=` for a
    /// declared-but-unset variable, and that has to mean "ISR off", not "cache into the current
    /// working directory".
    #[must_use]
    pub fn cache_dir(&self) -> Option<&Path> {
        self.cache_dir
            .as_deref()
            .filter(|dir| !dir.as_os_str().is_empty())
    }

    /// The revalidation interval, or `None` for a permanent cache.
    ///
    /// Unlike the environment-variable reader this replaces, an *unparseable* value no longer
    /// falls back to "permanent" — it fails the boot, because figment rejects it before this is
    /// ever called. That is the intended trade: a typo used to silently disable revalidation on
    /// the one deployment shape that needs it, and a container that refuses to start is the
    /// louder half of the failure.
    #[must_use]
    pub fn invalidate_after(&self) -> Option<Duration> {
        (self.ttl_secs > 0).then(|| Duration::from_secs(self.ttl_secs))
    }
}

#[cfg(test)]
mod tests {
    use super::IsrConfig;
    use std::path::{Path, PathBuf};
    use std::time::Duration;

    #[test]
    fn isr_is_off_until_a_cache_directory_is_named() {
        assert_eq!(IsrConfig::default().cache_dir(), None);
    }

    /// `PORTFOLIO_ISR__CACHE_DIR=` is how a container platform spells "declared but unset"; it
    /// must disable ISR rather than cache into the working directory.
    #[test]
    fn an_empty_cache_directory_disables_isr() {
        let config = IsrConfig {
            cache_dir: Some(PathBuf::new()),
            ..IsrConfig::default()
        };
        assert_eq!(config.cache_dir(), None);
    }

    #[test]
    fn a_named_cache_directory_enables_isr() {
        let config = IsrConfig {
            cache_dir: Some(PathBuf::from("/tmp/isr")),
            ..IsrConfig::default()
        };
        assert_eq!(config.cache_dir(), Some(Path::new("/tmp/isr")));
    }

    #[test]
    fn a_zero_ttl_means_a_permanent_cache() {
        assert_eq!(IsrConfig::default().invalidate_after(), None);
    }

    #[test]
    fn a_positive_ttl_opts_into_time_based_revalidation() {
        let config = IsrConfig {
            ttl_secs: 3600,
            ..IsrConfig::default()
        };
        assert_eq!(config.invalidate_after(), Some(Duration::from_secs(3600)));
    }
}
