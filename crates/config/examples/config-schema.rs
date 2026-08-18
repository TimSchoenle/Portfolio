//! Dump this workspace's configuration surface for the README generator.
//!
//! ```text
//! cargo run -p portfolio-config --features config-schema --example config-schema \
//!   -- --format markdown --scope server
//! ```
//!
//! `.github/workflows/update-files.yaml` runs it once per scope, feeds the results to
//! `.github/templates/README.md.hbs`, and commits the rendered `README.md` back to the pull
//! request. The tables therefore cannot drift from the types: a key renamed in `crates/config`
//! is a key renamed in the README, in the same commit.
//!
//! Nothing here reads the environment, so the output is the same on a developer's machine and on
//! a runner where none of the variables it describes are set. That is what makes the render
//! deterministic enough for the action to skip the commit when nothing changed.
//!
//! # Why there are scopes
//!
//! One flat table of every key says that a deployment needs a GitHub token. It does not.
//! `github.*` belongs to `update-repos`, a build-time tool that lists repositories and exits
//! during the image build; the SSR server never loads it and never sees it. A reference that
//! cannot express which binary reads a key documents an operational requirement that is not real,
//! so the roots below are split the way the binaries are.
//!
//! # No type is declared here at all
//!
//! Every scope describes an aggregate the binaries actually pass to `portfolio_config::load` —
//! [`ServerConfig`] or [`BuilderConfig`], which live in `crates/config` for exactly this reason.
//! Nothing is mirrored, so nothing can drift: a block added to what a binary loads is a block in
//! the README, and there is no second list to remember.
//!
//! `all` is `Schema::merge` of the two rather than a union type, so even the whole-workspace
//! document is built out of what the binaries load. A key both halves described would be kept
//! once and a key described *differently* by each would panic, which is the right answer for a
//! reference that has to have one.

use std::process::ExitCode;

use portfolio_config::{BuilderConfig, ServerConfig};
use terrace_config::schema::{Column, Schema};

fn main() -> ExitCode {
    let options = match Options::from_args() {
        Ok(options) => options,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::FAILURE;
        }
    };

    match render(&options) {
        Ok(rendered) => {
            println!("{rendered}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

/// The schema, rendered as `options` asks for it.
///
/// Built through [`portfolio_config::terrace`] rather than a bare dialect, so the two variables
/// the loader itself reads — `PORTFOLIO_CONFIG` and `PORTFOLIO_SECRETS_DIR` — are reported with
/// the names *this* workspace configures rather than the ones a default prefix would derive.
///
/// The `Default` column comes from a value built here, not from the process environment: the
/// documentation job runs where none of these variables exist, and that is the point.
fn render(options: &Options) -> Result<String, portfolio_config::ConfigError> {
    let schema = match options.scope {
        Scope::All => server()?.merge(builder()?),
        Scope::Server => server()?,
        Scope::Builder => builder()?,
    };

    match options.format {
        Format::Json => schema.to_json(),
        // The loader variables lead the server table and are absent from every other one. They
        // apply to both binaries, so a page repeating them above each scope would say the same
        // thing twice; the server scope is where an operator setting a deployment up will be.
        Format::Markdown => Ok(match options.scope {
            Scope::All | Scope::Server => schema.to_markdown(),
            Scope::Builder => schema.to_markdown_keys(Column::DEFAULT),
        }),
    }
}

/// The keys the SSR server loads, with the defaults it starts from.
fn server() -> Result<Schema, portfolio_config::ConfigError> {
    portfolio_config::terrace()
        .schema::<ServerConfig>()
        .with_defaults_from(&ServerConfig::default())
}

/// The keys the `update-repos` builder loads, with the defaults it starts from.
fn builder() -> Result<Schema, portfolio_config::ConfigError> {
    portfolio_config::terrace()
        .schema::<BuilderConfig>()
        .with_defaults_from(&BuilderConfig::default())
}

/// What to emit, and how much of it.
struct Options {
    format: Format,
    scope: Scope,
}

/// Which rendering to emit.
enum Format {
    /// The versioned contract, for a consumer that renders its own tables.
    Json,
    /// GitHub-flavoured tables, which is what the README template interpolates.
    Markdown,
}

/// Whose configuration to report.
enum Scope {
    /// Every key in the workspace, which is what a machine-readable contract wants.
    All,
    /// The keys the SSR server loads.
    Server,
    /// The keys the `update-repos` builder loads.
    Builder,
}

impl Options {
    /// Everything, as JSON: the output that loses nothing.
    fn from_args() -> Result<Self, String> {
        let mut options = Self {
            format: Format::Json,
            scope: Scope::All,
        };
        let mut args = std::env::args().skip(1);
        while let Some(flag) = args.next() {
            match flag.as_str() {
                "--format" => {
                    options.format = match args.next().as_deref() {
                        Some("json") => Format::Json,
                        Some("markdown" | "md") => Format::Markdown,
                        Some(other) => return Err(format!("unknown format `{other}`; {USAGE}")),
                        None => return Err(format!("--format takes a value; {USAGE}")),
                    };
                }
                "--scope" => {
                    options.scope = match args.next().as_deref() {
                        Some("all") => Scope::All,
                        Some("server") => Scope::Server,
                        Some("builder") => Scope::Builder,
                        Some(other) => return Err(format!("unknown scope `{other}`; {USAGE}")),
                        None => return Err(format!("--scope takes a value; {USAGE}")),
                    };
                }
                other => return Err(format!("unknown argument `{other}`; {USAGE}")),
            }
        }
        Ok(options)
    }
}

const USAGE: &str = "usage: config-schema [--format json|markdown] [--scope all|server|builder]";
