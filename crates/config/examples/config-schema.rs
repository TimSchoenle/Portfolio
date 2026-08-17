//! Dump this workspace's configuration surface for the README generator.
//!
//! ```text
//! cargo run -p portfolio-config --features config-schema --example config-schema \
//!   -- --format markdown
//! ```
//!
//! `.github/workflows/update-files.yaml` runs exactly that, feeds the result to
//! `.github/templates/README.md.hbs` as the `configTable` variable, and commits the rendered
//! `README.md` back to the pull request. The table therefore cannot drift from the types: a key
//! renamed in `crates/config` is a key renamed in the README, in the same commit.
//!
//! Nothing here reads the environment, so the output is the same on a developer's machine and on
//! a runner where none of the variables it describes are set. That is what makes the render
//! deterministic enough for the action to skip the commit when nothing changed.
//!
//! # Why the root type is here and not in the crate
//!
//! `portfolio-config` deliberately owns *blocks* rather than one aggregate: each binary declares
//! the struct it actually reads, so a field is evidence that something consumes it. A root type
//! in the library would be a fourth aggregate nobody loads, and would quietly become the place
//! fields are added "so they appear somewhere".
//!
//! The documentation still needs one root, because the operator setting this workspace up reads
//! one file and one environment — not three. [`Documented`] is that root and only that: it exists
//! under the `config-schema` feature, it is never loaded, and it is the one place a new block has
//! to be listed to reach the README.

use std::process::ExitCode;

use portfolio_config::{AssetsConfig, CspConfig, GithubConfig, IsrConfig};
use serde::Serialize;
use terrace_config::schema::Describe;

/// Every block this workspace can be configured with, under the path an operator spells it at.
///
/// The union of what the SSR server reads (`assets`, `csp`, `isr`) and what the `update-repos`
/// builder reads (`github`). No binary loads this type; see the module docs.
#[derive(Default, Serialize, Describe)]
struct Documented {
    #[config(nested)]
    assets: AssetsConfig,
    #[config(nested)]
    csp: CspConfig,
    #[config(nested)]
    isr: IsrConfig,
    #[config(nested)]
    github: GithubConfig,
}

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
    let schema = portfolio_config::terrace()
        .schema::<Documented>()
        .with_defaults_from(&Documented::default())?
        .subset(&options.only);

    match options.format {
        Format::Json => schema.to_json(),
        Format::Markdown => Ok(schema.to_markdown()),
    }
}

/// What to emit, and how much of it.
struct Options {
    format: Format,
    /// The subtree to keep. Empty means the whole configuration.
    only: String,
}

/// Which rendering to emit.
enum Format {
    /// The versioned contract, for a consumer that renders its own tables.
    Json,
    /// GitHub-flavoured tables, which is what the README template interpolates.
    Markdown,
}

impl Options {
    /// JSON and everything, unless asked otherwise: those are the outputs that lose nothing.
    fn from_args() -> Result<Self, String> {
        let mut options = Self {
            format: Format::Json,
            only: String::new(),
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
                "--only" => {
                    options.only = args
                        .next()
                        .ok_or_else(|| format!("--only takes a key prefix; {USAGE}"))?;
                }
                other => return Err(format!("unknown argument `{other}`; {USAGE}")),
            }
        }
        Ok(options)
    }
}

const USAGE: &str = "usage: config-schema [--format json|markdown] [--only <key-prefix>]";
