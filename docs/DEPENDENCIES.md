# Dependencies

Why the workspace manifests are set the way they are: which features each dependency takes, which
it leaves out, and how the build profiles differ.

`Cargo.toml` keeps one short comment per entry and points here.

## Workspace fields

`[workspace.package]` leaves out two fields.

- **`version`.** release-please's `cargo-workspace` plugin reads `[package] version` from each
  member and refuses `version.workspace = true`. Every manifest carries its own copy, and a release
  moves them together.
- **`license`.** The members are licensed to no one. `cargo about` in `apps/web/about.toml` would
  refuse `LicenseRef-Proprietary` on the crates it inventories.

The root `[package]` repeats the inherited values because
`TimSchoenle/actions/actions/common/readme-variables` reads that table alone and does not resolve
`.workspace = true`. The action deliberately refuses to take the description from the GitHub API. A
description edited in the web UI would change the rendered README with no commit behind it, and the
next unrelated pull request would fail the README drift gate for a reason its own diff does not
contain. The description, homepage and license are the same three strings as the image's OCI
labels.

`rust-version` is the toolchain the image is built with: the release `rust-toolchain.toml` pins and
the `rust:<version>-slim` base carries. Nothing tests it as a floor. Renovate's `rust toolchain`
group moves both `rust-version` lines together with those two, through a custom manager in the
shared `rust-application` preset.

## Lints

`missing_docs` fires only on an effectively-exported item. It reaches the libraries in `crates/`
and nothing in `apps/`, because all three binaries declare their modules privately. The rustdoc
link lints do fire inside a private module, so the `doc` job still gates the binaries. Coverage
there is a review criterion.

`clippy::missing_docs_in_private_items` is deliberately absent. A private helper whose body is six
obvious lines has nothing to say, and the private constant whose value came from a rate limit has
everything to say. No lint tells them apart.

`clippy::pedantic` is a code-quality choice this workspace made for itself.
[RUST.md](https://github.com/TimSchoenle/actions/blob/main/docs/doc-comments/RUST.md) keeps it out
of the documentation gate. The documentation lints are named individually, so removing `pedantic`
keeps every one of them.

## terrace-config

The loader layers struct defaults, then TOML, then `PORTFOLIO_` environment variables, then a
secrets directory, then `_FILE` indirection. A key supplied by two of the last three is refused
instead of resolved by precedence. It is pinned by tag, so an update to the loader is a deliberate
commit and not whatever `main` held at build time.

Default features are off, and two of the crate's six features are taken back:

- `loader` is the layering, and the reason the dependency exists.
- `explain` is the report that names which layer supplied each key and which keys more than one
  supplied. It costs no dependency. Both binaries print it beside a refused configuration, where the
  error names the key and the report names the mount it came from.

The rest are left out:

- `reload` is a supervisor that rebuilds a running service when its files change. It would link
  tokio, notify and tracing into every consumer for a reload this workspace cannot use.
  `crates/config/src/lib.rs` gives the two reasons.
- `schema` and `schema-cli` are turned on together by `portfolio-config`'s off-by-default
  `config-schema` feature, so the generator's argument parser never reaches a shipped binary.
  `schema` alone still reaches the server through `terrace-legal`, whose `LegalConfig` derives
  `Describe` unconditionally, and Cargo unifies the feature.
- `testing` is a dev-dependency of `crates/config`, whose tests are the only ones that use it.

## terrace-legal

The legal documents are configuration, under `legal.*`, not compiled-in copy. The four crates come
from one repository at one tag, pinned for the same reason as `terrace-config`, and at a tag that
resolves to the same `terrace-config`, so the workspace links one loader.

- `terrace-legal` is the configuration type, its validation into a catalog and the serving handle.
  It takes `terrace-config/schema` unconditionally, so the server links the schema half of the
  loader. That is the price of the configuration reference covering `legal.*` like every other
  block.
- `terrace-legal-axum` answers the public JSON routes with `ETag` and `304`. Its `router` feature is
  the ready-made pair of handlers, because the site publishes no OpenAPI document.
- `terrace-legal-model` and `terrace-legal-dioxus` build for `wasm32`: the wire types, and the
  provider plus the Markdown page component. The Dioxus crate renders Markdown to elements, never
  to an HTML string.

## i18nrs

Only the base component set, `dio`, is enabled. `dio-ssr` wires i18nrs's own cookie negotiation
through a `#[server]` round-trip, and upstream discards the `get_cookie` result. The site negotiates
the locale itself, so no server endpoint is needed. The server reads `?lang=`, the `lang` cookie
and `Accept-Language` from the request, and the client starts from the `<html lang>` the server
rendered and falls back to `document.cookie`.

## Sentry

Error reporting is off unless an operator configures a DSN. One `sentry` crate re-exports
`sentry-tracing` and `sentry-tower`, so the three versions cannot drift. Default features are off,
and the transport is the setting worth reading twice.

The default transport is `reqwest` with `native-tls`. `native-tls` links OpenSSL, which the
`scratch` runtime image does not contain, so it would fail at exec time in production instead of
at build time in CI. `reqwest` with `rustls` is the wrong replacement too. `sentry` pins reqwest
0.13 while the Dioxus fullstack stack builds 0.12, so it would be a second async HTTP client. Its
`rustls` feature also means `aws-lc-rs`, a cmake and C build in an image that is cross-compiled to
musl and built for bit-reproducibility, plus `rustls-platform-verifier`, which looks for an OS
trust store `scratch` does not have.

`ureq` costs nothing. It is already in the graph for `portfolio-update-repos`, and its `rustls`
feature is `ring` with bundled `webpki-roots`, the stack the runtime image is built for. It is a
blocking client, which is fine because the SDK sends from its own background thread and never on a
request path.

The features taken:

- `backtrace` and `debug-images` give stack traces and the module list that symbolicates them.
- `panic` turns a panic into an issue. A panic in a handler unwinds that request's task and leaves
  the process serving, so without the hook it is invisible.
- `tracing` is the subscriber layer.
- `tower-axum-matched-path` names one transaction per request by its matched route instead of its
  URI, so `/api/v1/legal/{slug}` is one transaction name and not one per document.

The defaults left out:

- `logs`, `metrics` and `release-health` are separate Sentry products this site does not use. Each
  would be an egress channel with no key in the configuration reference.
- `contexts` adds an OS, host and runtime tag block by pulling `os_info`, which carries a
  fifteen-crate `objc2` tree for macOS into `Cargo.lock` and so into the `/licenses` page. What it
  reports is a constant of the image, which the release tag already names.

`tracing-subscriber` serves the same feature. A Sentry layer has to be a layer of the subscriber,
and `dioxus_logger::initialize_default` builds one that cannot be extended afterwards. With Sentry
on, the server installs its own subscriber with the same `RUST_LOG` contract before
`dioxus::serve`, which then sees `tracing::dispatcher::has_been_set` and leaves it alone. Its
`env-filter` feature is already in the graph through `dioxus-logger`, so it adds nothing to
`THIRD-PARTY-NOTICES`.

## csp-shell

The Content-Security-Policy is built from the document the server is about to send, not from a
hand-maintained list of hashes that drifts the moment the shell changes. The `cloudflare` feature
is the crate's `presets` and `nonce` pair: the origins Cloudflare's products load from, the
directives they have to appear in, and the per-response nonce for the script the edge injects. It
is pinned by tag like `terrace-config`, and it comes from git because it is not on crates.io.

It is server-only, through `dep:` under `apps/web`'s `server` feature. The `nonce` feature pulls
`getrandom`, which has no default backend on `wasm32-unknown-unknown`, and the client never builds
a response header.

## Typst

The resume PDF engine is embedded as a library, so the generator is a self-contained `cargo run`
with no external binary in CI or the image. It writes a tagged PDF 1.7 with subsetted fonts and live
links.

`typst-render` rasterizes the Open Graph card. Facebook, LinkedIn, Slack and X all refuse SVG, so
the social image has to be a raster. Typst is already here with the brand font embedded, so the
card is a second small document instead of a new toolchain.

`qrcode` takes no default features. Its `image` and `svg` renderers would pull in a decoder stack
for output the generator does not use, because the modules are emitted as Typst vectors.

## time

`time` is exact-pinned. 0.3.52 changed `Parsable::parse`'s signature, which broke `cookie 0.18.1`,
pulled in through dioxus-fullstack, reqwest and `cookie_store`. Renovate moves the pin forward as
compatible releases land, so it tracks the newest version that builds. Drop the `=` once `cookie`
no longer constrains the choice.

## jsonschema

A dev-dependency of `crates/config`, which validates configurations against the JSON Schema the
published contract carries with a stock validator, as a consumer of the contract would. Default
features are off because they pull `reqwest` and a TLS stack to resolve remote `$ref`s, and the
contract's schema has none. It is the version terrace-config tests its own contracts with.

## Build profiles

`dx bundle --release` compiles under profiles it injects itself, `wasm-release` for the client and
`server-release` for the server. Both inherit `release`, so `release` reaches the shipped binaries
through them. `cargo build --release` and `cargo test --release` use it directly.

`release` unwinds on panic instead of aborting. It also builds the SSR server, where a panic in one
request handler must fail that request and not the process with every request in flight. The wasm
client is unaffected: `wasm32-unknown-unknown` has no unwinding, so a panic aborts there anyway and
the bundle keeps its size.

`wasm-release` is declared even though `dx` would inject it. `dx` strips the linked binary with
`rust-objcopy`, using the `strip` setting it finds along the `inherits` chain. From `release` that
is `--strip-all`, which since LLVM 23 (Rust 1.99) deletes every custom wasm section, including the
`__wasm_bindgen_unstable` metadata `wasm-bindgen` reads next. Declaring `strip = false` stops it,
and `release` keeps stripping the server. `dx bundle --debug-symbols=false` still drops the
client's debug info and `name` section afterwards.

`tools` builds `portfolio-update-repos` and `portfolio-resume`, which run once during the image
build and are never shipped. Everything `release` optimizes for is the wrong trade there:
`opt-level = "z"`, fat LTO and one codegen unit buy a smaller binary nobody keeps, at the cost of
the slowest compile of a 296-crate tree. Measured on `portfolio-resume` from a cold target
directory, the build takes 5m03s under `release` and 1m29s under `tools`. `opt-level = 1` is also
faster at runtime, because Typst's layout pass is compute-bound and `z` trades speed for size.
`tools` inherits `release` and not `dev`, so the generators still run optimized and without
`debug-assertions`.

## Member manifests

`portfolio-config`'s `config-schema` feature is off by default and has to stay off.
It compiles a proc macro and the contract emitter, and a running binary needs neither. Only `cargo
run -p portfolio-config --features config-schema --example config-schema`, the `--all-features` CI
gates and two Dockerfile stages turn it on: `deps` cooks its dependencies and `contract-builder`
runs the example. Neither stage's output is a binary in the runtime image, and `contract-builder`
hands over only a JSON file. It takes `schema-cli` and not `schema`, because
`schema-cli` is the `--format` vocabulary, argument parsing and dispatch every consumer of `schema`
was writing by hand, and it pulls no new dependency. The `#[config(...)]` helper attributes are
gated with it, because they belong to the derive: a build without the derive fails with
`cannot find attribute config` the moment one is unconditional.

The `verify-labels` example exists because the image's `LABEL` block is literal text in the
Dockerfile. The ways it goes wrong, a line dropped on a branch nobody read or a base image adding
its own, are invisible to a source diff, so the check reads back what `docker inspect` reports.

`terrace-config`'s `testing` feature is a dev-dependency of `portfolio-config`, because a service
asking for everything the crate does at runtime should not link a test harness. The harness derives
every name a test writes from the `Terrace` under test, so a test cannot keep passing against a
spelling the crate no longer reads.

`portfolio-data` is a dev-dependency of `portfolio-config`. The schema module takes the site's
languages as a parameter, because the config crate does not depend on the data crate at runtime,
and the examples and tests pass `portfolio_data::LANGUAGES`. That only works because
`portfolio-data` depends on nothing else in the workspace.

`apps/web` compiles Sentry into every server build and switches it on with `sentry.enabled`,
instead of putting it behind a Cargo feature. A feature would make possible an image where the
loader accepts `PORTFOLIO_SENTRY__ENABLED=true` and nothing happens. Off, it costs one `if` at boot.
The client has no Sentry: reporting a wasm panic would need a browser SDK loaded by the page, which
is a different product, a `connect-src` entry in the Content-Security-Policy and a line in the
privacy notice.

`portfolio-server` is server-only because it hashes response bodies for the Content-Security-Policy
and its nonces need an OS CSPRNG.
