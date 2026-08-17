//! Portfolio data embedded into the binary at build time.
//!
//! `repos.json` and `resume-fingerprint.json` are compiled in (see `build.rs`),
//! so the projects and contact sections render from the first paint — including
//! the server-side render — without an extra round-trip.

use std::sync::{Arc, LazyLock};

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
///
/// The parsed document is shared rather than owned: this value lives in a Dioxus
/// context and is cloned by everything that reads it, while the list behind it
/// is a compile-time constant nothing mutates. `Arc` rather than `Rc` so the
/// same type can back the process-wide [`REPOS`] below on the server, where the
/// renderer is not confined to one thread.
#[derive(Clone, PartialEq)]
pub enum ReposState {
    Ready(Arc<ReposFile>),
    Failed,
}

impl ReposState {
    /// The repositories, borrowed. Empty unless ready.
    ///
    /// Borrowed rather than cloned: the projects section and the command palette
    /// read this on every render, and giving each of them its own deep copy of
    /// the whole list was the largest allocation in a re-render.
    pub fn repos(&self) -> &[Repo] {
        match self {
            ReposState::Ready(file) => &file.repos,
            ReposState::Failed => &[],
        }
    }
}

/// The parsed `repos.json`, shared by every render in this process.
///
/// Parsing a compile-time constant a second time answers the same question
/// twice. That mattered most on the server, where `App` mounts once per render:
/// every incremental-cache miss re-parsed the entire document.
static REPOS: LazyLock<ReposState> =
    LazyLock::new(|| match serde_json::from_str::<ReposFile>(REPOS_JSON) {
        Ok(file) => ReposState::Ready(Arc::new(file)),
        Err(_e) => {
            #[cfg(feature = "web")]
            web_sys::console::warn_1(&format!("repos.json parse failed: {_e}").into());
            ReposState::Failed
        }
    });

/// The embedded repo list. Resolves identically on the server (SSR) and the wasm
/// client, so the rendered project list matches across hydration.
pub fn load_repos() -> ReposState {
    REPOS.clone()
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
