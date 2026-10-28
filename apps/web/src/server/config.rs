//! The server's configuration, loaded once before anything binds, and the exit that refuses to
//! start without a usable one.

use portfolio_config::ServerConfig;

/// The exit code for a configuration the process cannot start with, `EX_CONFIG` from `sysexits.h`.
///
/// An operator can tell a bad configuration apart from a crash without reading the logs.
const EX_CONFIG: i32 = 78;

/// Ends the process with [`EX_CONFIG`], printing `err` and the configuration provenance report to
/// stderr.
///
/// Neither holds a configuration value, so both are safe in a log that is shipped and retained.
// This runs before any logger is installed, so `tracing` would discard the message. The report
// names the layer that supplied each key, which an operator cannot get at from inside an image
// with no shell.
pub(super) fn refuse(err: &dyn std::fmt::Display) -> ! {
    eprintln!("portfolio: cannot start, the configuration is not usable: {err}");
    eprintln!("{}", portfolio_config::provenance());
    std::process::exit(EX_CONFIG)
}

/// Reads the configuration, or ends the process with [`EX_CONFIG`].
///
/// A configuration is unusable when a value cannot be *loaded* (a missing file, an unparseable
/// number, one key supplied by two layers) or when values that load individually cannot be
/// *served* together, as [`CspConfig::validate`](portfolio_config::CspConfig::validate),
/// [`SentryConfig::validate`](portfolio_config::SentryConfig::validate) and
/// [`HstsConfig::validate`](portfolio_config::HstsConfig::validate) decide.
// Failing here means the container never reports ready, instead of serving every visitor a blank
// page or reporting its errors into a void.
pub(super) fn load() -> ServerConfig {
    let config = match portfolio_config::load::<ServerConfig>() {
        Ok(config) => config,
        Err(err) => refuse(&err),
    };
    if let Err(err) = config.csp.validate() {
        refuse(&err);
    }
    if let Err(err) = config.sentry.validate() {
        refuse(&err);
    }
    if let Err(err) = config.hsts.validate() {
        refuse(&err);
    }
    config
}
