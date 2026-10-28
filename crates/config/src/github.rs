//! What the `portfolio-update-repos` builder needs to talk to the GitHub REST API.

use secrecy::SecretString;
use serde::Deserialize;

/// Credentials and scope for the build-time repository listing.
///
/// Every field is optional. An unset user means the one in the compile-time site config, an unset
/// token means an unauthenticated request against the lower rate limit, and an empty repository
/// list means every active repository the user owns.
#[derive(Debug, Clone, Default, Deserialize)]
#[cfg_attr(
    feature = "config-schema",
    derive(serde::Serialize, terrace_config::schema::Describe)
)]
pub struct GithubConfig {
    /// The user whose repositories to list.
    ///
    /// Unset falls back to the site's own identity in `portfolio_data::CONFIG`, which the builder
    /// resolves.
    #[cfg_attr(
        feature = "config-schema",
        config(note = "the site's own `CONFIG.github_username`")
    )]
    #[serde(default)]
    pub username: Option<String>,
    /// The bearer token that lifts the GitHub API rate limit.
    ///
    /// Optional: the listing is public, so an unauthenticated build still works, just against the
    /// anonymous quota.
    ///
    /// One of the workspace's two secrets, beside `sentry.dsn`. Supply it as
    /// `PORTFOLIO_GITHUB__TOKEN_FILE=/run/secrets/gh_token` (or from a secrets directory) so it
    /// never enters the process environment, where `/proc/<pid>/environ`, a crash dump or a
    /// `docker inspect` would carry it.
    // Not rustdoc: every `///` on a field in this crate is rendered into `config.example.toml`
    // and into the README table, for an operator who is not reading the source. Why the
    // attribute is `skip_serializing` is for whoever changes this line.
    //
    // `skip_serializing` rather than an impl: `SecretString` deliberately has no `Serialize`,
    // and the schema generator serializes the default config to read the `Default` column out of
    // it. The key still appears in the table — the derive reports it either way — with `unset`
    // for a default it never had, which is both true and the only safe thing to print. This is
    // the pattern `terrace-config` documents for a secret-bearing field.
    #[cfg_attr(feature = "config-schema", config(secret))]
    #[serde(default, skip_serializing)]
    pub token: Option<SecretString>,
    /// The repositories to list, in place of every active repository and its filtering.
    ///
    /// It is an array in both spellings: `repos = ["Portfolio", "actions"]` in TOML, and
    /// `PORTFOLIO_GITHUB__REPOS=[Portfolio,actions]` in the environment. A bare comma-separated
    /// string is not accepted.
    #[cfg_attr(
        feature = "config-schema",
        config(note = "every active repository the user owns")
    )]
    #[serde(default)]
    pub repos: Vec<String>,
}

impl GithubConfig {
    /// Returns the configured user, or `None` when unset or blank.
    #[must_use]
    pub fn username(&self) -> Option<&str> {
        self.username.as_deref().map(str::trim).filter(is_present)
    }

    /// Returns the bearer token, or `None` when unset or blank.
    ///
    /// CI passes the token through unconditionally, and a fork's pull request has no secret to
    /// pass, so a blank value is the normal case there.
    // Consuming because `SecretString` is deliberately not `Clone`: handing the secret on is a
    // move, which keeps "who holds this" answerable.
    #[must_use]
    pub fn into_token(self) -> Option<SecretString> {
        use secrecy::ExposeSecret as _;
        self.token
            .filter(|token| is_present(&token.expose_secret().trim()))
    }

    /// Returns the configured repository names, with blank entries dropped.
    pub fn repos(&self) -> impl Iterator<Item = &str> {
        self.repos.iter().map(|name| name.trim()).filter(is_present)
    }
}

/// Whether a supplied value carries anything. See [`GithubConfig::username`].
fn is_present(value: &&str) -> bool {
    !value.is_empty()
}

#[cfg(test)]
mod tests {
    use super::GithubConfig;
    use secrecy::SecretString;

    #[test]
    fn an_unset_github_block_asks_for_nothing() {
        let config = GithubConfig::default();
        assert_eq!(config.username(), None);
        assert_eq!(config.repos().count(), 0);
        assert!(config.into_token().is_none());
    }

    /// CI passes the token through unconditionally, so a fork's pull request supplies an empty
    /// one. That has to read as "unauthenticated", not as a token of length zero.
    #[test]
    fn a_blank_token_is_unauthenticated_rather_than_a_bad_credential() {
        let config = GithubConfig {
            token: Some(SecretString::from("   ")),
            ..GithubConfig::default()
        };
        assert!(config.into_token().is_none());
    }

    #[test]
    fn blank_repository_names_are_dropped() {
        let config = GithubConfig {
            repos: vec![
                " Portfolio ".to_owned(),
                String::new(),
                "actions".to_owned(),
            ],
            ..GithubConfig::default()
        };
        assert_eq!(config.repos().collect::<Vec<_>>(), ["Portfolio", "actions"]);
    }
}
