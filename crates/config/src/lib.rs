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
//! Call [`load`], which is the only entry point.
//!
//! # What each binary composes
//!
//! This crate owns the *blocks* and the dialect; each binary declares the aggregate it actually
//! reads, so a struct field is evidence that something consumes it:
//!
//! - the SSR server ([`AssetsConfig`], [`CspConfig`], [`IsrConfig`]),
//! - the `update-repos` builder ([`GithubConfig`]).
//!
//! # Why the loader half only
//!
//! `terrace-config` also ships a supervisor that rebuilds a running service when the files its
//! configuration came from change. This workspace deliberately does not take it, for two
//! reasons that both have to hold before it would be worth the tokio/notify/`inotify` weight:
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
//! # Blank means unset
//!
//! Every accessor in this crate treats an empty or whitespace-only value as though the key had
//! not been supplied. Container platforms routinely inject `KEY=` for a declared-but-unset
//! variable, and each block documents what the alternative reading would have cost.
//!
//! # The configuration reference in `README.md` is generated from these types
//!
//! Every block above derives `terrace_config::schema::Describe` under the off-by-default
//! `config-schema` feature, and `examples/config-schema.rs` walks it into the table CI renders
//! into the README. The feature is off in every build that ships, so `serde_json` and the derive
//! never reach a binary; `cargo clippy --all-features --all-targets` is what keeps the generator
//! compiling.
//!
//! Two consequences for anyone editing this crate:
//!
//! - **A field's `///` comment is one summary sentence on one line.** It is copied verbatim into
//!   a Markdown table cell, where a second paragraph becomes a `<br>`. Longer reasoning goes in
//!   `//` comments above the field, which is why the blocks below read the way they do.
//! - **A new block is not documented until it is added to the example's aggregate.** This crate
//!   deliberately has no root config type — each binary declares the aggregate it reads — so the
//!   generator declares one of its own, and nothing but that file can notice a block missing
//!   from it.

mod assets;
mod csp;
mod github;
mod isr;
mod loader;

pub use assets::AssetsConfig;
pub use csp::{CloudflareConfig, CspConfig, CspConfigError};
pub use github::GithubConfig;
pub use isr::IsrConfig;
pub use loader::{ConfigError, load, terrace};
