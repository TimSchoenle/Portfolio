//! The typed configuration surface every binary in this workspace reads, plus the Portfolio
//! dialect of the layered loader.
//!
//! The layering is [`terrace_config`]'s. Lowest precedence first: struct defaults, TOML at
//! `$PORTFOLIO_CONFIG` (a file, or every `*.toml` in it when it names a directory),
//! `PORTFOLIO_`-prefixed `__`-nested environment variables, `$PORTFOLIO_SECRETS_DIR`, and
//! `PORTFOLIO_<KEY>_FILE` indirection. The last three are mutually exclusive per key: a key
//! supplied by two of them is refused at boot rather than resolved by precedence, because a
//! stale environment variable shadowing a rotated mounted secret keeps the process running on
//! the old credential.
//!
//! Call [`load`], which is the only entry point, and [`provenance`] when it fails.
//!
//! # What each binary composes
//!
//! This crate owns the *blocks*, the dialect, and one aggregate per binary naming the blocks that
//! binary reads — so a struct field is evidence that something consumes it:
//!
//! - the SSR server, [`ServerConfig`] ([`AssetsConfig`], [`CspConfig`], [`IsrConfig`]),
//! - the `update-repos` builder, [`BuilderConfig`] ([`GithubConfig`]).
//!
//! The aggregates live here rather than in the binaries so the generated configuration reference
//! can describe the types those binaries actually load; see [`aggregates`].
//!
//! # Why the loader half only
//!
//! Two of `terrace-config`'s five features are taken: `loader`, which is the layering above, and
//! `explain`, which is [`provenance`] — the report naming which layer supplied each key, printed
//! beside the error when a boot is refused. `explain` costs no dependency at all, and it is what
//! answers the question the error cannot: an operator inside a distroless image with no shell
//! cannot otherwise see that the variable they thought they removed is still shadowing the
//! `Secret` they mounted.
//!
//! `reload` is the one deliberately left on the table. It is a supervisor that rebuilds a
//! running service when the files its configuration came from change, and this workspace does
//! not take it, for two reasons that both have to hold before it would be worth the
//! tokio/notify/`inotify` weight:
//!
//! 1. **The server holds no secrets.** Rotation is what the supervisor exists to survive, and
//!    the only secret in the workspace ([`GithubConfig::token`]) belongs to a one-shot build
//!    tool that exits in seconds. Every value the *server* reads changes only on a redeploy.
//! 2. **The serve loop is not ours to cancel.** `dioxus::serve` owns the listener and the accept
//!    loop, and offers no shutdown handle, so a rebuild could not stop the previous generation
//!    before the replacement binds the same address. Taking the supervisor would mean
//!    reimplementing the framework's serve loop — including the devtools hot-patch path that
//!    `dx serve` depends on in development — to gain a reload for configuration that a
//!    redeploy already replaces.
//!
//! Both halves of that reasoning are recorded here rather than in a commit message because the
//! second one changes the moment Dioxus grows a cancellable `serve`.
//!
//! The remaining two are development-time only: `schema` behind this crate's own off-by-default
//! `config-schema` feature (see below), and `testing` as a dev-dependency, which is what the
//! loader tests in `src/loader.rs` arrange their mounts through.
//!
//! # Blank means unset
//!
//! Every accessor in this crate treats an empty or whitespace-only value as though the key had
//! not been supplied. Container platforms routinely inject `KEY=` for a declared-but-unset
//! variable, and each block documents what the alternative reading would have cost.
//!
//! # The configuration reference in `README.md` is generated from these types
//!
//! Every block and aggregate above derives `terrace_config::schema::Describe` under the
//! off-by-default `config-schema` feature, and `examples/config-schema.rs` walks them into the
//! tables CI renders into the README — one per aggregate, so the reference says which binary
//! reads a key rather than implying every deployment needs all of them. The feature is off in
//! every build that ships, so `serde_json` and the derive never reach a binary;
//! `cargo clippy --all-features --all-targets` is what keeps the generator compiling.
//!
//! Write field documentation as rustdoc asks for it: a summary sentence, a blank line, then as
//! much reasoning as it takes. Only the summary reaches the Markdown table — `to_json` carries
//! the whole comment for anything that wants the rest — so nothing has to be kept short for the
//! README's sake, and nothing has to be annotated twice.
//!
//! A new block needs no registration anywhere — adding it to the aggregate that loads it is what
//! puts it in the README, because that aggregate is what the generator describes.

mod aggregates;
mod assets;
mod csp;
mod github;
mod isr;
mod loader;

pub use aggregates::{BuilderConfig, ServerConfig};
pub use assets::AssetsConfig;
pub use csp::{CloudflareConfig, CspConfig, CspConfigError};
pub use github::GithubConfig;
pub use isr::IsrConfig;
pub use loader::{ConfigError, load, provenance, terrace};
