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

#[cfg(test)]
mod tests {
    use crate::{CspConfig, GithubConfig, IsrConfig, load};
    use secrecy::ExposeSecret as _;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct Sample {
        #[serde(default)]
        csp: CspConfig,
        #[serde(default)]
        github: GithubConfig,
        #[serde(default)]
        isr: IsrConfig,
    }

    /// The dialect end to end: the prefix, the `__` nesting, and the defaults that fill in
    /// around what the environment supplied. `terrace-config` owns the layering and tests it;
    /// what this pins is that this crate wires it to the names an operator actually sets.
    #[test]
    // `figment::Jail::expect_with` fixes the closure's error type to the large `figment::Error`.
    #[expect(
        clippy::result_large_err,
        reason = "figment::Jail::expect_with fixes the closure's error type"
    )]
    fn the_environment_layer_speaks_the_documented_names() {
        figment::Jail::expect_with(|jail| {
            jail.clear_env();
            jail.set_env("PORTFOLIO_GITHUB__USERNAME", "TimSchoenle");
            jail.set_env("PORTFOLIO_ISR__TTL_SECS", "3600");
            // Two levels of nesting, which is the deepest key this workspace has and the one
            // spelling a README table could quietly get wrong.
            jail.set_env("PORTFOLIO_CSP__CLOUDFLARE__TURNSTILE", "true");

            let config: Sample = load().map_err(|e| e.to_string()).unwrap();
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
    /// win over the `Secret` that carries the real token — through the variable names *this*
    /// crate configures, which is the half a dependency cannot pin.
    #[test]
    #[expect(
        clippy::result_large_err,
        reason = "figment::Jail::expect_with fixes the closure's error type"
    )]
    fn a_secrets_directory_outranks_the_toml_layer() {
        figment::Jail::expect_with(|jail| {
            jail.clear_env();
            jail.create_file("config.toml", "[github]\ntoken = \"placeholder\"\n")?;
            jail.create_dir("secrets")?;
            jail.create_file("secrets/github__token", "ghp_real\n")?;
            jail.set_env(
                "PORTFOLIO_CONFIG",
                jail.directory().join("config.toml").display(),
            );
            jail.set_env(
                "PORTFOLIO_SECRETS_DIR",
                jail.directory().join("secrets").display(),
            );

            let config: Sample = load().map_err(|e| e.to_string()).unwrap();
            let token = config
                .github
                .into_token()
                .expect("the secret supplies a token");
            assert_eq!(token.expose_secret(), "ghp_real");
            Ok(())
        });
    }

    /// A key supplied by both the environment and a mounted file fails the boot instead of one
    /// silently winning. This is the whole reason the loader is `terrace-config` and not a bare
    /// figment: a `GITHUB_TOKEN` left behind by a half-finished migration must not keep the
    /// build running on a credential the operator believes they rotated.
    #[test]
    #[expect(
        clippy::result_large_err,
        reason = "figment::Jail::expect_with fixes the closure's error type"
    )]
    fn one_key_supplied_twice_is_refused() {
        figment::Jail::expect_with(|jail| {
            jail.clear_env();
            jail.create_dir("secrets")?;
            jail.create_file("secrets/github__token", "ghp_from_file")?;
            jail.set_env(
                "PORTFOLIO_SECRETS_DIR",
                jail.directory().join("secrets").display(),
            );
            jail.set_env("PORTFOLIO_GITHUB__TOKEN", "ghp_from_env");

            let error = load::<Sample>().expect_err("two sources for one key must not load");
            assert!(
                error.to_string().contains("github.token"),
                "the error must name the key: {error}"
            );
            Ok(())
        });
    }
}
