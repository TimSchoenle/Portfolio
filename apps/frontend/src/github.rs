//! Portfolio data embedded into the WASM binary at build time.
//!
//! `repos.json` and `resume-fingerprint.json` used to be fetched at runtime
//! relative to the site root; they are now compiled in, so the projects and
//! contact sections render from the first paint without an extra round-trip.

use portfolio_data::{Repo, ReposFile, ResumeFingerprints};

/// `repos.json`, embedded at compile time. Refreshed daily by the
/// `update-repos` workflow, which commits `apps/frontend/repos.json`.
const REPOS_JSON: &str = include_str!("../repos.json");

/// `resume-fingerprint.json`, embedded at compile time. `build.rs` writes the
/// generated manifest (or an empty default when no resumes were produced) into
/// `OUT_DIR`, so this include always resolves.
const RESUME_FINGERPRINT_JSON: &str =
    include_str!(concat!(env!("OUT_DIR"), "/resume-fingerprint.json"));

/// Availability of the embedded repo list, kept as an enum so the projects
/// section can still degrade gracefully if `repos.json` ever fails to parse.
#[derive(Clone, PartialEq)]
pub enum ReposState {
    Ready(ReposFile),
    Failed,
}

impl ReposState {
    /// Repos for consumers that only need the list (empty unless ready).
    pub fn repos(&self) -> Vec<Repo> {
        match self {
            ReposState::Ready(file) => file.repos.clone(),
            ReposState::Failed => Vec::new(),
        }
    }
}

/// Parses the embedded `repos.json` (generated daily by the `update-repos`
/// workflow and committed to the repository).
pub fn load_repos() -> ReposState {
    match serde_json::from_str::<ReposFile>(REPOS_JSON) {
        Ok(file) => ReposState::Ready(file),
        Err(e) => {
            web_sys::console::warn_1(&format!("repos.json parse failed: {e}").into());
            ReposState::Failed
        }
    }
}

/// Parses the embedded `resume-fingerprint.json`; `None` when no resumes were
/// generated (the embedded manifest is empty) or the manifest is malformed.
pub fn load_resume_fingerprints() -> Option<ResumeFingerprints> {
    let parsed = serde_json::from_str::<ResumeFingerprints>(RESUME_FINGERPRINT_JSON).ok()?;
    if parsed.is_empty() {
        None
    } else {
        Some(parsed)
    }
}
