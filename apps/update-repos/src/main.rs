//! Generates `apps/web/repos.json` from the GitHub REST API at build time.
//!
//! Run before the `web` build (its `build.rs` embeds the result), this lists
//! every repository the user owns and keeps only the active ones (not archived,
//! not blacklisted via `CONFIG.blacklisted_repos` and updated within the last
//! year), maps each onto the shared
//! [`portfolio_data::Repo`]/[`portfolio_data::ReposFile`] models (the very same
//! types the web client embeds and the server's schema describes) and writes the
//! pretty-printed JSON to disk. A specific set of repositories can still be
//! requested explicitly via `GITHUB_REPOS`.
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
//! Environment:
//!   GITHUB_USERNAME  user whose repos to fetch (default: CONFIG.github_username)
//!   GITHUB_REPOS  comma-separated repo names to fetch (default: all
//!                 non-archived repositories of the user)
//!   GH_TOKEN / GITHUB_TOKEN  bearer token to authenticate (optional)
//!   CI  when set, uses the longer (10h) cache TTL instead of 60 minutes

mod builder;
mod cache;
mod error;

use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

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

    let user =
        env_non_empty("GITHUB_USERNAME").unwrap_or_else(|| CONFIG.github_username.to_string());
    let token = env_non_empty("GH_TOKEN").or_else(|| env_non_empty("GITHUB_TOKEN"));

    // An explicit, comma-separated override; when unset the builder lists every
    // non-archived repository the user owns.
    let names: Vec<String> = match env_non_empty("GITHUB_REPOS") {
        Some(list) => list
            .split(',')
            .map(|name| name.trim().to_string())
            .filter(|name| !name.is_empty())
            .collect(),
        None => Vec::new(),
    };

    let builder = ReposBuilder::new(user)
        .token(token)
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

/// Returns the value of `var` only when it is set and non-empty.
fn env_non_empty(var: &str) -> Option<String> {
    std::env::var(var).ok().filter(|v| !v.is_empty())
}
