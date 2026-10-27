//! The repository list behind the projects section and the command palette.

use std::sync::{Arc, LazyLock};

use portfolio_data::{Repo, ReposFile};

/// `repos.json`, embedded at compile time. `build.rs` copies it into `OUT_DIR`
/// (or writes an empty default when absent), so this include always resolves.
const REPOS_JSON: &str = include_str!(concat!(env!("OUT_DIR"), "/repos.json"));

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
static REPOS: LazyLock<ReposState> = LazyLock::new(|| parse(REPOS_JSON));

/// Parses a repository list; [`ReposState::Failed`] when it does not parse.
fn parse(json: &str) -> ReposState {
    match serde_json::from_str::<ReposFile>(json) {
        Ok(file) => ReposState::Ready(Arc::new(file)),
        Err(err) => {
            crate::console::warn(format_args!("repos.json parse failed: {err}"));
            ReposState::Failed
        }
    }
}

/// The embedded repo list. Resolves identically on the server (SSR) and the wasm
/// client, so the rendered project list matches across hydration.
pub fn load_repos() -> ReposState {
    REPOS.clone()
}

#[cfg(test)]
mod tests {
    use super::{ReposState, parse};

    #[test]
    fn the_empty_default_is_ready_and_empty() {
        let state = parse(r#"{"generated_at":"","user":"","repos":[]}"#);
        assert!(matches!(state, ReposState::Ready(_)));
        assert_eq!(state.repos(), []);
    }

    #[test]
    fn a_malformed_list_fails_with_no_repos() {
        let state = parse("{");
        assert!(matches!(state, ReposState::Failed));
        assert_eq!(state.repos(), []);
    }
}
