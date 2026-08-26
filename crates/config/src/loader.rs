//! The Portfolio dialect of [`terrace_config`].
//!
//! The layering itself — the TOML fragments, the `PORTFOLIO_` environment layer, the
//! secrets-directory provider, the `_FILE` indirection and the shadow-key rejection — belongs
//! to `terrace-config`. What stays here is the one thing that is ours: which environment names
//! this deployment spells.

use serde::de::DeserializeOwned;
use terrace_config::Terrace;

pub use terrace_config::Error as ConfigError;

/// The prefix every configuration variable carries.
const PREFIX: &str = "PORTFOLIO_";

/// The loader every binary in this workspace boots through.
///
/// Layers, lowest precedence first: struct defaults, TOML at `$PORTFOLIO_CONFIG` (a file, or
/// every `*.toml` in it if it names a directory), `PORTFOLIO_`-prefixed `__`-nested environment
/// variables, `$PORTFOLIO_SECRETS_DIR`, and `PORTFOLIO_<KEY>_FILE` indirection. The last three
/// are mutually exclusive per key: a key supplied by two of them is refused at boot rather than
/// resolved by precedence, because a stale environment variable shadowing a rotated mounted
/// secret keeps the process running on the old credential.
///
/// Nothing is [`reserved`](Terrace::reserve) beyond the two names `terrace-config` reserves
/// itself (`PORTFOLIO_CONFIG` and `PORTFOLIO_SECRETS_DIR`, both read to decide what the layers
/// *are*). Reserving exists for keys read straight from the environment outside the layered
/// config, and this workspace has none: every remaining variable it reads —  `IP`, `PORT` and
/// `RUST_LOG`, which belong to the Dioxus toolchain, and `CI`, which belongs to the CI provider
/// — is outside the `PORTFOLIO_` namespace entirely, so no file layer could name one.
///
/// Both variable names are stated as literals rather than left to the derivation
/// `Terrace::new(PREFIX)` performs, so the documented surface in `README.md` can be checked
/// against this file instead of against a dependency's internals.
#[must_use]
pub fn terrace() -> Terrace {
    Terrace::new(PREFIX)
        .config_var("PORTFOLIO_CONFIG")
        .secrets_dir_var("PORTFOLIO_SECRETS_DIR")
}

/// Load a typed config through the layers above.
///
/// # Errors
/// Returns [`ConfigError`] if a required value is missing, a value fails to parse, a
/// file-backed source cannot be read, or one key is supplied by more than one of the last three
/// layers.
pub fn load<T: DeserializeOwned>() -> Result<T, ConfigError> {
    terrace().load()
}

/// Which layer supplied each key, as a block to print beside a failure.
///
/// [`load`]'s error names the key; this names the mount. Between them they answer the question
/// an operator is actually asking — *why is the value not the one I set* — without a debugger,
/// a rebuild, or a shell in an image that has none.
///
/// Assembled independently of the shadow policy, so the configuration this is most useful for
/// is the one it can still report: a key supplied twice appears once with both of its sources
/// rather than stopping the report the way it stopped the boot.
///
/// # Nothing here holds a value
///
/// The report records *where* each key came from and never *what* it was — a property of
/// `terrace-config`'s `Explanation` type, which has no field to leak, rather than of remembering
/// to redact. That is what makes printing one into a log that is shipped and retained safe by
/// default, and it is why [`GithubConfig::token`](crate::GithubConfig::token) needs no special
/// handling here.
///
/// Returns text whatever happens, the failure to assemble it included: this is called on a path
/// that is already ending the process, and a diagnostic with an error of its own to handle would
/// not be one.
#[must_use]
pub fn provenance() -> String {
    match terrace().explain() {
        Ok(explanation) => explanation.to_string(),
        Err(err) => format!("the configuration layers could not be reported: {err}"),
    }
}

#[cfg(test)]
mod tests {
    use crate::{CspConfig, GithubConfig, IsrConfig, SentryConfig, terrace};
    use secrecy::ExposeSecret as _;
    use serde::Deserialize;
    use terrace_config::testing::Harness;

    #[derive(Debug, Deserialize)]
    struct Sample {
        #[serde(default)]
        csp: CspConfig,
        #[serde(default)]
        github: GithubConfig,
        #[serde(default)]
        isr: IsrConfig,
        #[serde(default)]
        sentry: SentryConfig,
    }

    /// A sandbox over the loader [`terrace`] builds, so every name a test arranges is derived
    /// from the dialect under test rather than typed out beside it. That is the half worth
    /// pinning here: `terrace-config` owns the layering and tests it, and what this crate adds
    /// is the wiring to the names an operator actually sets.
    fn harness() -> Harness {
        Harness::over(terrace())
    }

    /// The dialect end to end: the prefix, the `__` nesting, and the defaults that fill in
    /// around what the environment supplied.
    ///
    /// The variables are spelled out rather than derived through `Jail::env_key`, because the
    /// spelling *is* the assertion — a `PORTFOLIO_` prefix and `__` for nesting are what
    /// `README.md` and every deployment manifest state, and a derived name would agree with the
    /// loader however the loader changed.
    #[test]
    fn the_environment_layer_speaks_the_documented_names() {
        harness().run(|jail| {
            jail.env("PORTFOLIO_GITHUB__USERNAME", "TimSchoenle");
            jail.env("PORTFOLIO_ISR__TTL_SECS", "3600");
            // Two levels of nesting, which is the deepest key this workspace has and the one
            // spelling a README table could quietly get wrong.
            jail.env("PORTFOLIO_CSP__CLOUDFLARE__TURNSTILE", "true");

            let config: Sample = jail.load()?;
            assert_eq!(config.github.username(), Some("TimSchoenle"));
            assert!(config.csp.cloudflare.turnstile);
            // Its siblings keep their defaults rather than being reset by the nested override.
            assert!(config.csp.cloudflare.script_nonce);
            assert!(config.csp.hash_inline_scripts);
            assert_eq!(
                config.isr.invalidate_after(),
                Some(std::time::Duration::from_secs(3600))
            );
            // A field the environment did not touch still materialises with its default.
            assert_eq!(config.isr.cache_dir(), None);
            Ok(())
        });
    }

    /// A mounted secret outranks the TOML layer, so a `ConfigMap` carrying a placeholder cannot
    /// win over the `Secret` that carries the real token.
    #[test]
    fn a_secrets_directory_outranks_the_toml_layer() {
        harness().run(|jail| {
            jail.config("[github]\ntoken = \"placeholder\"\n")?;
            jail.secret_key("github.token", "ghp_real\n")?;

            let config: Sample = jail.load()?;
            let token = config
                .github
                .into_token()
                .expect("the secret supplies a token");
            assert_eq!(token.expose_secret(), "ghp_real");
            Ok(())
        });
    }

    /// The other secret in the workspace, through the layer it is meant to arrive on.
    ///
    /// `sentry.dsn` is the only key the *server* mounts as a file, so this is where the
    /// `PORTFOLIO_<KEY>_FILE` indirection stops being a documented feature and becomes a tested
    /// one. The trailing newline is not incidental: `printf` into a Kubernetes `Secret`, an
    /// editor and a BuildKit secret all add one, and a DSN that keeps it does not parse.
    #[test]
    fn the_dsn_arrives_through_file_indirection_and_loses_its_newline() {
        harness().run(|jail| {
            jail.env_key("sentry.enabled", true);
            jail.indirection("sentry.dsn", "https://key@sentry.example/42\n")?;

            let config: Sample = jail.load()?;
            assert!(config.sentry.is_active());
            assert_eq!(config.sentry.dsn(), Some("https://key@sentry.example/42"));
            assert_eq!(config.sentry.validate(), Ok(()));
            // The block materialises around what was supplied rather than replacing it: nothing
            // here turns performance tracing on, and nothing here widens what an event carries.
            assert!((config.sentry.traces_sample_rate - 0.0).abs() < f32::EPSILON);
            assert!(!config.sentry.send_default_pii);
            Ok(())
        });
    }

    /// A key supplied by both the environment and a mounted file fails the boot instead of one
    /// silently winning. This is the whole reason the loader is `terrace-config` and not a bare
    /// figment: a `GITHUB_TOKEN` left behind by a half-finished migration must not keep the
    /// build running on a credential the operator believes they rotated.
    #[test]
    fn one_key_supplied_twice_is_refused() {
        harness().run(|jail| {
            jail.secret_key("github.token", "ghp_from_file")?;
            jail.env_key("github.token", "ghp_from_env");

            let error = jail
                .load::<Sample>()
                .expect_err("two sources for one key must not load");
            assert!(
                error.to_string().contains("github.token"),
                "the error must name the key: {error}"
            );
            Ok(())
        });
    }

    /// The report an operator gets when the boot above is refused: both mounts named, neither
    /// value shown. [`crate::provenance`] is what prints it, and what it prints has to be the
    /// contested key rather than a summary that omits it.
    #[test]
    fn the_report_names_every_layer_that_supplied_a_contested_key() {
        harness().run(|jail| {
            jail.secret_key("github.token", "ghp_from_file")?;
            jail.env_key("github.token", "ghp_from_env");

            let report = jail.explain()?.to_string();
            assert!(
                report.contains("github.token"),
                "the report must name the key: {report}"
            );
            assert!(
                report.contains("PORTFOLIO_GITHUB__TOKEN"),
                "the report must name the variable that shadowed the mount: {report}"
            );
            assert!(
                !report.contains("ghp_from_file") && !report.contains("ghp_from_env"),
                "the report must carry no value: {report}"
            );
            Ok(())
        });
    }
}
