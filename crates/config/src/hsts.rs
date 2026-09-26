//! The `Strict-Transport-Security` header the SSR server sends.

use core::fmt;

use serde::Deserialize;

/// The smallest `max-age` the HSTS preload list accepts: one year.
const PRELOAD_MIN_MAX_AGE_SECS: u64 = 31_536_000;

/// How the SSR server builds `Strict-Transport-Security`.
///
/// The defaults reproduce the header this site has always sent — one year, subdomains included,
/// preload requested — so an empty block changes nothing. They are configuration rather than
/// constants because both flags reach beyond this service: `includeSubDomains` binds every host
/// under the domain to HTTPS, and `preload` asks browsers to ship that rule in their binaries,
/// which a deployment on someone else's domain must not inherit from an image default.
// No `deny_unknown_fields`, for the reasons given on `AssetsConfig`.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(
    feature = "config-schema",
    derive(serde::Serialize, terrace_config::schema::Describe)
)]
pub struct HstsConfig {
    /// How long, in seconds, a browser keeps to HTTPS after seeing the header. Zero clears it.
    ///
    /// Zero is the documented way to *withdraw* HSTS: browsers drop the host from their list on
    /// the next response. It is refused together with `preload`, which that would contradict.
    #[serde(default = "HstsConfig::default_max_age_secs")]
    pub max_age_secs: u64,
    /// Apply the policy to every subdomain as well (`includeSubDomains`).
    #[serde(default = "HstsConfig::default_flag")]
    pub include_subdomains: bool,
    /// Ask to be included in the browsers' built-in HSTS preload list (`preload`).
    ///
    /// The list's own requirements are enforced at boot: `include_subdomains` on and a
    /// `max_age_secs` of at least one year.
    #[serde(default = "HstsConfig::default_flag")]
    pub preload: bool,
}

impl HstsConfig {
    /// One year; see the field.
    const fn default_max_age_secs() -> u64 {
        PRELOAD_MIN_MAX_AGE_SECS
    }

    /// Both flags are on by default; see the type.
    const fn default_flag() -> bool {
        true
    }

    /// Reject a combination the preload list would refuse, so it fails the boot rather than a
    /// submission months later.
    ///
    /// # Errors
    ///
    /// [`HstsConfigError`] naming the unmet requirement.
    pub const fn validate(&self) -> Result<(), HstsConfigError> {
        if !self.preload {
            return Ok(());
        }
        if !self.include_subdomains {
            return Err(HstsConfigError::PreloadWithoutSubdomains);
        }
        if self.max_age_secs < PRELOAD_MIN_MAX_AGE_SECS {
            return Err(HstsConfigError::PreloadMaxAgeTooShort);
        }
        Ok(())
    }

    /// The header value, e.g. `max-age=31536000; includeSubDomains; preload`.
    #[must_use]
    pub fn header_value(&self) -> String {
        let mut value = format!("max-age={}", self.max_age_secs);
        if self.include_subdomains {
            value.push_str("; includeSubDomains");
        }
        if self.preload {
            value.push_str("; preload");
        }
        value
    }
}

impl Default for HstsConfig {
    fn default() -> Self {
        Self {
            max_age_secs: Self::default_max_age_secs(),
            include_subdomains: Self::default_flag(),
            preload: Self::default_flag(),
        }
    }
}

/// Why an [`HstsConfig`] cannot be served.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HstsConfigError {
    /// `hsts.preload` without `hsts.include_subdomains`.
    PreloadWithoutSubdomains,
    /// `hsts.preload` with a `hsts.max_age_secs` below one year.
    PreloadMaxAgeTooShort,
}

impl fmt::Display for HstsConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PreloadWithoutSubdomains => f.write_str(
                "hsts.preload requires hsts.include_subdomains: the preload list refuses a host \
                 whose policy does not cover its subdomains",
            ),
            Self::PreloadMaxAgeTooShort => write!(
                f,
                "hsts.preload requires hsts.max_age_secs >= {PRELOAD_MIN_MAX_AGE_SECS} (one year)"
            ),
        }
    }
}

impl std::error::Error for HstsConfigError {}

#[cfg(test)]
mod tests {
    use super::{HstsConfig, HstsConfigError};

    #[test]
    fn the_default_is_the_header_the_site_has_always_sent() {
        let config = HstsConfig::default();
        assert_eq!(
            config.header_value(),
            "max-age=31536000; includeSubDomains; preload"
        );
        assert_eq!(config.validate(), Ok(()));
    }

    #[test]
    fn flags_that_are_off_are_left_out() {
        let config = HstsConfig {
            max_age_secs: 600,
            include_subdomains: false,
            preload: false,
        };
        assert_eq!(config.header_value(), "max-age=600");
        assert_eq!(config.validate(), Ok(()));
    }

    #[test]
    fn preload_is_held_to_the_lists_requirements() {
        let without_subdomains = HstsConfig {
            include_subdomains: false,
            ..HstsConfig::default()
        };
        assert_eq!(
            without_subdomains.validate(),
            Err(HstsConfigError::PreloadWithoutSubdomains)
        );

        let short = HstsConfig {
            max_age_secs: 86_400,
            ..HstsConfig::default()
        };
        assert_eq!(
            short.validate(),
            Err(HstsConfigError::PreloadMaxAgeTooShort)
        );
    }
}
