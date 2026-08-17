# Contributing to Portfolio

Thank you for your interest in contributing! This document provides guidelines and instructions for contributing to this project.

## Development Setup

### Prerequisites

- Rust (stable) with the `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- [Dioxus CLI](https://dioxuslabs.com) (`dx`): `cargo install dioxus-cli` (or `cargo binstall dioxus-cli`)
- Node.js (only for the Tailwind CSS build step)
- Docker (optional, for containerized builds)

### Getting Started

1. Clone the repository:

   ```bash
   git clone https://github.com/timschoenle/Portfolio.git
   cd Portfolio
   ```

2. Run the web dev server (SSR + hydration):

   ```bash
   cd apps/web
   npm ci && npm run build:css
   dx serve --platform web
   ```

3. Open [http://localhost:8080](http://localhost:8080) in your browser.

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

Its one variable is the configuration reference, generated from the `Describe`
derives on the blocks in `crates/config`:

```bash
cargo run -p portfolio-config --features config-schema --example config-schema \
  -- --format markdown   # the tables the README embeds
cargo run -p portfolio-config --features config-schema --example config-schema
                         # the same thing as the versioned JSON contract
```

Two rules follow from the table being generated:

- A field's `///` comment is **one summary sentence on one line** — it is copied
  verbatim into a Markdown table cell. Longer reasoning goes in `//` comments
  above the field.
- A new config block only reaches the README once it is listed in the
  `Documented` aggregate in `crates/config/examples/config-schema.rs`.

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
