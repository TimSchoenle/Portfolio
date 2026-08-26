//! What each binary in this workspace loads.
//!
//! The rest of this crate owns *blocks*. This module owns the two aggregates that say which
//! blocks a given binary reads — the mapping the crate docs used to state only as prose.
//!
//! They live here rather than in the binaries because the configuration reference in `README.md`
//! is generated from them. A generator cannot describe a private type in a crate it does not
//! depend on, so the alternative was a documentation root that *mirrored* each aggregate — and a
//! mirror can disagree with the thing it mirrors without anything failing, which is exactly the
//! drift generating the table exists to remove. Documenting the type the binary actually passes
//! to [`load`](crate::load) is the only version of this with no gap in it.
//!
//! A field is still evidence that something consumes it: each struct here is loaded by exactly
//! one binary, and nothing else constructs one.

use serde::Deserialize;

use crate::{AssetsConfig, CspConfig, GithubConfig, IsrConfig, SentryConfig};

/// What the SSR server (`apps/web`) loads.
///
/// Notably absent: the listen address. `IP`, `PORT` and `RUST_LOG` are the Dioxus toolchain's
/// contract with the binary — `dx serve` sets them to tell a development build which port it is
/// being proxied on — so folding them into the `PORTFOLIO_` namespace would take a name the
/// framework still reads and leave two sources of truth for one socket.
#[derive(Debug, Default, Deserialize)]
#[cfg_attr(
    feature = "config-schema",
    derive(serde::Serialize, terrace_config::schema::Describe)
)]
pub struct ServerConfig {
    /// Where the built client bundle is, for the readiness probe.
    #[cfg_attr(feature = "config-schema", config(nested))]
    #[serde(default)]
    pub assets: AssetsConfig,
    /// What the served `Content-Security-Policy` has to make room for.
    #[cfg_attr(feature = "config-schema", config(nested))]
    #[serde(default)]
    pub csp: CspConfig,
    /// Where rendered pages are cached, and for how long.
    #[cfg_attr(feature = "config-schema", config(nested))]
    #[serde(default)]
    pub isr: IsrConfig,
    /// Where errors and request traces are reported, if anywhere. Off unless configured.
    #[cfg_attr(feature = "config-schema", config(nested))]
    #[serde(default)]
    pub sentry: SentryConfig,
}

/// What the `update-repos` builder loads.
///
/// One block, so the aggregate looks redundant — it is not: it is what fixes the key path to
/// `github.*` rather than to the bare field names underneath.
///
/// Nothing the *server* reads belongs here, and that separation is load-bearing rather than
/// tidy: `github.token` is a build-time credential for a tool that lists repositories and exits
/// during the image build. A deployment never supplies it, and a reference that could not say so
/// would document a requirement that does not exist.
#[derive(Debug, Default, Deserialize)]
#[cfg_attr(
    feature = "config-schema",
    derive(serde::Serialize, terrace_config::schema::Describe)
)]
pub struct BuilderConfig {
    /// Credentials and scope for the build-time repository listing.
    #[cfg_attr(feature = "config-schema", config(nested))]
    #[serde(default)]
    pub github: GithubConfig,
}
