//! Optional Sentry error reporting and performance tracing.
//!
//! Off unless `sentry.enabled` is set, and then only with a DSN — the combination without one is
//! refused by [`SentryConfig::validate`] before this module is reached, because a client that
//! reports nowhere produces an empty dashboard that looks exactly like a site with no errors.
//!
//! Three sinks, all fed from the one client [`init`] installs:
//!
//! - **`tracing`** — [`tracing_layer`] turns records into issues and breadcrumbs, under the two
//!   thresholds in [`SentryConfig`].
//! - **panics** — the SDK's own hook, added by `sentry::init`. A panic in a handler unwinds that
//!   request's task and leaves the process serving every other request, so without the hook it is
//!   invisible: no 500 is logged, and the reader sees a dropped connection.
//! - **HTTP** — [`http_layers`], mounted by [`super::router`].
//!
//! # This module owns the log subscriber, but only while Sentry is on
//!
//! A Sentry layer has to be a layer *of* the subscriber, and `dioxus::serve` installs one through
//! `dioxus_logger::initialize_default`, which cannot be extended afterwards. It can, however, be
//! declined: it returns early when `tracing::dispatcher::has_been_set()`, so installing ours
//! first — before `dioxus::serve` — is the whole of the handover. [`init`] therefore builds a
//! subscriber that keeps the framework's contract (`RUST_LOG`, the same default verbosity, the
//! same `hyper_util` mute) and adds the Sentry layer to it.
//!
//! With Sentry off, nothing here is constructed and the framework's subscriber stands exactly as
//! before. That asymmetry is deliberate: the default path must not change behaviour to make an
//! opt-in feature possible.
//!
//! One visible difference on the opted-in path, and only in development: `dioxus_logger` drops
//! timestamps and targets when it detects that it is running under `dx`, and this does not,
//! because the detection lives in `dioxus-cli-config` — a crate the `server` feature does not
//! pull in, and not one worth pulling in for the width of a log line.

use core::fmt;

use portfolio_config::{SentryConfig, SentryLevel};
use sentry::integrations::tracing::{EventFilter, SentryLayer, default_span_filter};
use tracing::Level;
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::util::SubscriberInitExt as _;
use tracing_subscriber::{EnvFilter, Registry};

/// What the default release tag names the project, ahead of the version.
///
/// The image, the contract and the GitHub repository all spell it this way, so an event's release
/// is a string an operator can match against a tag without translating it.
const RELEASE_PREFIX: &str = "portfolio";

/// The module paths a stack trace should treat as application code.
///
/// The crate names as the linker spells them, so `apps/web`'s own frames and the two workspace
/// libraries are in-app and everything under `dioxus`, `axum` and `hyper` is not. That is what
/// makes a trace open on the handler rather than on a framework internal ten frames above it.
const IN_APP: &[&str] = &["web", "portfolio_config", "portfolio_data"];

/// Keeps the Sentry client alive.
///
/// Bind it (`let _telemetry = …`) for as long as the process should report; `let _ = …` drops it
/// immediately and closes the client before the server has served anything.
///
/// # The drop is not a shutdown flush here
///
/// `sentry::ClientInitGuard::drop` normally flushes what the transport has queued. In this binary
/// it never runs: `dioxus::serve` diverges (`-> !`), and its release path awaits `axum::serve`
/// with no shutdown signal, so a container stopping takes the default `SIGTERM` disposition and
/// the process is gone before any destructor. That is why `shutdown_timeout_secs` is absent from
/// the configuration surface — it would be a key describing a code path this server does not
/// have. What actually gets events out is the SDK's background transport, which sends
/// continuously rather than at exit.
#[must_use = "dropping the guard closes the Sentry client and stops reporting"]
pub struct TelemetryGuard(Option<sentry::ClientInitGuard>);

// Hand-written because `ClientInitGuard` has no `Debug`, and because the only fact worth printing
// is the one an operator asks for: is anything reporting at all.
impl fmt::Debug for TelemetryGuard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("TelemetryGuard")
            .field(&self.0.is_some())
            .finish()
    }
}

/// Install the Sentry client and the subscriber that feeds it, or leave both alone.
///
/// Called before `dioxus::serve`, which is the only moment the handover described in the module
/// documentation is available. Returns a guard the caller has to hold.
///
/// # Errors
///
/// [`TelemetryError::Dsn`] when `sentry.dsn` is not a Sentry DSN — checked here rather than in
/// `portfolio-config`, which does not link the SDK — and [`TelemetryError::Subscriber`] when a
/// `tracing` subscriber is already installed, which means the Sentry layer would silently report
/// nothing. Both are start-up failures the caller refuses the boot on.
pub fn init(config: &SentryConfig) -> Result<TelemetryGuard, TelemetryError> {
    // Off, or on with no DSN — the second already refused by `SentryConfig::validate`, and
    // treated here as off rather than re-reported, so this function has exactly one failure mode
    // per error variant.
    let Some(dsn) = config.enabled.then(|| config.dsn()).flatten() else {
        return Ok(TelemetryGuard(None));
    };

    // Parsed here rather than handed to `ClientOptions::dsn`, which panics on a malformed value.
    let dsn = dsn
        .parse::<sentry::types::Dsn>()
        .map_err(TelemetryError::Dsn)?;

    let mut options = sentry::ClientOptions::new()
        .debug(config.debug)
        .sample_rate(config.sample_rate)
        .traces_sample_rate(config.traces_sample_rate)
        .max_breadcrumbs(config.max_breadcrumbs)
        .attach_stacktrace(config.attach_stacktraces)
        .send_default_pii(config.send_default_pii)
        .environment(environment(config))
        .release(release(config))
        .in_app_include(IN_APP.to_vec());
    options.dsn = Some(dsn);
    if let Some(server_name) = config.server_name.clone() {
        options = options.server_name(server_name);
    }

    // Every field `apply_defaults` would otherwise fill from `SENTRY_DSN`, `SENTRY_RELEASE` or
    // `SENTRY_ENVIRONMENT` is set above, and that is the point: those variables are a second
    // configuration channel that bypasses the layered loader and the shadow-key rejection that
    // makes a rotated secret trustworthy, and an already-set field is one they cannot reach.
    //
    // Before the subscriber: the layer below reports onto this client, and the panic hook should
    // be in place for anything the subscriber build itself does.
    let guard = sentry::init(options);

    Registry::default()
        .with(env_filter())
        .with(tracing_layer(config))
        .with(tracing_subscriber::fmt::layer().with_target(true))
        .try_init()
        .map_err(|err| TelemetryError::Subscriber(err.to_string()))?;

    // After `try_init`, not beside `sentry::init`: a record emitted before the subscriber exists
    // goes nowhere, and "is Sentry actually on in this container" is the first question an
    // operator asks.
    tracing::info!(
        traces_sample_rate = config.traces_sample_rate,
        send_default_pii = config.send_default_pii,
        "Sentry reporting enabled"
    );

    Ok(TelemetryGuard(Some(guard)))
}

/// The per-request hub and the request-metadata layer, or `None` when Sentry is off.
///
/// The hub layer is not optional decoration. Without a hub per request, breadcrumbs from
/// concurrently served requests all land on the main hub, and every issue arrives with a trail
/// belonging to whoever else happened to be in flight.
///
/// Built after [`init`] rather than beside it, because `SentryHttpLayer::new` reads
/// `send_default_pii` off the bound client to decide whether to redact sensitive request headers
/// — with no client bound it would decide against a default rather than against the
/// configuration.
pub fn http_layers(
    config: &SentryConfig,
) -> Option<(
    sentry::integrations::tower::NewSentryLayer<axum::extract::Request>,
    sentry::integrations::tower::SentryHttpLayer,
)> {
    if !config.is_active() {
        return None;
    }

    let http = sentry::integrations::tower::SentryHttpLayer::new();
    let http = if config.http_transactions {
        http.enable_transaction()
    } else {
        http
    };
    Some((
        sentry::integrations::tower::NewSentryLayer::new_from_top(),
        http,
    ))
}

/// The `tracing` layer feeding the client.
///
/// Sits under the subscriber's [`EnvFilter`], which is the one surprise worth knowing: a record
/// `RUST_LOG` drops never reaches this layer, so tightening the log filter to `warn` also removes
/// every `info` breadcrumb.
fn tracing_layer<S>(config: &SentryConfig) -> SentryLayer<S>
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
{
    let capture = config.capture_level;
    let breadcrumb = config.breadcrumb_level;

    let layer = sentry::integrations::tracing::layer()
        .event_filter(move |metadata| {
            let level = *metadata.level();
            if accepts(capture, level) {
                EventFilter::Event
            } else if accepts(breadcrumb, level) {
                EventFilter::Breadcrumb
            } else {
                EventFilter::Ignore
            }
        })
        // Not additionally gated on `traces_sample_rate`: whether a span is *recorded* is the
        // sampler's decision, and gating span creation here would also remove the spans that
        // give an issue its context, not only the ones that would have become a transaction.
        .span_filter(default_span_filter);

    if config.span_attributes {
        layer.enable_span_attributes()
    } else {
        layer
    }
}

/// The verbosity filter, matching what `dioxus_logger::initialize_default` would have installed.
///
/// Same contract, so switching Sentry on does not also change what is logged: `RUST_LOG` wins,
/// the default is `debug` for a development build and `info` otherwise, and `hyper_util` is muted
/// because it leaves `debug!` calls on the connection accept path that drown everything else.
fn env_filter() -> EnvFilter {
    let default = if cfg!(debug_assertions) {
        Level::DEBUG
    } else {
        Level::INFO
    };
    let filter = EnvFilter::builder()
        .with_default_directive(default.into())
        .from_env_lossy();

    // A literal that cannot fail to parse, matched rather than unwrapped: a panic here would
    // take down a boot over a log directive, and the honest fallback is the filter without it.
    match "hyper_util=warn".parse() {
        Ok(directive) => filter.add_directive(directive),
        Err(_) => filter,
    }
}

/// The environment tag, from the configuration or from the build profile.
///
/// The profile is the only distinction this workspace has: it ships one image, and what separates
/// a developer's `dx serve` from the deployed container is what it was compiled as.
fn environment(config: &SentryConfig) -> String {
    config.environment.clone().unwrap_or_else(|| {
        if cfg!(debug_assertions) {
            "development".to_owned()
        } else {
            "production".to_owned()
        }
    })
}

/// The release tag, from the configuration or from the version this binary was built at.
///
/// `CARGO_PKG_VERSION` rather than a `v`-prefixed tag: every deploy of this site is a new image
/// cut from a new workspace version, so the bare version is what makes a regression attributable
/// to one.
fn release(config: &SentryConfig) -> String {
    config
        .release
        .clone()
        .unwrap_or_else(|| format!("{RELEASE_PREFIX}@{}", env!("CARGO_PKG_VERSION")))
}

/// Whether a record at `level` is at least as severe as `threshold`.
///
/// [`Level`] orders `ERROR` lowest, so "at least as severe" is `<=`.
fn accepts(threshold: SentryLevel, level: Level) -> bool {
    let threshold = match threshold {
        SentryLevel::Off => return false,
        SentryLevel::Error => Level::ERROR,
        SentryLevel::Warn => Level::WARN,
        SentryLevel::Info => Level::INFO,
        SentryLevel::Debug => Level::DEBUG,
        SentryLevel::Trace => Level::TRACE,
    };
    level <= threshold
}

/// A `[sentry]` block that loaded but cannot be installed.
///
/// Separate from [`portfolio_config::SentryConfigError`] on purpose: that one is everything the
/// schema can decide on its own, and this is the two answers only the SDK and the process can
/// give. Both end the boot through the same refusal, so an operator sees one behaviour.
#[derive(Debug)]
#[non_exhaustive]
pub enum TelemetryError {
    /// `sentry.dsn` is not a Sentry DSN.
    Dsn(sentry::types::ParseDsnError),
    /// A `tracing` subscriber was already installed, so the Sentry layer has nothing to sit in.
    Subscriber(String),
}

impl fmt::Display for TelemetryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // The DSN is deliberately not quoted: it is a credential, and this message reaches a
            // log stream that is shipped and retained.
            Self::Dsn(err) => write!(
                f,
                "sentry.dsn is not a valid Sentry DSN ({err}); the project settings page spells \
                 it https://<key>@<host>/<project>"
            ),
            Self::Subscriber(err) => write!(
                f,
                "a tracing subscriber was already installed ({err}), so the Sentry layer could \
                 not be added and nothing would be reported. This server installs the subscriber \
                 itself while sentry.enabled is set; something ran before it"
            ),
        }
    }
}

impl core::error::Error for TelemetryError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Dsn(err) => Some(err),
            Self::Subscriber(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{accepts, environment, http_layers, init, release};
    use portfolio_config::{SentryConfig, SentryLevel};
    use secrecy::SecretString;
    use tracing::Level;

    /// [`Level`] sorts `ERROR` *below* `TRACE`, so a severity threshold reads as `<=` and not
    /// `>=`. Inverting it turns `capture_level = "error"` into "capture everything", which is a
    /// bill rather than a compile error.
    #[test]
    fn a_threshold_accepts_only_levels_at_least_as_severe() {
        assert!(accepts(SentryLevel::Error, Level::ERROR));
        assert!(!accepts(SentryLevel::Error, Level::WARN));
        assert!(!accepts(SentryLevel::Error, Level::TRACE));

        assert!(accepts(SentryLevel::Info, Level::ERROR));
        assert!(accepts(SentryLevel::Info, Level::WARN));
        assert!(accepts(SentryLevel::Info, Level::INFO));
        assert!(!accepts(SentryLevel::Info, Level::DEBUG));

        for level in [
            Level::ERROR,
            Level::WARN,
            Level::INFO,
            Level::DEBUG,
            Level::TRACE,
        ] {
            assert!(!accepts(SentryLevel::Off, level));
            assert!(accepts(SentryLevel::Trace, level));
        }
    }

    /// The default path installs nothing at all — not a client with an empty DSN, which still
    /// starts a transport thread and still queues events — and leaves the subscriber to the
    /// framework, which is what keeps this feature from changing a deployment that has not asked
    /// for it.
    #[test]
    fn the_default_configuration_installs_nothing() {
        let config = SentryConfig::default();
        let guard = init(&config).expect("an off block cannot fail to install nothing");
        assert_eq!(format!("{guard:?}"), "TelemetryGuard(false)");
        assert!(http_layers(&config).is_none());
        assert!(
            !tracing::dispatcher::has_been_set(),
            "the off path must leave the subscriber to `dioxus::serve`; taking it here would \
             change what a deployment that never asked for Sentry logs"
        );
    }

    /// `enabled` without a DSN is refused by `SentryConfig::validate` at boot; reaching this
    /// module anyway must still install nothing rather than a client pointing at nowhere.
    #[test]
    fn enabled_without_a_dsn_installs_nothing() {
        let config = SentryConfig {
            enabled: true,
            ..SentryConfig::default()
        };
        let guard = init(&config).expect("no DSN is not this module's failure to report");
        assert_eq!(format!("{guard:?}"), "TelemetryGuard(false)");
        assert!(http_layers(&config).is_none());
    }

    /// A DSN that is not one has to fail the boot rather than panic inside `ClientOptions`, which
    /// is what `ClientOptions::dsn` does with the same value.
    #[test]
    fn a_malformed_dsn_is_an_error_rather_than_a_panic() {
        let config = SentryConfig {
            enabled: true,
            dsn: Some(SecretString::from("not-a-dsn")),
            ..SentryConfig::default()
        };
        let err = init(&config).expect_err("a malformed DSN must not install a client");
        let message = err.to_string();
        assert!(message.contains("sentry.dsn"), "{message}");
        // The credential must not reach a log line, even the malformed one — an operator who
        // pasted the wrong secret into the right key would otherwise publish it.
        assert!(!message.contains("not-a-dsn"), "{message}");
    }

    /// Both tags fall back to something that identifies the build, because an untagged event is
    /// an event nobody can attribute to a deploy.
    #[test]
    fn the_release_and_environment_tags_have_defaults() {
        let config = SentryConfig::default();
        assert_eq!(
            release(&config),
            format!("portfolio@{}", env!("CARGO_PKG_VERSION"))
        );
        assert!(matches!(
            environment(&config).as_str(),
            "development" | "production"
        ));

        let overridden = SentryConfig {
            environment: Some("staging".to_owned()),
            release: Some("portfolio@custom".to_owned()),
            ..SentryConfig::default()
        };
        assert_eq!(environment(&overridden), "staging");
        assert_eq!(release(&overridden), "portfolio@custom");
    }
}
