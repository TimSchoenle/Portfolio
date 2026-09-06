//! Where the SSR server finds the built client bundle.

use std::path::{Path, PathBuf};

use serde::Deserialize;

/// The built client assets the server reads.
///
/// Only the readiness probe consults this: the bundle is served by the Dioxus asset router,
/// which resolves `public/` relative to the binary on its own. What the probe answers is
/// whether that bundle is actually there, so a pod missing it is removed from the Service
/// endpoints instead of serving a page with no wasm.
// No `deny_unknown_fields` on any block in this crate, and it is not an oversight — though the
// reason it used to give was. The `PORTFOLIO_<KEY>_FILE` indirection is *not* the obstacle:
// terrace-config resolves the suffix into the key it names before anything is deserialised, so a
// closed struct loads through both the `_FILE` layer and the secrets directory. Two other
// reasons stand in its place:
//
//  1. **The two aggregates share one prefix.** `ServerConfig` and `BuilderConfig` are both
//     rooted at `PORTFOLIO_`, so a closed `ServerConfig` refuses `PORTFOLIO_GITHUB__TOKEN` —
//     the build-time credential `update-repos` reads — and any environment carrying both
//     would fail the server's boot on a key that is simply not its own.
//  2. **It would publish nothing.** `deny_unknown_fields` reaches the schema as
//     `additionalProperties: false` only for a type published through `#[config(element)]`,
//     and every struct here is `#[config(nested)]`, which contributes keys rather than an
//     object constraint. Closing one changes what the loader accepts at runtime and leaves the
//     generated contract byte-identical.
//
// Hardening the blocks against a misspelt key is still worth doing. It is a deliberate
// behaviour change and belongs in its own commit, not folded into a schema migration.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(
    feature = "config-schema",
    derive(serde::Serialize, terrace_config::schema::Describe)
)]
pub struct AssetsConfig {
    /// Directory holding the `dx bundle` output, relative to the working directory.
    ///
    /// The image runs from `/app`, next to `public/`, so the default resolves without anything
    /// being set.
    #[serde(default = "AssetsConfig::default_dist_dir")]
    pub dist_dir: PathBuf,
}

impl AssetsConfig {
    /// The sibling `public/` directory a `dx bundle` produces.
    fn default_dist_dir() -> PathBuf {
        PathBuf::from("public")
    }

    /// The configured bundle directory, falling back to the default when the value supplied was
    /// empty.
    ///
    /// Container platforms routinely inject `KEY=` for a declared-but-unset variable, and an
    /// empty path here would resolve the readiness probe against the working directory — which
    /// answers "ready" for a deploy that shipped no assets at all.
    #[must_use]
    pub fn dist_dir(&self) -> &Path {
        if self.dist_dir.as_os_str().is_empty() {
            Path::new("public")
        } else {
            &self.dist_dir
        }
    }
}

impl Default for AssetsConfig {
    fn default() -> Self {
        Self {
            dist_dir: Self::default_dist_dir(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AssetsConfig;
    use std::path::{Path, PathBuf};

    #[test]
    fn an_unset_bundle_directory_falls_back_to_public() {
        assert_eq!(AssetsConfig::default().dist_dir(), Path::new("public"));
    }

    /// `PORTFOLIO_ASSETS__DIST_DIR=` is how a container platform spells "declared but unset",
    /// and it must not resolve the probe against the working directory.
    #[test]
    fn an_empty_bundle_directory_is_treated_as_unset() {
        let config = AssetsConfig {
            dist_dir: PathBuf::new(),
        };
        assert_eq!(config.dist_dir(), Path::new("public"));
    }
}
