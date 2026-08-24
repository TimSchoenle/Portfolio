//! The GitHub half of `update-repos`: what is listed, what is kept, and what is thrown away.
//!
//! Two modes, and they are not variations of one another. With no explicit names the builder
//! pages through everything the user owns and then filters: archived, blacklisted, or untouched
//! for [`MAX_AGE_DAYS`] days is dropped. With names supplied it fetches each one and filters
//! nothing, because a repository somebody asked for by name is not one that needs judging.
//!
//! Only [`ReposBuilder::fetch`] and the two functions under it touch the network. Everything the
//! tests care about — the filter, the assembly — is reachable without it, which is why
//! `active_repos` takes `now` as an argument instead of reading the clock.

use std::time::Duration;

use portfolio_data::{Repo, ReposFile};
use secrecy::{ExposeSecret as _, SecretString};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::error::UpdateReposError;

/// Default GitHub REST API base URL.
const GITHUB_API_BASE: &str = "https://api.github.com";
/// Identifies this client to the GitHub API (a User-Agent is mandatory).
const USER_AGENT: &str = concat!("portfolio-update-repos/", env!("CARGO_PKG_VERSION"));
/// Page size used when listing all of a user's repositories (GitHub's maximum).
const PER_PAGE: u32 = 100;
/// Repositories with no update within this many days are considered stale and
/// dropped from the listing.
const MAX_AGE_DAYS: i64 = 365;

/// Fluent builder that fetches a specific set of repositories and produces a
/// [`ReposFile`] ready to be serialized to `repos.json`.
pub struct ReposBuilder {
    user: String,
    token: Option<SecretString>,
    repos: Vec<String>,
    blacklist: Vec<String>,
    api_base: String,
    agent: ureq::Agent,
}

impl ReposBuilder {
    /// A builder against the public GitHub API, unauthenticated, with no repository named.
    ///
    /// The agent it constructs has a 30-second global timeout covering connect, send and read
    /// together. `update-repos` runs inside an image build, where a hung socket is a hung build
    /// and no retry is attempted.
    pub fn new(user: impl Into<String>) -> Self {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(30)))
            .build()
            .into();
        Self {
            user: user.into(),
            token: None,
            repos: Vec::new(),
            blacklist: Vec::new(),
            api_base: GITHUB_API_BASE.to_string(),
            agent,
        }
    }

    /// Authenticates every subsequent request, lifting the API rate limit.
    ///
    /// `None` or an all-empty token leaves the requests unauthenticated. That is the normal case
    /// on a fork's pull request, where there is no secret to pass, rather than a misconfiguration
    /// worth failing over.
    ///
    /// Kept wrapped rather than taken as a `String`: the token arrives from a mounted file and is
    /// never printed, so the only place it becomes a `&str` is [`Self::get`], where it goes onto
    /// the wire.
    pub fn token(mut self, token: Option<SecretString>) -> Self {
        self.token = token.filter(|token| !token.expose_secret().is_empty());
        self
    }

    /// Names the repositories to fetch, in place of listing the account.
    ///
    /// Blank names are dropped. The rest keep their order all the way into the [`ReposFile`], and
    /// none of them are filtered for age, archival or the blacklist.
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

    /// Excludes repositories by name, case-insensitively, when the account is listed.
    ///
    /// Ignored entirely by [`Self::repos`], which fetches what it was asked for.
    pub fn blacklist<I, S>(mut self, blacklist: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.blacklist = blacklist
            .into_iter()
            .map(Into::into)
            .filter(|name| !name.is_empty())
            .collect();
        self
    }

    /// Fetches the user's repositories and assembles them into a [`ReposFile`].
    ///
    /// When no explicit repository names are configured (the default), every
    /// repository the user owns is listed and the archived ones are filtered
    /// out. Otherwise only the configured repositories are fetched by name,
    /// preserving their order.
    pub fn fetch(&self) -> Result<ReposFile, UpdateReposError> {
        if self.user.is_empty() {
            return Err(UpdateReposError::MissingUser);
        }

        let repos = if self.repos.is_empty() {
            self.fetch_all()?
        } else {
            let mut repos = Vec::with_capacity(self.repos.len());
            for name in &self.repos {
                repos.push(self.fetch_repo(name)?);
            }
            repos
        };

        if repos.is_empty() {
            return Err(UpdateReposError::NoRepos);
        }

        self.assemble(repos)
    }

    /// Lists every repository the user owns (following pagination) and keeps
    /// only the active repositories (not archived, not blacklisted and updated
    /// within the last [`MAX_AGE_DAYS`] days).
    fn fetch_all(&self) -> Result<Vec<Repo>, UpdateReposError> {
        let mut all = Vec::new();
        let mut page = 1u32;
        loop {
            let url = format!(
                "{base}/users/{user}/repos?per_page={PER_PAGE}&page={page}&type=owner&sort=updated",
                base = self.api_base,
                user = self.user,
            );

            let batch = self.get(&url).call()?.body_mut().read_json::<Vec<Repo>>()?;
            let fetched = batch.len();
            all.extend(batch);

            // A short page signals the last one; stop before an empty request.
            if (fetched as u32) < PER_PAGE {
                break;
            }
            page += 1;
        }

        Ok(Self::active_repos(
            all,
            &self.blacklist,
            OffsetDateTime::now_utc(),
        ))
    }

    /// The active repositories out of `repos`, in the order they arrived.
    ///
    /// Archived, blacklisted, and last updated more than [`MAX_AGE_DAYS`] days before `now` are
    /// each dropped. A repository whose `updated_at` is missing or unparsable is kept. `now` is a
    /// parameter rather than a clock read so the age boundary is a thing a test can stand on.
    fn active_repos(repos: Vec<Repo>, blacklist: &[String], now: OffsetDateTime) -> Vec<Repo> {
        let cutoff = now - time::Duration::days(MAX_AGE_DAYS);
        repos
            .into_iter()
            .filter(|repo| !repo.archived)
            .filter(|repo| !Self::is_blacklisted(&repo.name, blacklist))
            .filter(|repo| Self::is_recent(repo, cutoff))
            .collect()
    }

    /// `true` when `name` matches an entry on the blacklist (case-insensitive).
    fn is_blacklisted(name: &str, blacklist: &[String]) -> bool {
        blacklist.iter().any(|b| b.eq_ignore_ascii_case(name))
    }

    /// `true` when the repository's `updated_at` is at or after `cutoff`.
    /// Repositories with a missing or unparsable timestamp are kept.
    fn is_recent(repo: &Repo, cutoff: OffsetDateTime) -> bool {
        match OffsetDateTime::parse(&repo.updated_at, &Rfc3339) {
            Ok(updated) => updated >= cutoff,
            Err(_) => true,
        }
    }

    /// Builds a GET request to `url` with the shared GitHub API headers (and the
    /// bearer token when configured).
    fn get(&self, url: &str) -> ureq::RequestBuilder<ureq::typestate::WithoutBody> {
        let mut request = self
            .agent
            .get(url)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .header("User-Agent", USER_AGENT);
        if let Some(token) = &self.token {
            request = request.header("Authorization", format!("Bearer {}", token.expose_secret()));
        }
        request
    }

    /// Fetches a single repository by name, deserializing GitHub's JSON directly
    /// into the shared [`Repo`] model (unknown fields are ignored).
    fn fetch_repo(&self, name: &str) -> Result<Repo, UpdateReposError> {
        let url = format!(
            "{base}/repos/{user}/{name}",
            base = self.api_base,
            user = self.user,
        );

        let repo = self.get(&url).call()?.body_mut().read_json::<Repo>()?;
        Ok(repo)
    }

    /// The listing, stamped with the current UTC time in RFC 3339.
    ///
    /// That stamp is what the next build's freshness check reads, so it is taken after the fetch
    /// rather than when the builder was created.
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

    /// A fixed "now" used by the filter tests; `SAMPLE`'s timestamps
    /// (2026-05/06) fall comfortably inside the one-year window from here.
    fn now() -> OffsetDateTime {
        OffsetDateTime::parse("2026-06-28T00:00:00Z", &Rfc3339).unwrap()
    }

    #[test]
    fn active_repos_drops_archived_and_keeps_order() {
        let mut repos: Vec<Repo> = serde_json::from_str(SAMPLE).unwrap();
        // Append an archived repo that must be filtered out.
        let mut archived = repos[0].clone();
        archived.name = "old-thing".to_string();
        archived.archived = true;
        repos.push(archived);

        let active = ReposBuilder::active_repos(repos, &[], now());
        assert_eq!(active.len(), 2, "archived repo is dropped");
        assert_eq!(active[0].name, "Portfolio", "API order is preserved");
        assert_eq!(active[1].name, "helm-charts");
        assert!(
            active.iter().all(|r| !r.archived),
            "no archived repos remain"
        );
    }

    #[test]
    fn active_repos_drops_blacklisted_repos_case_insensitively() {
        let repos: Vec<Repo> = serde_json::from_str(SAMPLE).unwrap();
        // "portfolio" differs in case from the "Portfolio" sample repo.
        let blacklist = vec!["portfolio".to_string()];

        let active = ReposBuilder::active_repos(repos, &blacklist, now());
        assert_eq!(active.len(), 1, "blacklisted repo is dropped");
        assert_eq!(active[0].name, "helm-charts");
    }

    #[test]
    fn active_repos_drops_stale_repos() {
        let mut repos: Vec<Repo> = serde_json::from_str(SAMPLE).unwrap();
        // Older than 365 days before `now` (2026-06-28) -> stale.
        let mut stale = repos[0].clone();
        stale.name = "ancient".to_string();
        stale.updated_at = "2024-01-01T00:00:00Z".to_string();
        repos.push(stale);
        // Exactly the cutoff (365 days back) is kept (>= comparison).
        let mut edge = repos[0].clone();
        edge.name = "edge".to_string();
        edge.updated_at = "2025-06-28T00:00:00Z".to_string();
        repos.push(edge);

        let active = ReposBuilder::active_repos(repos, &[], now());
        let names: Vec<&str> = active.iter().map(|r| r.name.as_str()).collect();
        assert!(!names.contains(&"ancient"), "stale repo is dropped");
        assert!(names.contains(&"edge"), "repo at the cutoff is kept");
        assert!(names.contains(&"Portfolio"));
        assert!(names.contains(&"helm-charts"));
    }

    #[test]
    fn repos_ignores_empty_names_and_preserves_order() {
        let builder = ReposBuilder::new("u").repos(["Portfolio", "", "helm-charts"]);
        assert_eq!(builder.repos, vec!["Portfolio", "helm-charts"]);
    }
}
