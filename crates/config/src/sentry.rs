//! Sentry error reporting and performance tracing for the SSR server.
//!
//! The block is here and the client is in `apps/web/src/server/telemetry.rs`, for the same
//! reason every other block is split that way: this crate owns the schema, so the generated
//! configuration reference can describe the keys without linking the SDK that reads them.
//!
//! Off by default, and it has to stay that way. A DSN is an egress destination for whatever a
//! log line happens to carry, and this site publishes a privacy page saying it measures nothing;
//! switching it on is an operator's decision, made once per deployment, with
//! [`send_default_pii`](SentryConfig::send_default_pii) still off underneath it.

use core::fmt;

use secrecy::{ExposeSecret as _, SecretString};
use serde::Deserialize;

/// How much of the `tracing` stream one Sentry sink takes.
///
/// A threshold names the least severe record it accepts: `warn` means `error` and `warn`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[cfg_attr(
    feature = "config-schema",
    derive(serde::Serialize, terrace_config::schema::Describe)
)]
#[serde(rename_all = "lowercase")]
pub enum SentryLevel {
    /// Take nothing.
    Off,
    /// `error` only.
    #[default]
    Error,
    /// `error` and `warn`.
    Warn,
    /// Down to `info`.
    Info,
    /// Down to `debug`.
    Debug,
    /// Everything.
    Trace,
}

/// Sentry error reporting and performance tracing.
///
/// Every key is inert while [`enabled`](Self::enabled) is off, which is the default. Switched on
/// without a usable [`dsn`](Self::dsn), the boot is refused; see [`validate`](Self::validate).
// No `deny_unknown_fields`, for the reasons given on `AssetsConfig`: closing a `#[config(nested)]`
// block narrows what the loader accepts and publishes nothing to the contract.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(
    feature = "config-schema",
    derive(serde::Serialize, terrace_config::schema::Describe)
)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "independent operator toggles, one per PORTFOLIO_SENTRY__* variable"
)]
pub struct SentryConfig {
    /// Whether the server initializes the Sentry client. Off installs no client, panic hook,
    /// `tracing` layer or HTTP middleware, and nothing leaves the process.
    ///
    /// It also decides who owns the log subscriber. Off, the Dioxus toolchain installs its own.
    /// On, the server installs one first, still reading `RUST_LOG` and formatting the same way,
    /// because a Sentry layer has to be a layer of the subscriber.
    #[serde(default)]
    pub enabled: bool,

    /// The ingest URL, `https://<key>@<host>/<project>`.
    ///
    /// The embedded key is a bearer credential for the project's ingest endpoint. Supply it as
    /// `PORTFOLIO_SENTRY__DSN_FILE=/run/secrets/sentry_dsn`, or as `sentry__dsn` inside
    /// `$PORTFOLIO_SECRETS_DIR`, so it never enters the process environment, where
    /// `/proc/<pid>/environ`, a crash dump and `docker inspect` all carry it and every child
    /// process inherits it.
    ///
    /// Absent while `sentry.enabled` is set, it fails the boot.
    // Not rustdoc: `skip_serializing` is here for the same reason it is on `github.token` —
    // `SecretString` has no `Serialize`, and the schema generator serializes a default config to
    // read the `Default` column out of it. The key still appears in the table, with `unset` for
    // a default it never had.
    #[cfg_attr(feature = "config-schema", config(secret))]
    #[serde(default, skip_serializing)]
    pub dsn: Option<SecretString>,

    /// The environment tag on every event.
    ///
    /// Unset resolves to `production` for a release build and `development` otherwise. The
    /// workspace ships one image, so the build profile is what separates a developer's
    /// `dx serve` from the deployed container.
    #[cfg_attr(
        feature = "config-schema",
        config(note = "`production` in a release build, `development` otherwise")
    )]
    #[serde(default)]
    pub environment: Option<String>,

    /// The release tag on every event.
    ///
    /// Unset resolves to `portfolio@<version>`, the workspace version the binary was built from.
    /// Every deploy is a new image cut from a new version, so the tag attributes a regression to
    /// a deploy.
    #[cfg_attr(
        feature = "config-schema",
        config(note = "`portfolio@` and the built version")
    )]
    #[serde(default)]
    pub release: Option<String>,

    /// The host tag on every event. Unset reports none.
    ///
    /// The hostname of a replica is infrastructure detail, and this site runs one image behind a
    /// tunnel, so it identifies nothing an event does not already say.
    #[serde(default)]
    pub server_name: Option<String>,

    /// The fraction of captured events sent, from `0.0` to `1.0`.
    ///
    /// It is a blunt volume cap: a rare bug is as likely to be dropped as a noisy one. Leave it
    /// at `1.0` unless a quota forces otherwise.
    // The bound is [`check_rate`]'s, published rather than restated: `validate` already refuses a
    // boot outside `0.0..=1.0`, so a schema saying so rejects nothing the loader would have
    // accepted. Float literals because the field is an `f32` — `min = 0` would publish the
    // integer `0`, which a consumer walking the keywords reads as a different number.
    #[cfg_attr(feature = "config-schema", config(range(min = 0.0, max = 1.0)))]
    #[serde(default = "SentryConfig::default_sample_rate")]
    pub sample_rate: f32,

    /// The fraction of request traces recorded, from `0.0` to `1.0`.
    ///
    /// The default, `0.0`, records none, so switching Sentry on for crash reporting does not
    /// also switch performance data on. `0.05` to `0.2` is an ordinary production figure. This
    /// server is the edge and starts every trace it records, so this rate alone decides whether
    /// a trace exists.
    // Same bound and same source as `sample_rate`: both go through [`check_rate`].
    #[cfg_attr(feature = "config-schema", config(range(min = 0.0, max = 1.0)))]
    #[serde(default)]
    pub traces_sample_rate: f32,

    /// The least severe `tracing` level reported as a Sentry issue.
    #[cfg_attr(feature = "config-schema", config(values))]
    #[serde(default)]
    pub capture_level: SentryLevel,

    /// The least severe `tracing` level kept as a breadcrumb, the trail attached to the next
    /// issue.
    ///
    /// Records at or above `sentry.capture_level` become issues instead, so the two thresholds
    /// partition the stream.
    #[cfg_attr(feature = "config-schema", config(values))]
    #[serde(default = "SentryConfig::default_breadcrumb_level")]
    pub breadcrumb_level: SentryLevel,

    /// The most breadcrumbs one event carries.
    #[serde(default = "SentryConfig::default_max_breadcrumbs")]
    pub max_breadcrumbs: usize,

    /// Whether events that carry no stack trace of their own get one attached.
    #[serde(default = "SentryConfig::default_attach_stacktraces")]
    pub attach_stacktraces: bool,

    /// Whether every event carries personally identifying data: the client IP, the full request
    /// header set including `Cookie`, and the resolved user.
    ///
    /// **Off, and worth leaving off.** A crash report is actionable without a reader's IP address
    /// and cookies, and every event sent is personal data handed to a third party, which the
    /// published privacy notice would then have to name. On, it also stops the HTTP middleware
    /// from redacting sensitive request headers.
    #[serde(default)]
    pub send_default_pii: bool,

    /// Whether the server records one Sentry transaction per request, named by the matched route.
    ///
    /// Naming by route keeps `/api/v1/legal/{slug}` one transaction name for every document.
    /// Whether a started transaction is kept is `sentry.traces_sample_rate`'s decision. Off starts
    /// no transaction, but the HTTP middleware stays installed and still attaches the request to
    /// events.
    #[serde(default = "SentryConfig::default_http_transactions")]
    pub http_transactions: bool,

    /// Whether `tracing` span fields are copied onto the Sentry span as attributes.
    ///
    /// Off, because span fields here carry request paths and the negotiated locale, and a
    /// transaction is stored under a longer retention than a log line.
    #[serde(default)]
    pub span_attributes: bool,

    /// Whether the SDK prints its own diagnostics to stderr, for proving a DSN works.
    #[serde(default)]
    pub debug: bool,
}

impl SentryConfig {
    /// Returns the default of [`Self::sample_rate`], everything.
    const fn default_sample_rate() -> f32 {
        1.0
    }

    /// Returns the default of [`Self::breadcrumb_level`], `info`.
    const fn default_breadcrumb_level() -> SentryLevel {
        SentryLevel::Info
    }

    /// Returns the default of [`Self::max_breadcrumbs`], the SDK's own, restated for the
    /// generated table.
    const fn default_max_breadcrumbs() -> usize {
        100
    }

    /// Returns the default of [`Self::attach_stacktraces`], on.
    const fn default_attach_stacktraces() -> bool {
        true
    }

    /// Returns the default of [`Self::http_transactions`], on.
    const fn default_http_transactions() -> bool {
        true
    }

    /// Returns the ingest URL, or `None` when unset or blank.
    ///
    /// This is the single point where the DSN is exposed.
    // Blank reads as unset, not as a DSN that fails to parse: `PORTFOLIO_SENTRY__DSN=` is how a
    // container platform spells a declared-but-unset variable and how an unfilled chart value
    // arrives, and only the "missing" message sends the operator to the right place.
    #[must_use]
    pub fn dsn(&self) -> Option<&str> {
        self.dsn
            .as_ref()
            .map(|dsn| dsn.expose_secret().trim())
            .filter(|dsn| !dsn.is_empty())
    }

    /// Reports whether a client should be installed: [`enabled`](Self::enabled) with a usable
    /// [`dsn`](Self::dsn).
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.enabled && self.dsn().is_some()
    }

    /// Checks that an enabled block would install a client that reports somewhere and that the
    /// SDK accepts.
    ///
    /// The server calls it at boot, before the listener exists. It does not check that the DSN
    /// parses, which needs the SDK this crate does not link; the server checks that next.
    ///
    /// # Errors
    ///
    /// Returns [`SentryConfigError::MissingDsn`] when [`enabled`](Self::enabled) is set with no
    /// usable [`dsn`](Self::dsn), and [`SentryConfigError::RateOutOfRange`] when a sample rate
    /// falls outside `0.0..=1.0`, which the SDK would accept and then never sample as asked.
    ///
    /// ```
    /// # use portfolio_config::{SentryConfig, SentryConfigError};
    /// let enabled = SentryConfig { enabled: true, ..SentryConfig::default() };
    /// assert_eq!(enabled.validate(), Err(SentryConfigError::MissingDsn));
    /// assert_eq!(SentryConfig::default().validate(), Ok(()));
    /// ```
    pub fn validate(&self) -> Result<(), SentryConfigError> {
        if !self.enabled {
            return Ok(());
        }
        if self.dsn().is_none() {
            return Err(SentryConfigError::MissingDsn);
        }
        check_rate("sentry.sample_rate", self.sample_rate)?;
        check_rate("sentry.traces_sample_rate", self.traces_sample_rate)?;
        Ok(())
    }
}

/// Checks that `rate`, the value of `key`, is a fraction the SDK can sample by.
fn check_rate(key: &'static str, rate: f32) -> Result<(), SentryConfigError> {
    if (0.0..=1.0).contains(&rate) {
        Ok(())
    } else {
        Err(SentryConfigError::RateOutOfRange { key, rate })
    }
}

impl Default for SentryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            dsn: None,
            environment: None,
            release: None,
            server_name: None,
            sample_rate: Self::default_sample_rate(),
            traces_sample_rate: 0.0,
            capture_level: SentryLevel::Error,
            breadcrumb_level: Self::default_breadcrumb_level(),
            max_breadcrumbs: Self::default_max_breadcrumbs(),
            attach_stacktraces: Self::default_attach_stacktraces(),
            send_default_pii: false,
            http_transactions: Self::default_http_transactions(),
            span_attributes: false,
            debug: false,
        }
    }
}

/// The reason a `[sentry]` block cannot be started with.
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum SentryConfigError {
    /// `sentry.enabled` is set while `sentry.dsn` is unset or blank.
    MissingDsn,
    /// A sample rate is outside `0.0..=1.0`.
    RateOutOfRange {
        /// The key that carries the rate, as an operator spells it.
        key: &'static str,
        /// The value the key was set to, which makes a percentage typed as `25` obvious from the
        /// log line alone.
        rate: f32,
    },
}

impl fmt::Display for SentryConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingDsn => f.write_str(
                "sentry.enabled is set but sentry.dsn is unset or empty, so a client would be \
                 installed that reports nowhere — and a dashboard with no events in it looks \
                 exactly like a site with no errors. Supply the DSN (sentry.dsn, \
                 PORTFOLIO_SENTRY__DSN_FILE, or sentry__dsn in the secrets directory), or set \
                 sentry.enabled = false",
            ),
            // Deliberately not naming the DSN anywhere in this enum: the message reaches a log
            // stream that is shipped and retained.
            Self::RateOutOfRange { key, rate } => write!(
                f,
                "{key} is a fraction between 0.0 and 1.0, and was set to {rate}. A percentage \
                 belongs here as a decimal — 10% is 0.1",
            ),
        }
    }
}

impl core::error::Error for SentryConfigError {}

#[cfg(test)]
mod tests {
    use super::{SentryConfig, SentryConfigError, SentryLevel};
    use secrecy::SecretString;

    /// A deployment that says nothing about Sentry gets no client and no egress. Every key is
    /// `#[serde(default)]`, twice over — once on `sentry` in the aggregate, once per field here
    /// — so an absent block has to materialize rather than fail the boot of a deployment that
    /// has never heard of this feature.
    #[test]
    fn an_unmentioned_block_is_off() {
        let config = SentryConfig::default();
        assert!(!config.enabled);
        assert!(!config.is_active());
        assert_eq!(config.dsn(), None);
        assert!(!config.send_default_pii);
        assert!((config.traces_sample_rate - 0.0).abs() < f32::EPSILON);
        assert_eq!(config.capture_level, SentryLevel::Error);
        assert_eq!(config.breadcrumb_level, SentryLevel::Info);
        assert_eq!(config.validate(), Ok(()));
    }

    /// The failure this refuses to repeat: a client installed against nothing, whose empty
    /// dashboard is indistinguishable from a healthy site.
    #[test]
    fn enabling_without_a_dsn_is_refused() {
        let config = SentryConfig {
            enabled: true,
            ..SentryConfig::default()
        };
        assert_eq!(config.validate(), Err(SentryConfigError::MissingDsn));
    }

    /// `PORTFOLIO_SENTRY__DSN=` is how a container platform spells "declared but unset", and how
    /// a compose pass-through or an unfilled chart value arrives. It has to land on the missing
    /// message rather than on a parse error about a URL the operator never typed.
    #[test]
    fn a_blank_dsn_reads_as_absent_rather_than_malformed() {
        let config = SentryConfig {
            enabled: true,
            dsn: Some(SecretString::from("   ")),
            ..SentryConfig::default()
        };
        assert_eq!(config.dsn(), None);
        assert_eq!(config.validate(), Err(SentryConfigError::MissingDsn));
    }

    #[test]
    fn a_dsn_is_trimmed_of_the_newline_a_mounted_file_carries() {
        let config = SentryConfig {
            enabled: true,
            dsn: Some(SecretString::from("https://key@sentry.example/42\n")),
            ..SentryConfig::default()
        };
        assert_eq!(config.dsn(), Some("https://key@sentry.example/42"));
        assert!(config.is_active());
        assert_eq!(config.validate(), Ok(()));
    }

    /// A percentage typed as `25` is the mistake this catches, and it is silent otherwise:
    /// `ClientOptions` takes the value and then samples everything.
    #[test]
    fn a_rate_outside_the_unit_interval_is_refused() {
        let enabled = SentryConfig {
            enabled: true,
            dsn: Some(SecretString::from("https://key@sentry.example/42")),
            ..SentryConfig::default()
        };

        for rate in [-0.1_f32, 1.1, 25.0] {
            let config = SentryConfig {
                traces_sample_rate: rate,
                ..enabled.clone()
            };
            assert_eq!(
                config.validate(),
                Err(SentryConfigError::RateOutOfRange {
                    key: "sentry.traces_sample_rate",
                    rate,
                })
            );
        }

        for rate in [0.0_f32, 0.5, 1.0] {
            let config = SentryConfig {
                sample_rate: rate,
                ..enabled.clone()
            };
            assert_eq!(config.validate(), Ok(()));
        }
    }

    /// Nothing is validated while the block is off, so a half-filled `[sentry]` left behind by an
    /// operator who turned it off cannot fail a boot that never installs a client.
    #[test]
    fn a_disabled_block_is_not_checked() {
        let config = SentryConfig {
            enabled: false,
            traces_sample_rate: 42.0,
            ..SentryConfig::default()
        };
        assert_eq!(config.validate(), Ok(()));
        assert!(!config.is_active());
    }

    /// The refusal names the keys an operator has to change, and never the value of the one that
    /// is a credential.
    #[test]
    fn the_refusals_name_the_keys_and_not_the_secret() {
        let missing = SentryConfigError::MissingDsn.to_string();
        assert!(missing.contains("sentry.enabled"));
        assert!(missing.contains("sentry.dsn"));

        let rate = SentryConfigError::RateOutOfRange {
            key: "sentry.sample_rate",
            rate: 25.0,
        }
        .to_string();
        assert!(rate.contains("sentry.sample_rate"));
        assert!(rate.contains("25"));
    }
}
