# Contributing to Portfolio

This repository is the source of a personal website and is published under a proprietary license
(see [LICENSE](LICENSE)). Bug reports and security reports are welcome. Pull requests from outside
contributors are not accepted, because the license grants no right to modify or redistribute the
code. The rest of this document describes the development workflow the maintainer follows.

## Development setup

### Prerequisites

- Rust, at the version in `rust-version` of the root `Cargo.toml`, with the WebAssembly target:
  `rustup target add wasm32-unknown-unknown`
- The [Dioxus CLI](https://dioxuslabs.com) (`dx`), at the version the image pins in
  `DIOXUS_CLI_VERSION` in the `Dockerfile`: `cargo install --locked dioxus-cli --version 0.7.9`
- [cargo-about](https://github.com/EmbarkStudios/cargo-about), only to render the third-party
  license page: `cargo install --locked cargo-about`
- [just](https://github.com/casey/just), which runs every check CI runs
- Node.js, only for the Tailwind CSS build step
- Docker, only for building the image

### Getting started

1. Clone the repository:

   ```bash
   git clone https://github.com/TimSchoenle/Portfolio.git
   cd Portfolio
   ```

2. Generate the build-time artifacts the web build embeds. Both are optional: without them
   `build.rs` embeds empty defaults and the affected pages render their empty state.

   ```bash
   cargo run --profile tools -p resume-generator -- apps/web/generated
   just licenses
   ```

3. Run the web dev server (SSR + hydration):

   ```bash
   cd apps/web
   npm ci && npm run build:css
   PORTFOLIO_CONFIG=../../legal dx serve --platform web
   ```

   `PORTFOLIO_CONFIG` points the loader at the legal document templates in `legal/`. The server
   refuses to start without an imprint and a privacy notice, and none is compiled in. The
   templates contain placeholders; the published texts are maintained outside this repository.

4. Open <http://localhost:8080>.

## Checks

Before opening a pull request, run the same checks CI runs:

```bash
just verify   # fmt, lint, docs, test
```

`just --list` shows the individual recipes.

## Generated files

`README.md` is rendered from
[`.github/templates/README.md.hbs`](.github/templates/README.md.hbs), and `config.example.toml`
from [`.github/templates/config.example.toml.hbs`](.github/templates/config.example.toml.hbs).
Edit the templates, never the output. CI renders both on every pull request and commits the result
back to the branch, so no local tooling is required.

Their variables include the configuration reference: one table per binary, generated from the
`Describe` derives on `ServerConfig` and `BuilderConfig` in `crates/config`:

```bash
# The server's table, as Markdown.
cargo run -p portfolio-config --features config-schema --example config-schema \
  -- --format markdown --scope server
# The builder's table, as Markdown.
cargo run -p portfolio-config --features config-schema --example config-schema \
  -- --format markdown --scope builder
# Every key, as the versioned JSON contract.
cargo run -p portfolio-config --features config-schema --example config-schema
```

The split is deliberate: `github.*` is read only by the build-time `update-repos` tool, and one
flat table would tell an operator their deployment needs a GitHub token. It does not.

Document fields the way rustdoc asks: a summary sentence, a blank line, then as much reasoning as
it takes. Only the summary reaches the table, so nothing has to be kept short for the README's
sake.

A new configuration block needs no registration. Add it to the aggregate the binary loads and it
appears in the README, because that aggregate is what the generator describes.

Two more files come out of the same types and are committed rather than rendered on demand:
`docs/config.contract.json`, which a chart's CI reads to check that what it renders is what this
image loads, and the `terrace-config:labels` region in the `Dockerfile`, which makes that document
discoverable on the image without pulling a layer. Rewrite both with:

```bash
just regenerate
```

That recipe writes and never checks. The check is
`TimSchoenle/actions/actions/rust/config-contract`, which the `Config Contract` job in **Build**
runs on every pull request, so there is exactly one implementation of each and they cannot
disagree about where the region ends. `just render <format>` prints one rendering without writing
it anywhere.

The `config-schema` feature is off by default and never enabled by a build that ships;
`cargo clippy --all-features --all-targets` is what keeps the generator compiling.

## Translations

All user-visible strings live in `crates/data/i18n/{en,de}.json`, including labels, `aria-label`
and `title` attributes. Both files must define exactly the same key set, in the same order;
`cargo test -p portfolio-data` enforces the key set. When adding UI text, add the key to **both**
files. The English copy is US English, which a test also enforces.

## Writing style

Documentation, comments and commit messages use US English (`license`, `artifact`,
`serialize`), the same variant as the site's English copy. A comment states what the code does
now and why; how it used to be belongs in the commit message.

## Commit messages

Use [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `chore:`, …).
release-please derives the version and the changelog from them.
