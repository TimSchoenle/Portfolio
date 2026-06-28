//! A small builder for fetching a configured set of GitHub repositories and
//! assembling them into the shared [`portfolio_data::ReposFile`] model.
//!
//! Rather than listing every public repository of a user, the builder fetches
//! only the explicitly configured repositories by name (one
//! `GET /repos/{user}/{name}` request each), so `repos.json` mirrors exactly the
//! projects declared in `CONFIG.repos`.
//!
//! The builder is deliberately decoupled from I/O concerns: [`ReposBuilder::fetch`]
//! talks to the GitHub REST API and returns a fully-populated `ReposFile`, while
//! [`ReposBuilder::assemble`] performs the pure assembly step (timestamp + user +
//! repos) so it can be unit-tested without any network access.

use std::time::Duration;

use portfolio_data::{Repo, ReposFile};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::error::UpdateReposError;

/// Default GitHub REST API base URL.
const GITHUB_API_BASE: &str = "https://api.github.com";
/// Identifies this client to the GitHub API (a User-Agent is mandatory).
const USER_AGENT: &str = concat!("portfolio-update-repos/", env!("CARGO_PKG_VERSION"));

/// Fluent builder that fetches a specific set of repositories and produces a
/// [`ReposFile`] ready to be serialized to `repos.json`.
pub struct ReposBuilder {
    user: String,
    token: Option<String>,
    repos: Vec<String>,
    api_base: String,
    agent: ureq::Agent,
}

impl ReposBuilder {
    /// Starts a builder for the given GitHub `user`.
    pub fn new(user: impl Into<String>) -> Self {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(30)))
            .build()
            .into();
        Self {
            user: user.into(),
            token: None,
            repos: Vec::new(),
            api_base: GITHUB_API_BASE.to_string(),
            agent,
        }
    }

    /// Sets the bearer token used to authenticate API requests (lifts the rate
    /// limit and is required for the workflow's `GITHUB_TOKEN`). A `None` or
    /// empty token leaves requests unauthenticated.
    pub fn token(mut self, token: Option<String>) -> Self {
        self.token = token.filter(|t| !t.is_empty());
        self
    }

    /// Sets the exact list of repository names to fetch (empty names are
    /// ignored). Their order is preserved in the resulting [`ReposFile`].
    pub fn repos<I, S>(mut self, repos: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.repos = repos
            .into_iter()
            .map(Into::into)
            .filter(|name| !name.is_empty())
            .collect();
        self
    }

    /// Fetches each configured repository by name and assembles them into a
    /// [`ReposFile`], preserving the configured order.
    pub fn fetch(&self) -> Result<ReposFile, UpdateReposError> {
        if self.user.is_empty() {
            return Err(UpdateReposError::MissingUser);
        }
        if self.repos.is_empty() {
            return Err(UpdateReposError::NoRepos);
        }

        let mut repos = Vec::with_capacity(self.repos.len());
        for name in &self.repos {
            repos.push(self.fetch_repo(name)?);
        }

        self.assemble(repos)
    }

    /// Fetches a single repository by name, deserializing GitHub's JSON directly
    /// into the shared [`Repo`] model (unknown fields are ignored).
    fn fetch_repo(&self, name: &str) -> Result<Repo, UpdateReposError> {
        let url = format!(
            "{base}/repos/{user}/{name}",
            base = self.api_base,
            user = self.user,
        );

        let mut request = self
            .agent
            .get(&url)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .header("User-Agent", USER_AGENT);
        if let Some(token) = &self.token {
            request = request.header("Authorization", format!("Bearer {token}"));
        }

        let repo = request.call()?.body_mut().read_json::<Repo>()?;
        Ok(repo)
    }

    /// Pure assembly step: stamps the current time and wraps the repositories
    /// in a [`ReposFile`]. Exposed for unit testing without network access.
    pub fn assemble(&self, repos: Vec<Repo>) -> Result<ReposFile, UpdateReposError> {
        let generated_at = OffsetDateTime::now_utc().format(&Rfc3339)?;
        Ok(ReposFile {
            generated_at,
            user: self.user.clone(),
            repos,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"[
        {
            "name": "Portfolio",
            "full_name": "timschoenle/Portfolio",
            "description": "My portfolio",
            "html_url": "https://github.com/timschoenle/Portfolio",
            "language": "Rust",
            "stargazers_count": 7,
            "forks_count": 2,
            "updated_at": "2026-06-01T00:00:00Z",
            "topics": ["rust", "yew"],
            "fork": false,
            "archived": false,
            "homepage": "https://tim-schoenle.de",
            "ignored_extra_field": 123
        },
        {
            "name": "helm-charts",
            "full_name": "timschoenle/helm-charts",
            "description": null,
            "html_url": "https://github.com/timschoenle/helm-charts",
            "language": null,
            "stargazers_count": 0,
            "forks_count": 0,
            "updated_at": "2026-05-01T00:00:00Z",
            "fork": true,
            "archived": false,
            "homepage": null
        }
    ]"#;

    #[test]
    fn github_json_maps_into_repo_model() {
        let repos: Vec<Repo> = serde_json::from_str(SAMPLE).expect("sample parses");
        assert_eq!(repos.len(), 2);

        let first = &repos[0];
        assert_eq!(first.name, "Portfolio");
        assert_eq!(first.language.as_deref(), Some("Rust"));
        assert_eq!(first.stargazers_count, 7);
        assert_eq!(first.topics, vec!["rust", "yew"]);
        assert!(first.is_featured(), "Portfolio is in CONFIG.featured_repos");

        let second = &repos[1];
        assert_eq!(second.description, None);
        assert_eq!(second.language, None);
        assert!(second.fork);
        assert!(second.topics.is_empty(), "missing topics default to empty");
    }

    #[test]
    fn into_repos_file_preserves_user_and_order() {
        let repos: Vec<Repo> = serde_json::from_str(SAMPLE).unwrap();
        let builder = ReposBuilder::new("timschoenle");
        let file = builder.assemble(repos).expect("assembles");

        assert_eq!(file.user, "timschoenle");
        assert_eq!(file.repos.len(), 2);
        assert_eq!(file.repos[0].name, "Portfolio");
        assert_eq!(file.repos[1].name, "helm-charts");
        assert!(
            !file.generated_at.is_empty(),
            "generated_at is an RFC 3339 timestamp"
        );
    }

    #[test]
    fn empty_user_is_rejected() {
        let err = ReposBuilder::new("")
            .repos(["Portfolio"])
            .fetch()
            .unwrap_err();
        assert!(matches!(err, UpdateReposError::MissingUser));
    }

    #[test]
    fn no_repos_is_rejected() {
        let err = ReposBuilder::new("u").fetch().unwrap_err();
        assert!(matches!(err, UpdateReposError::NoRepos));
    }

    #[test]
    fn repos_ignores_empty_names_and_preserves_order() {
        let builder = ReposBuilder::new("u").repos(["Portfolio", "", "helm-charts"]);
        assert_eq!(builder.repos, vec!["Portfolio", "helm-charts"]);
    }
}
