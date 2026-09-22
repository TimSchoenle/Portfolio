//! The one description of this workspace's configuration surface that every generated artefact
//! is rendered from.
//!
//! `examples/config-schema.rs` renders the README tables, `config.example.toml`, the contract and
//! the Dockerfile's label block from here; `examples/verify-labels.rs` checks a built image's
//! labels against the contract built here. Before this module each example assembled its own
//! schema and contract, so a change landing in one and not the other would have had the label
//! check compare an image against a document the image does not publish.
//!
//! # What the types cannot state
//!
//! The server refuses to start unless `legal.documents` holds every slug in
//! [`REQUIRED_DOCUMENTS`](crate::REQUIRED_DOCUMENTS). `LegalConfig` types that key as an open map
//! with an empty default, so a schema built from the types alone publishes `{}` as valid — and a
//! deployment generated from it passes every gate and fails at boot. [`schema`] therefore refines
//! every scope that contains `legal` with [`legal_catalog_builder`], the value the server boots
//! with, so the published requirement and the enforced one have a single source.
//!
//! # Locales are a parameter
//!
//! [`legal_catalog_builder`] takes the site's languages, and they live in `portfolio-data`, which
//! this crate does not depend on at runtime. Every function here that needs them takes them, and
//! the callers — the two examples and the tests — pass `portfolio_data::LANGUAGES`, reached as a
//! development dependency. None of the refinements depends on the locales today; taking them
//! anyway is what makes this the builder the server runs rather than a second one assembled for
//! documentation, which is the one thing that could drift.

use terrace_config::schema::cli::Cli;
use terrace_config::schema::{
    App, Contract, ContractBuilder, Docs, External, ExternalVar, JsonSchema, Schema, TomlExample,
};

use crate::{BuilderConfig, ConfigError, ServerConfig, legal_catalog_builder};

/// Where [`ServerConfig`] mounts `terrace_legal::LegalConfig`, which is where the legal
/// refinements are anchored.
///
/// The field name, not a key path: the paths of the requirement itself (`documents`) come from
/// `terrace-legal`, relative to this mount.
const LEGAL: &str = "legal";

/// Whose configuration to describe.
///
/// One flat table of every key says that a deployment needs a GitHub token. It does not:
/// `github.*` belongs to `update-repos`, a build-time tool that exits during the image build, and
/// the SSR server never loads it. The scopes are split the way the binaries are.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Scope {
    /// Every key in the workspace.
    All,
    /// The keys the SSR server loads. The only scope a contract describes: the runtime image
    /// holds the server and nothing else.
    Server,
    /// The keys the `update-repos` builder loads.
    Builder,
}

/// The schema one scope describes, with the defaults its binary starts from and every refinement
/// its binary enforces at boot.
///
/// Built through [`crate::terrace`] rather than a bare dialect, so `PORTFOLIO_CONFIG` and
/// `PORTFOLIO_SECRETS_DIR` are reported with the names this workspace configures. The defaults
/// come from values built here, not from the process environment, so the output is the same on a
/// developer's machine and on a runner where none of the variables are set.
///
/// `locales` are the site's languages, `portfolio_data::LANGUAGES`; see the module documentation.
///
/// # Errors
/// [`ConfigError`] if a default cannot be serialised, if the two scopes of [`Scope::All`]
/// describe one key differently, or if a refinement names a key the schema does not have or
/// cannot spell — the last is a mount point renamed without this module following it.
pub fn schema(scope: Scope, locales: &[&'static str]) -> Result<Schema, ConfigError> {
    match scope {
        // Refined after the merge, as terrace-config requires: a refined key and its unrefined
        // twin are two descriptions of one path, which `merge` refuses.
        Scope::All => refine_server(server()?.merge(builder()?), locales),
        Scope::Server => refine_server(server()?, locales),
        // `BuilderConfig` has no `legal` block, so there is nothing of the server's to refine.
        Scope::Builder => builder(),
    }
}

/// The contract the runtime image publishes: the server scope, [`app`] and [`external`].
///
/// This is what `--format contract` renders (without a build stamp) and what
/// `examples/verify-labels.rs` checks an image's labels against, so both read one document.
///
/// # Errors
/// As [`schema`], and [`ConfigError`] when [`ContractBuilder::build`] refuses the external
/// surface.
pub fn contract(locales: &[&'static str]) -> Result<Contract, ConfigError> {
    with_external(schema(Scope::Server, locales)?.into_contract(app())).build()
}

/// The generator every rendering but `--format variables` goes through.
///
/// Here rather than in `examples/config-schema.rs` so the contract it renders can be compared, in
/// a test, with [`contract`] — the one `examples/verify-labels.rs` checks an image against.
#[must_use]
pub fn cli() -> Cli<'static> {
    Cli::new(app())
        // No `$id`: this workspace publishes no schema document at a URL, and an editor told to
        // resolve one that is not there fails louder than one given nothing to resolve.
        .json_schema(JsonSchema::new().title("portfolio configuration"))
        .toml_example(toml_example())
        .contract_with(&with_external)
}

/// How `config.example.toml` renders, for [`cli`] and for the example file inside the README
/// payload alike.
///
/// [`Docs::Full`] rather than the default summary: this is the file an operator edits with no
/// rustdoc open beside it, and every paragraph the fields carry is a paragraph the hand-written
/// version of this file used to carry too. No header, because the template supplies one — what
/// this file is and how to point the loader at it are facts about the repository rather than
/// about the schema, and the layering itself is documented once, in `README.md`.
#[must_use]
pub fn toml_example() -> TomlExample {
    TomlExample::new().header(false).docs(Docs::Full)
}

/// Adds this image's [`external`] surface to a contract under construction.
///
/// The form [`Cli::contract_with`] takes, so the generator's contract and [`contract`] are
/// finished by the same function.
#[must_use]
pub fn with_external(builder: ContractBuilder) -> ContractBuilder {
    builder.external(external())
}

/// The image this workspace builds, as the contract names it.
///
/// The version is `v`-prefixed because that is how this repository tags its images, and the field
/// exists to be compared against a tag. `CARGO_PKG_VERSION` yields the bare form.
#[must_use]
pub fn app() -> App {
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
#[must_use]
pub fn external() -> External {
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

/// The keys the SSR server loads, with the defaults it starts from and nothing else.
fn server() -> Result<Schema, ConfigError> {
    crate::terrace()
        .schema::<ServerConfig>()
        .with_defaults_from(&ServerConfig::default())
}

/// The keys the `update-repos` builder loads, with the defaults it starts from.
fn builder() -> Result<Schema, ConfigError> {
    crate::terrace()
        .schema::<BuilderConfig>()
        .with_defaults_from(&BuilderConfig::default())
}

/// Publishes what the server enforces at boot beyond its types, from the builder it enforces it
/// with.
fn refine_server(schema: Schema, locales: &[&'static str]) -> Result<Schema, ConfigError> {
    schema.refine_with(LEGAL, &legal_catalog_builder(locales))
}

#[cfg(test)]
mod tests {
    use portfolio_data::LANGUAGES;
    use serde_json::{Value, json};
    use terrace_config::schema::cli::Request;
    use terrace_config::schema::{Contract, Key};
    use terrace_legal::LegalConfig;

    use super::{Scope, cli, contract, schema};
    use crate::{REQUIRED_DOCUMENTS, legal_catalog_builder};

    /// The committed legal texts, joined into one document. Each fragment owns a distinct table.
    const COMMITTED_LEGAL_TEXTS: [&str; 3] = [
        include_str!("../../../legal/legal.toml"),
        include_str!("../../../legal/imprint.toml"),
        include_str!("../../../legal/privacy.toml"),
    ];

    fn published() -> Contract {
        contract(&LANGUAGES).expect("the contract builds")
    }

    fn documents_key(keys: &[Key]) -> &Key {
        keys.iter()
            .find(|key| key.path == "legal.documents")
            .expect("`legal.documents` is a key")
    }

    /// The slugs as the refinement publishes them: a sorted set, whatever order the source lists.
    fn required_slugs() -> Value {
        let mut slugs = REQUIRED_DOCUMENTS.to_vec();
        slugs.sort_unstable();
        json!(slugs)
    }

    /// What a consumer of the contract does with it: validate a rendered document against the
    /// JSON Schema it carries, with a stock validator and nothing of this crate's.
    fn schema_errors(document: &Value) -> Vec<String> {
        let validator =
            jsonschema::validator_for(&published().json_schema).expect("the schema compiles");
        validator
            .iter_errors(document)
            .map(|error| format!("{}: {error}", error.instance_path()))
            .collect()
    }

    /// The committed texts, loaded through the real dialect into the shape a chart renders.
    fn committed_document() -> Value {
        let mut document = Value::Null;
        terrace_config::testing::Harness::over(crate::terrace()).run(|jail| {
            jail.config(COMMITTED_LEGAL_TEXTS.join("\n"))?;
            document = jail.load()?;
            Ok(())
        });
        document
    }

    #[test]
    fn the_contract_publishes_the_required_documents_on_the_key() {
        let contract = published();
        let key = documents_key(&contract.schema.keys);
        let constraint = key.constraint.as_ref().expect("a map carries a constraint");
        assert_eq!(constraint["required"], required_slugs());
        assert!(key.required, "a default the server refuses is no default");
        assert_eq!(key.default, None);
        assert_eq!(key.default_value, None);
    }

    #[test]
    fn the_contract_json_schema_requires_the_documents() {
        let json_schema = &published().json_schema;
        let documents = &json_schema["properties"]["legal"]["properties"]["documents"];
        assert_eq!(documents["required"], required_slugs());
        assert!(documents.get("default").is_none(), "{documents}");
    }

    /// Every scope that loads `legal` is refined, including the merged one the example file and
    /// the README payload are rendered from. The builder's scope has no `legal` to refine.
    #[test]
    fn every_scope_containing_legal_is_refined() {
        for scope in [Scope::All, Scope::Server] {
            let schema = schema(scope, &LANGUAGES).expect("the schema builds");
            let key = documents_key(&schema.keys);
            let constraint = key.constraint.as_ref().expect("a map carries a constraint");
            assert_eq!(constraint["required"], required_slugs(), "{scope:?}");
            assert!(key.required, "{scope:?}");
        }
        let builder = schema(Scope::Builder, &LANGUAGES).expect("the schema builds");
        assert!(
            builder
                .keys
                .iter()
                .all(|key| !key.path.starts_with("legal."))
        );
    }

    #[test]
    fn the_json_schema_rejects_an_empty_document_map() {
        let errors = schema_errors(&json!({ "legal": { "documents": {} } }));
        assert!(!errors.is_empty(), "an empty map must be refused");
        for slug in REQUIRED_DOCUMENTS {
            assert!(
                errors.iter().any(|error| error.contains(slug)),
                "{slug} is not named in {errors:?}"
            );
        }
    }

    #[test]
    fn the_json_schema_rejects_a_map_missing_one_required_document() {
        let document = json!({ "legal": { "documents": {
            "imprint": { "body": { "en": "Imprint", "de": "Impressum" } },
        } } });
        let errors = schema_errors(&document);
        assert!(
            errors.iter().any(|error| error.contains("privacy")),
            "{errors:?}"
        );
    }

    /// A document *without* `legal.documents` is not refused by the JSON Schema, and cannot be:
    /// the loader takes the key from the environment or a mounted file as readily as from the
    /// document, so terrace-config publishes presence on the key (`required: true`) rather than as
    /// a JSON Schema `required` that would reject a chart supplying every text through `_FILE`.
    /// Pinned so that changing it is a decision rather than a surprise.
    #[test]
    fn an_absent_document_map_is_the_keys_requirement_not_the_json_schemas() {
        assert_eq!(schema_errors(&json!({})), Vec::<String>::new());
        assert_eq!(schema_errors(&json!({ "legal": {} })), Vec::<String>::new());
        assert!(documents_key(&published().schema.keys).required);
    }

    #[test]
    fn the_committed_legal_texts_satisfy_the_json_schema() {
        let document = committed_document();
        assert!(document["legal"]["documents"].is_object(), "{document}");
        assert_eq!(schema_errors(&document), Vec::<String>::new());
    }

    /// What the JSON Schema cannot express: every hosted document has a text in every site
    /// language. A configuration naming both slugs is schema-valid and still refused at boot when
    /// `privacy` has no German body.
    #[test]
    fn a_schema_valid_configuration_missing_a_language_is_still_refused_at_boot() {
        let mut document = committed_document();
        document["legal"]["documents"]["privacy"]["body"]
            .as_object_mut()
            .expect("privacy is hosted")
            .remove("de")
            .expect("privacy has a German text to remove");
        assert_eq!(schema_errors(&document), Vec::<String>::new());

        let legal: LegalConfig =
            serde_json::from_value(document["legal"].clone()).expect("still a legal block");
        let keys: Vec<String> = legal_catalog_builder(&LANGUAGES)
            .build(&legal)
            .expect_err("the server refuses it")
            .with_prefix("legal")
            .iter()
            .map(terrace_legal::ConfigIssue::key)
            .collect();
        assert_eq!(keys, ["legal.documents.privacy.body.de"]);
    }

    /// `examples/config-schema.rs --format contract` and `examples/verify-labels.rs` read one
    /// contract: the generator's rendering and [`contract`] are byte-identical.
    #[test]
    fn the_generator_and_the_label_check_read_one_contract() {
        let request = Request::parse(["--format".to_owned(), "contract".to_owned()])
            .expect("a valid request");
        let rendered = cli()
            .render(
                &request,
                schema(Scope::Server, &LANGUAGES).expect("the schema builds"),
            )
            .expect("the contract renders");
        assert_eq!(
            rendered,
            published().to_json().expect("the contract serialises")
        );
    }
}
