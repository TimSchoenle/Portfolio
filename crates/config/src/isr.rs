//! Incremental static regeneration: where rendered pages are cached, and for how long.

use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::Deserialize;

/// The SSR server's incremental render cache.
///
/// The cache is off unless [`cache_dir`](Self::cache_dir) names a directory. When that directory
/// turns out not to be writable, the server starts anyway without the on-disk cache, and only the
/// in-memory page memo holds rendered pages.
#[derive(Debug, Clone, Default, Deserialize)]
#[cfg_attr(
    feature = "config-schema",
    derive(serde::Serialize, terrace_config::schema::Describe)
)]
pub struct IsrConfig {
    /// The writable directory rendered HTML is cached into. Unset or empty disables ISR.
    ///
    /// Keep it outside the bundled `public/` asset tree so those content-hashed assets stay
    /// immutable. The image points it at a sub-directory of `/tmp`, which the deployment provides
    /// as a writable mount even under a read-only root filesystem.
    #[cfg_attr(
        feature = "config-schema",
        config(note = "ISR off; the image sets `/tmp/isr`")
    )]
    #[serde(default)]
    pub cache_dir: Option<PathBuf>,
    /// The revalidation interval in seconds. Zero means a permanent cache.
    ///
    /// A page changes only with a new build or a new `legal.*` configuration, and the server keeps
    /// whatever the directory holds at boot. Permanent suits the image's default, a directory that
    /// starts empty with every container. A cache volume that outlives a deploy needs a positive
    /// value, or it keeps serving pages rendered by the earlier build or configuration.
    #[cfg_attr(feature = "config-schema", config(note = "permanent"))]
    #[serde(default)]
    pub ttl_secs: u64,
}

impl IsrConfig {
    /// Returns the cache directory, or `None` when ISR is off.
    ///
    /// An empty value counts as unset, never as the current working directory.
    #[must_use]
    pub fn cache_dir(&self) -> Option<&Path> {
        self.cache_dir
            .as_deref()
            .filter(|dir| !dir.as_os_str().is_empty())
    }

    /// Returns the revalidation interval, or `None` for a permanent cache.
    ///
    /// ```
    /// # use portfolio_config::IsrConfig;
    /// # use std::time::Duration;
    /// let isr = IsrConfig { ttl_secs: 60, ..IsrConfig::default() };
    /// assert_eq!(isr.invalidate_after(), Some(Duration::from_secs(60)));
    /// assert_eq!(IsrConfig::default().invalidate_after(), None);
    /// ```
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
