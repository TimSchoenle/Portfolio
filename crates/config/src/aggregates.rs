//! What each binary in this workspace loads.
//!
//! The rest of this crate owns *blocks*. This module owns the two aggregates that say which
//! blocks a given binary reads.
//!
//! They live here, not in the binaries, because the configuration reference in `README.md` is
//! generated from them. A generator cannot describe a private type in a crate it does not depend
//! on, so the alternative was a documentation root that mirrored each aggregate, and a mirror can
//! disagree with the thing it mirrors without anything failing. Documenting the type the binary
//! passes to [`load`](crate::load) leaves no such gap.
//!
//! A field is still evidence that something consumes it: each struct here is loaded by exactly
//! one binary, and nothing else constructs one.

use serde::Deserialize;
use terrace_legal::LegalConfig;

use crate::{AssetsConfig, CspConfig, GithubConfig, HstsConfig, IsrConfig, SentryConfig};

/// The configuration the SSR server in `apps/web` loads.
///
/// The listen address is not part of it. `IP`, `PORT` and `RUST_LOG` are the Dioxus toolchain's
/// contract with the binary, and `dx serve` sets them to tell a development build which port it
/// is proxied on.
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
    /// The `Strict-Transport-Security` header: its lifetime and how far it reaches.
    #[cfg_attr(feature = "config-schema", config(nested))]
    #[serde(default)]
    pub hsts: HstsConfig,
    /// Where rendered pages are cached, and for how long.
    #[cfg_attr(feature = "config-schema", config(nested))]
    #[serde(default)]
    pub isr: IsrConfig,
    /// Where errors and request traces are reported, if anywhere. Off unless configured.
    #[cfg_attr(feature = "config-schema", config(nested))]
    #[serde(default)]
    pub sentry: SentryConfig,
    /// The legal documents the site publishes: imprint, privacy notice and any others.
    ///
    /// Required: [`legal_catalog_builder`](crate::legal_catalog_builder) refuses a configuration
    /// without an imprint and a privacy notice in every language the site renders, so the
    /// default is a deployment that does not start. Each document's text is Markdown and is
    /// usually supplied as a mounted TOML fragment or through `_FILE` indirection.
    #[cfg_attr(feature = "config-schema", config(nested))]
    #[serde(default)]
    pub legal: LegalConfig,
}

/// The configuration the `portfolio-update-repos` builder loads.
///
/// It holds one block, and it is what roots that block's keys at `github.*`. Nothing the server
/// reads belongs here: `github.token` is a credential for a tool that runs during the image build,
/// and a deployment never supplies it.
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
