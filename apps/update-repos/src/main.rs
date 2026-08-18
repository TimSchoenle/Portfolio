//! Generates `apps/web/repos.json` from the GitHub REST API at build time.
//!
//! Run before the `web` build (its `build.rs` embeds the result), this lists
//! every repository the user owns and keeps only the active ones (not archived,
//! not blacklisted via `CONFIG.blacklisted_repos` and updated within the last
//! year), maps each onto the shared
//! [`portfolio_data::Repo`]/[`portfolio_data::ReposFile`] models (the very same
//! types the web client embeds and the server's schema describes) and writes the
//! pretty-printed JSON to disk. A specific set of repositories can still be
//! requested explicitly via `github.repos`.
//!
//! To avoid hitting the GitHub API on every rebuild (and its rate limits), the
//! existing output is reused while it is still fresh: the network fetch is
//! skipped when `repos.json`'s `generated_at` is within the cache TTL — 10 hours
//! on CI, 60 minutes otherwise (see [`cache`]). When the cache is stale or
//! missing the fetch runs and the build fails if it cannot be produced.
//!
//! Usage:
//! ```text
//! update-repos [OUTPUT_PATH]      # default: apps/web/repos.json
//! ```
//!
//! Configuration is layered (see `portfolio_config`); the keys this binary reads
//! are the `github` block of [`BuilderConfig`]:
//!
//! ```text
//! PORTFOLIO_GITHUB__USERNAME    user whose repos to fetch
//!                               (default: CONFIG.github_username)
//! PORTFOLIO_GITHUB__REPOS       repo names to fetch, e.g. [Portfolio,actions]
//!                               (default: all active repositories of the user)
//! PORTFOLIO_GITHUB__TOKEN       bearer token lifting the API rate limit
//!                               (optional; prefer ..._TOKEN_FILE)
//! PORTFOLIO_GITHUB__TOKEN_FILE  path to a file holding the token, so it never
//!                               enters the process environment
//! ```
//!
//! `CI` is read separately and deliberately stays outside that namespace: it is
//! the CI provider's variable, not ours, and it selects the cache TTL rather
//! than configuring the fetch.

mod builder;
mod cache;
mod error;

use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use portfolio_config::BuilderConfig;
use portfolio_data::CONFIG;
use time::OffsetDateTime;

use crate::builder::ReposBuilder;
use crate::error::UpdateReposError;

/// Where `repos.json` is committed, relative to the workspace root.
const DEFAULT_OUTPUT: &str = "apps/web/repos.json";

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("update-repos: {err}");
            // Only for the failure the report is about. The token this tool carries is the one
            // secret in the workspace and arrives as a mounted file, so "which layer supplied
            // `github.token`" is the question a failed build asks — and the report answers it
            // without printing the value, which is why it can be in CI output at all.
            if matches!(err, UpdateReposError::Config(_)) {
                eprintln!("{}", portfolio_config::provenance());
            }
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), UpdateReposError> {
    let output = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_OUTPUT));

    // Reuse a still-fresh cache to keep rebuilds (and CI) off the GitHub API.
    let ttl = cache::ttl_for_env();
    if cache::is_cached_fresh(&output, OffsetDateTime::now_utc(), ttl) {
        println!(
            "update-repos: {} is still fresh (within {} min); skipping fetch",
            output.display(),
            ttl.whole_minutes()
        );
        return Ok(());
    }

    let github = config()?.github;
    // The site's own identity is the only sensible default, and it lives in the
    // compile-time data rather than in the config schema — a deployment that has
    // to name the user it is a portfolio *for* has bigger problems.
    let user = github
        .username()
        .unwrap_or(CONFIG.github_username)
        .to_owned();

    // An explicit override; when empty the builder lists every active repository
    // the user owns. Collected before the token moves out of `github`.
    let names: Vec<String> = github.repos().map(str::to_owned).collect();

    let builder = ReposBuilder::new(user)
        .token(github.into_token())
        .repos(names)
        .blacklist(CONFIG.blacklisted_repos.iter().map(|name| name.to_string()));

    let repos = builder.fetch()?;

    if let Some(parent) = output.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(&repos)?;
    fs::write(&output, format!("{json}\n"))?;

    println!(
        "update-repos: wrote {} repos for {} to {}",
        repos.repos.len(),
        repos.user,
        output.display()
    );
    Ok(())
}

/// Reads the layered configuration.
///
/// A configuration error aborts the build rather than falling back to an
/// unauthenticated fetch: silently dropping the token would turn a typo in a
/// secret's path into an intermittent rate-limit failure much later, in a job
/// that has nothing to do with the mistake.
fn config() -> Result<BuilderConfig, UpdateReposError> {
    portfolio_config::load().map_err(UpdateReposError::from)
}
