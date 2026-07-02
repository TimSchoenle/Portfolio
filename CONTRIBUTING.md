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

## Translations

All user-visible strings live in `crates/data/i18n/{en,de}.json`. Both files
must define exactly the same key set — `cargo test -p portfolio-data` enforces
this. When adding UI text, add the key to **both** files.

## Commit Messages

Use [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`,
`fix:`, `chore:`, …).
