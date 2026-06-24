//! Runtime data loaded relative to the site root.

use std::collections::BTreeMap;

use gloo_net::http::Request;
use portfolio_data::{Repo, ReposFile};
use serde::Deserialize;

/// Load state of `repos.json`, driving skeletons vs. content vs. offline UI.
#[derive(Clone, PartialEq)]
pub enum ReposState {
    Loading,
    Ready(ReposFile),
    Failed,
}

impl ReposState {
    /// Repos for consumers that only need the list (empty unless ready).
    pub fn repos(&self) -> Vec<Repo> {
        match self {
            ReposState::Ready(file) => file.repos.clone(),
            _ => Vec::new(),
        }
    }
}

/// Loads `repos.json`, generated daily by the `update-repos` workflow.
pub async fn load_repos() -> Result<ReposFile, String> {
    fetch_json("./repos.json").await
}

/// Checksums of the generated resume PDFs, written by the resume generator.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ResumeFingerprints {
    pub algorithm: String,
    pub generated_at: String,
    /// File name (e.g. "en.pdf") -> hex digest.
    pub files: BTreeMap<String, String>,
}

/// Loads `resume-fingerprint.json`; absent in dev builds without resumes.
pub async fn load_resume_fingerprints() -> Result<ResumeFingerprints, String> {
    fetch_json("./resume-fingerprint.json").await
}

async fn fetch_json<T: for<'de> Deserialize<'de>>(url: &str) -> Result<T, String> {
    let resp = Request::get(url).send().await.map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("{url}: HTTP {}", resp.status()));
    }
    resp.json::<T>().await.map_err(|e| e.to_string())
}
