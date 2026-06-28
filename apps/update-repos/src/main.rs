//! Refreshes `apps/frontend/repos.json` from the GitHub REST API.
//!
//! This is the Rust replacement for the former inline Python step in
//! `.github/workflows/update-repos.yml`. It fetches the explicit set of
//! repositories declared in [`portfolio_data::CONFIG::repos`] (one request per
//! repo), maps each one onto the shared
//! [`portfolio_data::Repo`]/[`portfolio_data::ReposFile`] models (the very same
//! types the frontend embeds and the server's schema describes) and writes the
//! pretty-printed JSON to disk.
//!
//! Usage:
//! ```text
//! update-repos [OUTPUT_PATH]      # default: apps/frontend/repos.json
//! ```
//!
//! Environment:
//!   GITHUB_USERNAME  user whose repos to fetch (default: CONFIG.github_username)
//!   GITHUB_REPOS  comma-separated repo names to fetch (default: CONFIG.repos)
//!   GH_TOKEN / GITHUB_TOKEN  bearer token to authenticate (optional)

mod builder;
mod error;

use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use portfolio_data::CONFIG;

use crate::builder::ReposBuilder;
use crate::error::UpdateReposError;

/// Where `repos.json` is committed, relative to the workspace root.
const DEFAULT_OUTPUT: &str = "apps/frontend/repos.json";

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

    let user =
        env_non_empty("GITHUB_USERNAME").unwrap_or_else(|| CONFIG.github_username.to_string());
    let token = env_non_empty("GH_TOKEN").or_else(|| env_non_empty("GITHUB_TOKEN"));

    let names: Vec<String> = match env_non_empty("GITHUB_REPOS") {
        Some(list) => list
            .split(',')
            .map(|name| name.trim().to_string())
            .filter(|name| !name.is_empty())
            .collect(),
        None => CONFIG.repos.iter().map(|name| name.to_string()).collect(),
    };

    let builder = ReposBuilder::new(user).token(token).repos(names);

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
