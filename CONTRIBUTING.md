# Contributing to Portfolio

Thank you for your interest in contributing! This document provides guidelines and instructions for contributing to this project.

## Development Setup

### Prerequisites

- Rust (stable) with the `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- [Trunk](https://trunkrs.dev): `cargo install trunk` (or `cargo binstall trunk`)
- Node.js (only for the Tailwind CSS build step)
- Docker (optional, for containerized builds)

### Getting Started

1. Clone the repository:

   ```bash
   git clone https://github.com/timschoenle/Portfolio.git
   cd Portfolio
   ```

2. Run the frontend dev server:

   ```bash
   cd apps/frontend
   npm install
   trunk serve
   ```

3. Open [http://localhost:8080](http://localhost:8080) in your browser.

## Checks

Before opening a pull request, make sure the following pass:

```bash
cargo fmt --check
cargo test -p portfolio-data
cargo clippy -p frontend --target wasm32-unknown-unknown -- -D warnings
cargo clippy -p portfolio-data -p server -p resume-generator -- -D warnings
```

## Translations

All user-visible strings live in `crates/data/i18n/{en,de}.json`. Both files
must define exactly the same key set — `cargo test -p portfolio-data` enforces
this. When adding UI text, add the key to **both** files.

## Commit Messages

Use [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`,
`fix:`, `chore:`, …).
