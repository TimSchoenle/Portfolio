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
//! The same workflow regenerates the two artefacts the image publishes about itself —
//! `docs/config.contract.json` and the Dockerfile's `dev.terrace.config.*` label block — through
//! `just regenerate`, and the Config Contract job in `build.yaml` checks both afterwards. That
//! job is the gate; this is what keeps a pull request from having to clear it by hand, including
//! the release pull request, whose only change is the version the contract stamps into the
//! document.
//!
//! Nothing here reads the environment, so the output is the same on a developer's machine and on
//! a runner where none of the variables it describes are set. That is what makes the render
//! deterministic enough for the action to skip the commit when nothing changed.
//!
//! # What is left here
//!
//! The `--format` vocabulary, the dispatch across the renderings, the contract's build stamp and
//! the argument parsing for all of it are [`Cli`] and [`Request`]. They were the same two hundred
//! lines in every repository that had a generator, which is how several of them ended up
//! disagreeing about how to cut a `LABEL` block back out of a Dockerfile.
//!
//! What stays is only what no other repository has: [`Scope`], because this workspace ships two
//! binaries that load two different roots, and `--format variables`, because the payload the
//! templates read mixes tables from *both* of them and so is not a rendering of any one
//! [`Schema`]. Both are expressed by choosing what to hand [`Cli::render`] rather than by
//! re-implementing it — [`Request::parse`] still owns every argument but `--scope`, so a
//! rendering added upstream arrives here without a line being written.
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
//! `--format variables` emits the whole render payload — both tables, the example file, and the
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
use terrace_config::schema::cli::{Cli, Format, Request, USAGE};
use terrace_config::schema::{
    App, Column, Docs, External, ExternalVar, JsonSchema, Key, Schema, TomlExample,
};

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

/// What this generator adds to the argument list [`USAGE`] describes.
///
/// Appended to it rather than replacing it: the format vocabulary belongs to [`Request`], and a
/// second copy of it here is the copy that goes stale the next time terrace-config grows a
/// rendering.
const EXTRA_USAGE: &str =
    "\n       and here also: [--format variables] [--scope all|server|builder]";

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
    let Some(request) = &options.request else {
        return variables();
    };

    Ok(Cli::new(app())
        // No `$id`: this workspace publishes no schema document at a URL, and an editor told to
        // resolve one that is not there fails louder than one given nothing to resolve.
        .json_schema(JsonSchema::new().title("portfolio configuration"))
        .toml_example(toml_example())
        .contract_with(&|builder| builder.external(external()))
        .render(request, schema(options.scope)?)?)
}

/// The image this workspace builds, as the contract names it.
///
/// The version is `v`-prefixed, because that is how this repository tags its images and the field
/// exists to be compared against a tag. `CARGO_PKG_VERSION` yields the bare form.
fn app() -> App {
    App::new("portfolio")
        .version(concat!("v", env!("CARGO_PKG_VERSION")))
        .source("https://github.com/TimSchoenle/Portfolio")
}

/// The part of the contract no derive can see.
///
/// `PORT`, `IP` and `RUST_LOG` are read by the Dioxus toolchain and by `tracing`, before any
/// layer of this loader exists, and they carry no `PORTFOLIO_` prefix — so nothing in the types
/// can report them. Declared here they are checked like any key: a chart passing `PORT: "http"`
/// fails the same gate that a chart passing `PORTFOLIO_ISR__TTL_SECS: "soon"` fails.
///
/// The defaults are the ones the Dockerfile's `ENV` block bakes in, which is where the image's
/// real behaviour is decided.
fn external() -> External {
    External::new()
        .var(
            ExternalVar::new("PORT")
                .owner("dioxus")
                .ty("u16")
                .default("8080")
                .docs("Bind port. Read by the Dioxus toolchain, not by this loader."),
        )
        .var(
            ExternalVar::new("IP")
                .owner("dioxus")
                .ty("IpAddr")
                .default("0.0.0.0")
                .docs("Bind address. Read by the Dioxus toolchain, not by this loader."),
        )
        .var(
            ExternalVar::new("RUST_LOG")
                .owner("tracing")
                .ty("String")
                .default("info")
                .docs("Verbosity, as `tracing` directives — `info`, `web=debug,info`."),
        )
        // What a pod carries that no image asked for, which `Unknown::Reject` names: the API
        // server's five, and the container runtime's one. An image on `scratch` contributes none
        // of its own. The third entry on that list — the service links — is not here and cannot
        // be: their names are built from the release name, so they belong to
        // `enableServiceLinks: false` on the pod.
        .ignore("KUBERNETES_*")
        .ignore("HOSTNAME")
}

/// How `config.example.toml` renders, in the one place both callers read it from.
///
/// [`Docs::Full`] rather than the default summary: this is the file an operator edits with no
/// rustdoc open beside it, and every paragraph the fields carry is a paragraph the hand-written
/// version of this file used to carry too. No header, because the template supplies one — what
/// this file is and how to point the loader at it are facts about the repository rather than
/// about the schema, and the layering itself is documented once, in `README.md`.
fn toml_example() -> TomlExample {
    TomlExample::new().header(false).docs(Docs::Full)
}

/// The whole render payload, as the strict JSON the template action takes.
///
/// One object for both templates. An unread name costs a template nothing — strict mode fails on
/// a reference the payload does not *define*, never on a definition nothing reads — so the
/// alternative, a payload per template, would only be two ways to get the same schema out.
///
/// This is the one rendering [`Cli`] cannot produce, and the reason this generator still has a
/// `--format` of its own: it mixes a table from each scope with a third built from both, so there
/// is no single [`Schema`] whose rendering it is.
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
        "exampleConfig": all.to_toml_example_with(&toml_example()).trim_end(),
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

/// The schema one scope describes.
fn schema(scope: Scope) -> Result<Schema, portfolio_config::ConfigError> {
    match scope {
        Scope::All => all(),
        Scope::Server => server(),
        Scope::Builder => builder(),
    }
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
    /// What to hand [`Cli::render`], or `None` for `--format variables` — see [`variables`].
    request: Option<Request>,
    /// Whose configuration to render. Resolved rather than optional: the whole-image formats fix
    /// their own scope, and everything else defaults to [`Scope::All`].
    scope: Scope,
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
    /// Take `--scope` and `--format variables`, and let [`Request::parse`] have the rest.
    ///
    /// Forwarding rather than parsing is what keeps this file from owning a copy of the format
    /// vocabulary: `--only`, `--path`, `--version`, `--revision`, `--created` and every rendering
    /// terrace-config ships are handled by the crate that defines them, with its messages, and a
    /// rendering added upstream needs no change here.
    fn from_args() -> Result<Self, String> {
        let mut scope = None;
        let mut variables = false;
        let mut forwarded = Vec::new();

        let mut args = std::env::args().skip(1);
        while let Some(flag) = args.next() {
            match flag.as_str() {
                "--scope" => {
                    scope = match args.next().as_deref() {
                        Some("all") => Some(Scope::All),
                        Some("server") => Some(Scope::Server),
                        Some("builder") => Some(Scope::Builder),
                        Some(other) => return Err(usage(&format!("unknown scope `{other}`"))),
                        None => return Err(usage("`--scope` takes a value")),
                    };
                }
                // The one rendering that is this repository's own. Every other spelling —
                // including one neither side knows — travels on to `Request::parse`, which is
                // where the "unknown format" message belongs and stays correct.
                "--format" => {
                    let value = args.next();
                    if value.as_deref() == Some("variables") {
                        variables = true;
                    } else {
                        forwarded.push(flag);
                        forwarded.extend(value);
                    }
                }
                _ => forwarded.push(flag),
            }
        }

        if variables {
            // Refused rather than ignored: the payload carries a table *per* scope, so a caller
            // asking for one scope of it is asking for something this cannot give, and silently
            // handing back all of it would be found much later by whoever read the rendered page.
            if scope.is_some() {
                return Err(usage("`--format variables` covers every scope"));
            }
            // Same reasoning for the rest of the argument list: `--only` slices and `--version`
            // stamps a rendering this does not produce, so anything passed alongside would have
            // been quietly dropped.
            if let Some(other) = forwarded.first() {
                return Err(usage(&format!(
                    "`--format variables` takes no other argument, and got `{other}`"
                )));
            }
            return Ok(Self {
                request: None,
                scope: Scope::All,
            });
        }

        let request = Request::parse(forwarded).map_err(|error| usage(error.message()))?;

        // A contract names the image it belongs to, and this workspace ships one: the `scratch`
        // runtime stage holding the SSR server. Its scope is decided here, not by a caller who
        // could otherwise publish a document claiming the server reads a build-time credential —
        // `github.*` belongs to `update-repos`, which runs during the build and then exits.
        let scope = if request.format().whole_image() {
            if scope.is_some() {
                return Err(usage(&format!(
                    "`--format {}` describes the runtime image and fixes its own scope",
                    request.format()
                )));
            }
            Scope::Server
        } else {
            scope.unwrap_or(Scope::All)
        };

        // The loader variables lead the server table and are absent from every other one. They
        // apply to both binaries, so a page repeating them above each scope would say the same
        // thing twice; the server scope is where an operator setting a deployment up will be.
        let request = if scope == Scope::Builder && request.format() == Format::Markdown {
            request.with_format(Format::MarkdownKeys)
        } else {
            request
        };

        Ok(Self {
            request: Some(request),
            scope,
        })
    }
}

/// A refusal, with both usage lines under it.
fn usage(message: &str) -> String {
    format!("{message}; {USAGE}{EXTRA_USAGE}")
}
