//! Dump this workspace's configuration surface for the documents CI renders from it.
//!
//! ```text
//! cargo run -p portfolio-config --features config-schema --example config-schema \
//!   -- --format markdown --scope server
//! ```
//!
//! `.github/workflows/update-files.yaml` runs it once, feeds the result to the templates under
//! `.github/templates/`, and commits the rendered `README.md` and `config.example.toml` back to
//! the pull request. Neither document can drift from the types: a key renamed in `crates/config`
//! is a key renamed in both, in the same commit.
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
//!
//! # Why the payload is assembled here and not in YAML
//!
//! [`Format::Variables`] emits the whole render payload — both tables, the example file, and the
//! spellings the prose in `README.md.hbs` names — as one strict-JSON object. The workflow step
//! that used to run this binary once per scope and stitch the results together with `jq` now
//! runs it once and forwards what it printed.
//!
//! That is not only shorter. The names the templates read are declared beside the schema that
//! produces them, so widening what the documents interpolate is a change to this file rather
//! than to a shell snippet embedded in YAML — and [`KEYS`] turns a key the prose names and the
//! types no longer have into a failed CI step instead of a README with a blank in it.

use std::error::Error;
use std::process::ExitCode;

use portfolio_config::{BuilderConfig, ServerConfig};
use serde_json::{Map, Value, json};
use terrace_config::schema::{Column, Docs, Key, Schema, TomlExample};

/// The keys `README.md.hbs` names in prose, under the names it reads them by.
///
/// Prose cannot be generated from the types the way a table can, but the *spellings* inside it
/// can: `PORTFOLIO_GITHUB__TOKEN_FILE` is derived rather than typed, so a renamed field renames
/// it in every sentence that mentions it.
///
/// This list is the one thing here that is not derived, and it earns that: a path no key has is
/// [refused](variables) rather than rendered empty, which makes a rename that outruns the prose
/// a failed CI step. Adding to it is what lets the template stop hard-coding a spelling.
const KEYS: &[(&str, &str)] = &[
    ("assetsDistDir", "assets.dist_dir"),
    ("cspHashInlineScripts", "csp.hash_inline_scripts"),
    ("cspScriptNonce", "csp.cloudflare.script_nonce"),
    ("githubRepos", "github.repos"),
    ("githubToken", "github.token"),
];

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
fn render(options: &Options) -> Result<String, Box<dyn Error>> {
    if options.format == Format::Variables {
        return variables();
    }

    let schema = match options.scope {
        Scope::All => all()?,
        Scope::Server => server()?,
        Scope::Builder => builder()?,
    };

    match options.format {
        Format::Json => Ok(schema.to_json()?),
        // The loader variables lead the server table and are absent from every other one. They
        // apply to both binaries, so a page repeating them above each scope would say the same
        // thing twice; the server scope is where an operator setting a deployment up will be.
        Format::Markdown => Ok(match options.scope {
            Scope::All | Scope::Server => schema.to_markdown(),
            Scope::Builder => schema.to_markdown_keys(Column::DEFAULT),
        }),
        Format::Toml => Ok(example_config(&schema)),
        Format::Variables => unreachable!("returned above"),
    }
}

/// The whole render payload, as the strict JSON the template action takes.
///
/// One object for both templates. An unread name costs a template nothing — strict mode fails on
/// a reference the payload does not *define*, never on a definition nothing reads — so the
/// alternative, a payload per template, would only be two ways to get the same schema out.
///
/// # Errors
/// If a path in [`KEYS`] names no key in the merged schema, which is a rename the prose in
/// `README.md.hbs` has not caught up with.
fn variables() -> Result<String, Box<dyn Error>> {
    let all = all()?;

    let mut keys = Map::new();
    for (name, path) in KEYS {
        let key = all
            .keys
            .iter()
            .find(|key| key.path == *path)
            .ok_or_else(|| format!("`{path}` is named in README.md.hbs but no key has it"))?;
        keys.insert((*name).to_owned(), spellings(key));
    }

    // Trimmed, all three of them. Each rendering ends with a newline of its own, and every
    // template that interpolates one follows the mustache with a newline too — so an untrimmed
    // value is a blank line under every table and a blank line at the end of the example file.
    // Trailing whitespace is not content in any of the three, and this is the one place it can
    // be dropped for all of them.
    let payload = json!({
        "serverConfigTable": server()?.to_markdown().trim_end(),
        "builderConfigTable": builder()?.to_markdown_keys(Column::DEFAULT).trim_end(),
        "exampleConfig": example_config(&all).trim_end(),
        "loader": {
            "envPrefix": all.dialect.prefix,
            "envNesting": all.dialect.nesting_separator,
            "fileSuffix": all.dialect.indirection_suffix,
            "configVar": portfolio_config::terrace().config_var_name(),
            "secretsDirVar": portfolio_config::terrace().secrets_dir_var_name(),
        },
        "keys": keys,
    });
    Ok(serde_json::to_string(&payload)?)
}

/// Every way one key can be supplied, and what it is when nothing does.
fn spellings(key: &Key) -> Value {
    json!({
        "path": key.path,
        "env": key.env,
        "envFile": key.env_file,
        "secretsFile": key.secrets_file,
        "default": key.default,
    })
}

/// The body of `config.example.toml`: every key, commented out at its default.
///
/// No preamble, because the template supplies one — what this file is and how to point the
/// loader at it are facts about the repository rather than about the schema, and the layering
/// itself is documented once, in `README.md`. What is left is what only the types know.
///
/// [`Docs::Full`] rather than the default summary: this is the file an operator edits with no
/// rustdoc open beside it, and every paragraph the fields carry is a paragraph the hand-written
/// version of this file used to carry too.
fn example_config(schema: &Schema) -> String {
    schema.to_toml_example_with(&TomlExample::new().header(false).docs(Docs::Full))
}

/// Every key in the workspace, which is what a machine-readable contract and the example file
/// both want.
fn all() -> Result<Schema, portfolio_config::ConfigError> {
    Ok(server()?.merge(builder()?))
}

/// The keys the SSR server loads, with the defaults it starts from.
///
/// Built through [`portfolio_config::terrace`] rather than a bare dialect, so the two variables
/// the loader itself reads — `PORTFOLIO_CONFIG` and `PORTFOLIO_SECRETS_DIR` — are reported with
/// the names *this* workspace configures rather than the ones a default prefix would derive.
///
/// The `Default` column comes from a value built here, not from the process environment: the
/// documentation job runs where none of these variables exist, and that is the point.
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
#[derive(PartialEq, Eq, Clone, Copy)]
enum Format {
    /// The versioned contract, for a consumer that renders its own tables.
    Json,
    /// GitHub-flavoured tables, which is what the README template interpolates.
    Markdown,
    /// A commented `config.toml` holding every key at its default.
    Toml,
    /// Everything the templates interpolate, as one strict-JSON object.
    Variables,
}

/// Whose configuration to report.
#[derive(PartialEq, Eq, Clone, Copy)]
enum Scope {
    /// Every key in the workspace.
    All,
    /// The keys the SSR server loads.
    Server,
    /// The keys the `update-repos` builder loads.
    Builder,
}

impl Options {
    /// Everything, as JSON: the output that loses nothing.
    fn from_args() -> Result<Self, String> {
        let mut format = Format::Json;
        let mut scope = None;
        let mut args = std::env::args().skip(1);
        while let Some(flag) = args.next() {
            match flag.as_str() {
                "--format" => {
                    format = match args.next().as_deref() {
                        Some("json") => Format::Json,
                        Some("markdown" | "md") => Format::Markdown,
                        Some("toml") => Format::Toml,
                        Some("variables") => Format::Variables,
                        Some(other) => return Err(format!("unknown format `{other}`; {USAGE}")),
                        None => return Err(format!("--format takes a value; {USAGE}")),
                    };
                }
                "--scope" => {
                    scope = match args.next().as_deref() {
                        Some("all") => Some(Scope::All),
                        Some("server") => Some(Scope::Server),
                        Some("builder") => Some(Scope::Builder),
                        Some(other) => return Err(format!("unknown scope `{other}`; {USAGE}")),
                        None => return Err(format!("--scope takes a value; {USAGE}")),
                    };
                }
                other => return Err(format!("unknown argument `{other}`; {USAGE}")),
            }
        }

        // Refused rather than ignored: the payload carries a table *per* scope, so a caller
        // asking for one scope of it is asking for something this cannot give, and silently
        // handing back all of it would be found much later by whoever read the rendered page.
        if format == Format::Variables && scope.is_some() {
            return Err(format!("--format variables covers every scope; {USAGE}"));
        }

        Ok(Self {
            format,
            scope: scope.unwrap_or(Scope::All),
        })
    }
}

const USAGE: &str =
    "usage: config-schema [--format json|markdown|toml|variables] [--scope all|server|builder]";
