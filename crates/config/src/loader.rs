//! The Portfolio dialect of [`terrace_config`].
//!
//! The layering itself belongs to `terrace-config`: the TOML fragments, the `PORTFOLIO_`
//! environment layer, the secrets-directory provider, the `_FILE` indirection and the shadow-key
//! rejection. What stays here is which environment names this deployment spells.
//!
//! Nothing is reserved beyond the two names `terrace-config` reserves itself, `PORTFOLIO_CONFIG`
//! and `PORTFOLIO_SECRETS_DIR`. Reserving exists for keys read straight from the environment
//! outside the layered config, and every other variable this workspace reads is outside the
//! `PORTFOLIO_` namespace: `IP`, `PORT` and `RUST_LOG` belong to the Dioxus toolchain and `CI` to
//! the CI provider, so no file layer could name one.

use serde::de::DeserializeOwned;
use terrace_config::Terrace;

pub use terrace_config::Error as ConfigError;

/// The prefix every configuration variable carries.
const PREFIX: &str = "PORTFOLIO_";

/// Returns the loader every binary in this workspace boots through.
///
/// Layers, lowest precedence first: struct defaults, TOML at `$PORTFOLIO_CONFIG` (a file, or
/// every `*.toml` in it if it names a directory), `PORTFOLIO_`-prefixed `__`-nested environment
/// variables, `$PORTFOLIO_SECRETS_DIR`, and `PORTFOLIO_<KEY>_FILE` indirection. The last three
/// are mutually exclusive per key: a key supplied by two of them is refused at boot, because a
/// stale environment variable shadowing a rotated mounted secret keeps the process running on
/// the old credential.
// Both variable names are literals, not the derivation `Terrace::new(PREFIX)` performs, so the
// surface documented in `README.md` can be checked against this file.
#[must_use]
pub fn terrace() -> Terrace {
    Terrace::new(PREFIX)
        .config_var("PORTFOLIO_CONFIG")
        .secrets_dir_var("PORTFOLIO_SECRETS_DIR")
}

/// Loads a typed configuration through the layers [`terrace`] describes.
///
/// # Errors
///
/// Returns [`ConfigError`] when a required value is missing, a value fails to parse, a
/// file-backed source cannot be read, or one key is supplied by more than one of the
/// environment, the secrets directory and `_FILE` indirection.
///
/// # Examples
///
/// ```no_run
/// # fn main() -> Result<(), portfolio_config::ConfigError> {
/// let config: portfolio_config::ServerConfig = portfolio_config::load()?;
/// config.hsts.validate().expect("a servable HSTS block");
/// # Ok(())
/// # }
/// ```
pub fn load<T: DeserializeOwned>() -> Result<T, ConfigError> {
    terrace().load()
}

/// Returns a report of which layer supplied each key, to print beside a failure from [`load`].
///
/// The error names the key and this names the mount, which together answer why a value is not
/// the one the operator set. A key supplied twice appears once with both of its sources, so the
/// report still works for the configuration that stopped the boot.
///
/// The report records where each key came from and never its value, so it is safe to print into
/// a log that is shipped and retained. It is text whatever happens: a failure to assemble it is
/// reported in the text.
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
        #[serde(default)]
        legal: terrace_legal::LegalConfig,
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
            // A field the environment did not touch still materializes with its default.
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
    /// editor and a `BuildKit` secret all add one, and a DSN that keeps it does not parse.
    #[test]
    fn the_dsn_arrives_through_file_indirection_and_loses_its_newline() {
        harness().run(|jail| {
            jail.env_key("sentry.enabled", true);
            jail.indirection("sentry.dsn", "https://key@sentry.example/42\n")?;

            let config: Sample = jail.load()?;
            assert!(config.sentry.is_active());
            assert_eq!(config.sentry.dsn(), Some("https://key@sentry.example/42"));
            assert_eq!(config.sentry.validate(), Ok(()));
            // The block materializes around what was supplied rather than replacing it: nothing
            // here turns performance tracing on, and nothing here widens what an event carries.
            assert!((config.sentry.traces_sample_rate - 0.0).abs() < f32::EPSILON);
            assert!(!config.sentry.send_default_pii);
            Ok(())
        });
    }

    /// The two ways a legal text reaches the server, in the shape a deployment uses them: a
    /// TOML fragment carrying the metadata and one body inline, and a mounted Markdown file
    /// named by `_FILE` indirection for the other. The locale sits two levels below the slug,
    /// which is the deepest key in the workspace and the one a hand-written variable name gets
    /// wrong first.
    #[test]
    fn legal_texts_arrive_inline_and_through_file_indirection() {
        harness().run(|jail| {
            jail.config(
                "[legal]
default_locale = \"en\"

                 [legal.documents.imprint]
updated = \"2026-06-10\"
order = 10
                 title = { en = \"Imprint\", de = \"Impressum\" }
                 body.en = '''
## Information according to § 5 DDG
'''
",
            )?;
            jail.indirection(
                "legal.documents.imprint.body.de",
                "## Angaben gemäß § 5 DDG
",
            )?;

            let config: Sample = jail.load()?;
            let imprint = &config.legal.documents["imprint"];
            assert_eq!(config.legal.default_locale.as_deref(), Some("en"));
            assert_eq!(imprint.order, 10);
            assert_eq!(imprint.title["de"], "Impressum");
            assert!(imprint.body["en"].contains("§ 5 DDG"));
            assert!(imprint.body["de"].starts_with("## Angaben"));
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
