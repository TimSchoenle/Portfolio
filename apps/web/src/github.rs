//! Portfolio data embedded into the binary at build time.
//!
//! `repos.json` and `resume-fingerprint.json` are compiled in (see `build.rs`),
//! so the projects and contact sections render from the first paint — including
//! the server-side render — without an extra round-trip.

use portfolio_data::{Repo, ReposFile, ResumeFingerprints};

/// `repos.json`, embedded at compile time. `build.rs` copies it into `OUT_DIR`
/// (or writes an empty default when absent), so this include always resolves.
const REPOS_JSON: &str = include_str!(concat!(env!("OUT_DIR"), "/repos.json"));

/// `resume-fingerprint.json`, embedded at compile time (empty default when no
/// resumes were produced).
const RESUME_FINGERPRINT_JSON: &str =
    include_str!(concat!(env!("OUT_DIR"), "/resume-fingerprint.json"));

/// Availability of the embedded repo list, kept as an enum so the projects
/// section can degrade gracefully if `repos.json` ever fails to parse.
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

/// Parses the embedded `repos.json`. Runs identically on the server (SSR) and
/// the wasm client, so the rendered project list matches across hydration.
pub fn load_repos() -> ReposState {
    match serde_json::from_str::<ReposFile>(REPOS_JSON) {
        Ok(file) => ReposState::Ready(file),
        Err(_e) => {
            #[cfg(feature = "web")]
            web_sys::console::warn_1(&format!("repos.json parse failed: {_e}").into());
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
