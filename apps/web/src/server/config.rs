//! The server's configuration, loaded once before anything binds, and the exit that refuses to
//! start without a usable one.

use portfolio_config::ServerConfig;

/// Exit code for a configuration the process cannot start with (`EX_CONFIG` from `sysexits.h`),
/// so an operator can tell a bad config apart from a crash without reading the logs.
const EX_CONFIG: i32 = 78;

/// Ends the process with [`EX_CONFIG`], naming what could not be started with.
///
/// Runs before the serve loop installs a logger — and, when Sentry is on, before this server
/// installs one — so `tracing` would discard this; it goes to stderr directly.
///
/// The error names the key; the report under it names the layer that supplied it, which is the
/// half an operator cannot get at from inside a distroless image with no shell. Neither holds a
/// configuration value, so both are safe in a log that is shipped and retained.
pub(super) fn refuse(err: &dyn std::fmt::Display) -> ! {
    eprintln!("portfolio: cannot start, the configuration is not usable: {err}");
    eprintln!("{}", portfolio_config::provenance());
    std::process::exit(EX_CONFIG)
}

/// Reads the configuration, or ends the process with [`EX_CONFIG`].
///
/// Two ways to be unusable, and both are start-up failures: a value that cannot be *loaded* (a
/// missing file, an unparseable number, one key supplied by two layers), and a set of values that
/// load individually but cannot be *served* together — see
/// [`CspConfig::validate`](portfolio_config::CspConfig::validate) and
/// [`SentryConfig::validate`](portfolio_config::SentryConfig::validate) and
/// [`HstsConfig::validate`](portfolio_config::HstsConfig::validate). Failing here means the
/// container never reports ready, rather than serving every visitor a blank page or reporting its
/// errors into a void.
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
