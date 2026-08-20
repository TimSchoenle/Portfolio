# Contributing to Portfolio

Thank you for your interest in contributing! This document provides guidelines and instructions for contributing to this project.

## Development Setup

### Prerequisites

- Rust (stable) with the `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- [Dioxus CLI](https://dioxuslabs.com) (`dx`): `cargo install dioxus-cli` (or `cargo binstall dioxus-cli`)
- [cargo-about](https://github.com/EmbarkStudios/cargo-about) (only to render the third-party licences page): `cargo install --locked cargo-about`
- Node.js (only for the Tailwind CSS build step)
- Docker (optional, for containerized builds)

### Getting Started

1. Clone the repository:

   ```bash
   git clone https://github.com/timschoenle/Portfolio.git
   cd Portfolio
   ```

2. Render the third-party licence inventory the `/licenses` page reads
   (optional — without it `build.rs` embeds an empty default and the page says
   it has nothing to show):

   ```bash
   just licenses
   ```

3. Run the web dev server (SSR + hydration):

   ```bash
   cd apps/web
   npm ci && npm run build:css
   dx serve --platform web
   ```

4. Open [http://localhost:8080](http://localhost:8080) in your browser.

## Checks

Before opening a pull request, make sure the following pass:

```bash
cargo fmt --check
cargo test -p portfolio-data
cargo test -p web --no-default-features --features server
cargo clippy -p web --no-default-features --features web --target wasm32-unknown-unknown -- -D warnings
cargo clippy -p web --no-default-features --features server -- -D warnings
cargo clippy -p portfolio-data -p resume-generator -p update-repos -- -D warnings
```

## Generated Files

`README.md` is rendered from
[`.github/templates/README.md.hbs`](.github/templates/README.md.hbs) — edit the
template, never the README. CI renders it on every pull request and commits the
result back to the branch, so there is no toolchain to install locally.

Its variables are the configuration reference — one table per binary, generated
from the `Describe` derives on `ServerConfig` and `BuilderConfig` in
`crates/config`:

```bash
cargo run -p portfolio-config --features config-schema --example config-schema \
  -- --format markdown --scope server
cargo run -p portfolio-config --features config-schema --example config-schema \
  -- --format markdown --scope builder
cargo run -p portfolio-config --features config-schema --example config-schema
                         # every key, as the versioned JSON contract
```

The split is deliberate: `github.*` is read only by the build-time `update-repos`
tool, and one flat table would tell an operator their deployment needs a GitHub
token. It does not.

Document fields the way rustdoc asks: a summary sentence, a blank line, then as
much reasoning as it takes. Only the summary reaches the table, so nothing has to
be kept short for the README's sake.

A new config block needs no registration. Add it to the aggregate the binary
loads and it is in the README, because that aggregate is what the generator
describes.

Two more files come out of the same types and are committed rather than
rendered on demand: `docs/config.contract.json`, which a chart's CI reads to
check that what it renders is what this image loads, and the
`terrace-config:labels` region in the `Dockerfile`, which is what makes that
document discoverable on the image without pulling a layer. Rewrite both with:

```bash
just regenerate
```

That recipe writes and never checks. The checking is
`TimSchoenle/actions/actions/rust/config-contract`, which the `Config Contract`
job in **Build** runs on every pull request — so there is exactly one
implementation of each, and they cannot disagree about where the region ends.
`just --list` has the rest, including `just render <format>` for reading one
rendering without redirecting it anywhere.

The `config-schema` feature is off by default and is never enabled by a build
that ships; `cargo clippy --all-features --all-targets` is what keeps the
generator compiling.

## Translations

All user-visible strings live in `crates/data/i18n/{en,de}.json`. Both files
must define exactly the same key set — `cargo test -p portfolio-data` enforces
this. When adding UI text, add the key to **both** files.

## Commit Messages

Use [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`,
`fix:`, `chore:`, …).
