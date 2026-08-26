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
/// Ordered by severity, so a threshold names the *least* severe record it accepts: `warn` means
/// `error` and `warn`.
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
/// Every key is inert while [`enabled`](Self::enabled) is off, which is the default and what an
/// unmentioned `[sentry]` block produces. Switched on without a usable
/// [`dsn`](Self::dsn), the boot is refused rather than started with a reporter that reports
/// nowhere — see [`validate`](Self::validate).
// No `deny_unknown_fields`, for the reason given on `AssetsConfig`: the `_FILE` indirection keys
// share this namespace.
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
    /// Initialise the Sentry client. `false` installs no client, no panic hook, no `tracing`
    /// layer and no HTTP middleware, so every other key here is inert and nothing leaves the
    /// process.
    ///
    /// It also decides who owns the log subscriber. Off — the default — the Dioxus toolchain
    /// installs its own, which is the behaviour this server has always had. On, the server
    /// installs one first (still reading `RUST_LOG`, still formatting the same way) because a
    /// Sentry layer has to be a layer *of* the subscriber, and the framework's is not
    /// extensible after the fact.
    #[serde(default)]
    pub enabled: bool,

    /// Ingest URL, `https://<key>@<host>/<project>`.
    ///
    /// The embedded key is a bearer credential for the project's ingest endpoint, so it is held
    /// as a `SecretString` — this block is nested in a `ServerConfig` that is printed with `?` on
    /// the refusal path. Supply it as `PORTFOLIO_SENTRY__DSN_FILE=/run/secrets/sentry_dsn`, or as
    /// `sentry__dsn` inside `$PORTFOLIO_SECRETS_DIR`, so it never enters the process
    /// environment — where `/proc/<pid>/environ`, a crash dump and `docker inspect` all carry
    /// it, and every child process inherits it.
    ///
    /// Absent while `sentry.enabled` is set is a boot failure, not a silent no-op.
    // Not rustdoc: `skip_serializing` is here for the same reason it is on `github.token` —
    // `SecretString` has no `Serialize`, and the schema generator serialises a default config to
    // read the `Default` column out of it. The key still appears in the table, with `unset` for
    // a default it never had.
    #[cfg_attr(feature = "config-schema", config(secret))]
    #[serde(default, skip_serializing)]
    pub dsn: Option<SecretString>,

    /// Environment tag on every event.
    ///
    /// Unset resolves to `production` for a release build and `development` otherwise, which is
    /// the only distinction this workspace has: it ships one image, and the thing that separates
    /// a developer's `dx serve` from the deployed container is the profile it was built with.
    #[cfg_attr(
        feature = "config-schema",
        config(note = "`production` in a release build, `development` otherwise")
    )]
    #[serde(default)]
    pub environment: Option<String>,

    /// Release tag on every event.
    ///
    /// Unset resolves to `portfolio@<version>`, the workspace version the binary was built from
    /// — which is what makes a regression attributable to a deploy, since every deploy of this
    /// site is a new image cut from a new version.
    #[cfg_attr(
        feature = "config-schema",
        config(note = "`portfolio@` and the built version")
    )]
    #[serde(default)]
    pub release: Option<String>,

    /// Host tag on every event.
    ///
    /// Left unset, Sentry reports none. The hostname of a replica is infrastructure detail that
    /// `sentry.send_default_pii` would otherwise be the gate for, and this site runs one image
    /// behind a tunnel, so it identifies nothing an event does not already say.
    #[serde(default)]
    pub server_name: Option<String>,

    /// Fraction of captured events actually sent, `0.0`–`1.0`.
    ///
    /// A blunt volume cap: it drops whole issues rather than repetitions of one, so a rare bug
    /// is exactly as likely to be dropped as a noisy one. Leave it at `1.0` unless a quota
    /// forces otherwise.
    #[serde(default = "SentryConfig::default_sample_rate")]
    pub sample_rate: f32,

    /// Fraction of request traces recorded, `0.0`–`1.0`.
    ///
    /// `0.0` — the default — records none, which is what keeps switching Sentry on for crash
    /// reporting from also switching performance data on. `0.05`–`0.2` is an ordinary
    /// production figure.
    ///
    /// This server starts every trace it has: it is the edge, and nothing upstream of it hands
    /// it a sampled trace to continue. That is the difference from a service tier, where the
    /// rate of whoever *starts* the trace decides whether it exists at all.
    #[serde(default)]
    pub traces_sample_rate: f32,

    /// Least severe `tracing` level reported as a Sentry **issue**.
    #[cfg_attr(feature = "config-schema", config(values))]
    #[serde(default)]
    pub capture_level: SentryLevel,

    /// Least severe `tracing` level kept as a **breadcrumb**, the trail attached to the next
    /// issue.
    ///
    /// Records at or above `sentry.capture_level` become issues instead, so the two thresholds
    /// partition the stream rather than overlapping on it.
    #[cfg_attr(feature = "config-schema", config(values))]
    #[serde(default = "SentryConfig::default_breadcrumb_level")]
    pub breadcrumb_level: SentryLevel,

    /// How many breadcrumbs one event carries.
    #[serde(default = "SentryConfig::default_max_breadcrumbs")]
    pub max_breadcrumbs: usize,

    /// Attach a stack trace to events that carry none of their own.
    #[serde(default = "SentryConfig::default_attach_stacktraces")]
    pub attach_stacktraces: bool,

    /// Send personally identifying data with every event: the client IP, the full request header
    /// set (`Cookie` included) and the resolved user.
    ///
    /// **Off, and worth leaving off.** A reader's IP address and cookies are exactly what a
    /// crash report does not need in order to be actionable, and Sentry is a third party for the
    /// purposes of the privacy page this site publishes — a page that currently says no
    /// third-party service receives visitor data, which is a statement this key is the one way
    /// to falsify. On, it also widens what the HTTP middleware records, because `sentry-tower`
    /// reads this same flag to decide whether to redact sensitive request headers.
    #[serde(default)]
    pub send_default_pii: bool,

    /// Record one Sentry transaction per request, named by the *matched route* rather than by
    /// the URI — so `/api/repos/{name}` does not become one transaction name per repository.
    ///
    /// Whether a started transaction is *kept* is `sentry.traces_sample_rate`'s decision; this is
    /// the switch for taking the middleware out entirely.
    #[serde(default = "SentryConfig::default_http_transactions")]
    pub http_transactions: bool,

    /// Copy `tracing` span fields onto the Sentry span as attributes.
    ///
    /// Off. Span fields here routinely carry request paths and the negotiated locale, and a
    /// transaction is stored under a longer retention than a log line.
    #[serde(default)]
    pub span_attributes: bool,

    /// Print the SDK's own diagnostics to stderr. For proving a DSN works, not for running.
    #[serde(default)]
    pub debug: bool,
}

impl SentryConfig {
    /// Send everything that is captured; see the field.
    const fn default_sample_rate() -> f32 {
        1.0
    }

    /// Breadcrumbs down to `info`; see the field.
    const fn default_breadcrumb_level() -> SentryLevel {
        SentryLevel::Info
    }

    /// The SDK's own default, restated so the generated table can print it.
    const fn default_max_breadcrumbs() -> usize {
        100
    }

    /// Stack traces are the point; see the field.
    const fn default_attach_stacktraces() -> bool {
        true
    }

    /// Route-named transactions are the point; see the field.
    const fn default_http_transactions() -> bool {
        true
    }

    /// The ingest URL, or `None` when unset or blank.
    ///
    /// Blank has to read as unset rather than as a DSN that fails to parse: `PORTFOLIO_SENTRY__DSN=`
    /// is how a container platform spells a declared-but-unset variable, and how an unfilled chart
    /// value arrives. The two produce very different messages and only one sends the operator to
    /// the right place.
    ///
    /// This is the single point where the DSN is exposed, and the only caller is the line in
    /// `apps/web/src/server/telemetry.rs` that hands it to `sentry::init`.
    #[must_use]
    pub fn dsn(&self) -> Option<&str> {
        self.dsn
            .as_ref()
            .map(|dsn| dsn.expose_secret().trim())
            .filter(|dsn| !dsn.is_empty())
    }

    /// Whether a client should be installed at all.
    ///
    /// One accessor rather than reading the field, because "on" means "on *and* pointing
    /// somewhere": [`validate`](Self::validate) has already refused the boot for the other
    /// combination, so these two are the same question by the time anything asks it.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.enabled && self.dsn().is_some()
    }

    /// Reject a `[sentry]` block that would install a client reporting nowhere, or one the SDK
    /// would panic on.
    ///
    /// Called at boot beside [`CspConfig::validate`](crate::CspConfig::validate), before the
    /// listener exists, so the container fails to start rather than serving a site whose errors
    /// go into a void the dashboard makes look empty.
    ///
    /// What is *not* checked here is whether the DSN parses: that is
    /// `sentry::types::Dsn`'s answer, and this crate does not link the SDK. The server asks it
    /// immediately afterwards and refuses the same way.
    ///
    /// # Errors
    ///
    /// [`SentryConfigError::MissingDsn`] when [`enabled`](Self::enabled) is set with no usable
    /// [`dsn`](Self::dsn), and [`SentryConfigError::RateOutOfRange`] when a sample rate falls
    /// outside `0.0..=1.0` — which `ClientOptions` would otherwise accept and then never sample
    /// as asked.
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

/// Whether `rate` is a fraction the SDK can sample by.
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

/// A `[sentry]` block that cannot be started with.
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum SentryConfigError {
    /// `sentry.enabled` is set while `sentry.dsn` is unset or blank.
    MissingDsn,
    /// A sample rate is outside `0.0..=1.0`.
    RateOutOfRange {
        /// The key that carries it, as an operator spells it.
        key: &'static str,
        /// What it was set to. Not a secret, and naming it is what makes a stray `%` or a
        /// percentage typed as `25` obvious from the log line alone.
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
    /// — so an absent block has to materialise rather than fail the boot of a deployment that
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
