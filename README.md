# Portfolio

Personal portfolio at [tim-schoenle.de](https://tim-schoenle.de).

A **Rust + Dioxus** fullstack application (server-side rendered with WASM
hydration) styled with **Tailwind CSS**, served by an **Axum** server running from
a distroless container.

## Table of Contents

- [Architecture](#architecture)
- [Technology Stack](#technology-stack)
- [Features](#features)
- [Getting Started](#getting-started)
  - [Prerequisites](#prerequisites)
  - [Development](#development)
  - [Production Build](#production-build)
  - [Container](#container)
- [Deployment](#deployment)
  - [Probe Endpoints](#probe-endpoints)
  - [Read-only and Security Posture](#read-only-and-security-posture)
  - [Reproducible Builds](#reproducible-builds)
- [Project Data (`repos.json`)](#project-data-reposjson)
- [Contributing](#contributing)
- [Security](#security)
- [License](#license)

## Architecture

The project is a Cargo workspace composed of one shared library crate and three
applications.

| Crate | Purpose |
| --- | --- |
| `crates/data` | Shared, language-neutral data (config, skills, experience, repos schema) plus embedded `i18n/{en,de}.json` translations |
| `apps/web` | Dioxus 0.7 fullstack app: a single crate that compiles to both the WASM client (`web` feature) and the native Axum SSR server (`server` feature), with the JSON API, SEO documents, security headers, and probe endpoints |
| `apps/resume-generator` | Generates `resume/{en,de}.pdf` and `resume-fingerprint.json` (Typst, embedded subset of Liberation Sans) |
| `apps/update-repos` | Builder that fetches the user's active GitHub repositories (skipping archived, blacklisted, and >1-year-stale ones) and refreshes `apps/web/repos.json` using the shared `Repo`/`ReposFile` models |

## Technology Stack

- **App:** Rust, [Dioxus](https://dioxuslabs.com) 0.7 fullstack (Axum SSR + WASM
  hydration) with the built-in Dioxus router
- **Internationalization:** EN/DE via [i18nrs](https://crates.io/crates/i18nrs).
  Translations live in `crates/data/i18n/`; the active language is persisted in
  `localStorage` (`lang`) and detected from the browser language on first visit.
  A unit test in `crates/data` enforces key parity between both languages.
- **Styling:** Tailwind CSS v4 with custom design tokens
- **Build:** the [Dioxus CLI](https://dioxuslabs.com) (`dx`)
- **Data:** `apps/web/repos.json` is regenerated at build time from the GitHub
  API (all active repositories — archived, blacklisted, and >1-year-stale ones
  excluded) and embedded into the binary by `apps/web/build.rs`

## Features

- Routes: `/` (single-page sections `s1`-`s5`), `/imprint`, `/privacy`, and a 404 page
- Hero with a two-line display name, scroll parallax, and a live meta card
- Fixed chapter rail with scroll tracking and staggered reveal-on-scroll blocks
- Stack section: an interactive skill radar (per-skill hover tooltips, category
  filtering/dimming) and a chip matrix with confidence bars
- Projects: a GitHub statistics strip, language filter, and loading skeletons fed
  by `repos.json`
- Experience accordion with year badges and animated bodies (real career data)
- Contact: a terminal with a type-in `ssh` animation, oversized email, and action buttons
- Command palette (Cmd+K / Ctrl+K) with fuzzy search and keyboard navigation
- Language switcher (EN/DE) — every visible string is translated
- Localized, single-page, ATS-readable resume PDFs with SHA-256 fingerprints on the contact card. The
  generator scales typography down until the content fits one A4 page.
- Legal pages (imprint, privacy policy) localized and rendered from the translation files
- Server-side rendering with WASM hydration; per-route `<head>` metadata, JSON-LD,
  and server-negotiated locale for the first paint
- SEO: meta/OG tags, JSON-LD, `robots.txt`, `sitemap.xml`, and a web manifest
- Security headers (CSP, HSTS, and others) set by the server
- WASM client tuned for size: `opt-level = "z"`, LTO, and a single codegen unit

## Getting Started

### Prerequisites

- Rust (stable) with the `wasm32-unknown-unknown` target:
  `rustup target add wasm32-unknown-unknown`
- [Dioxus CLI](https://dioxuslabs.com) (`dx`): `cargo install dioxus-cli`
  (or `cargo binstall dioxus-cli`)
- Node.js (required only for the Tailwind CSS build step)
- Docker (optional, for containerized builds)

### Development

```bash
# Resume PDFs + resume-fingerprint.json. Run BEFORE the web build so the
# fingerprint is embedded (build.rs) and the PDFs are served from public/resume/.
cargo run -p resume-generator -- apps/web/generated

# Web dev server (SSR + hydration, http://localhost:8080)
cd apps/web
npm ci && npm run build:css
dx serve --platform web

# Tests (including i18n key parity)
cargo test -p portfolio-data
cargo test -p web --no-default-features --features server
```

### Production Build

```bash
cargo run --release -p resume-generator -- apps/web/generated
cd apps/web && npm ci && npm run build:css && dx bundle --platform web --release
```

### Container

The container performs all of the above and serves the result on port 8080:

```bash
docker build -t portfolio .
```

## Deployment

The image is built on a `distroless/cc` base holding the dynamically linked
`server` binary plus its sibling read-only `public/` assets under `/app` — no
shell, package manager, or writable system paths. The Helm chart lives in a
separate repository; the application and image here are prepared to run under a
hardened pod spec out of the box.

### Probe Endpoints

| Endpoint | Alias | Purpose |
| --- | --- | --- |
| `GET /api/health` | — | General health report with the current UTC time |
| `GET /api/health/live` | `GET /livez` | **Liveness** — process is running; failure restarts the container |
| `GET /api/health/ready` | `GET /readyz` | **Readiness** — the client bundle (`$DIST_DIR/index.html`, default `public/index.html`) is present and servable; failure removes the pod from the Service endpoints |

All probe responses are `no-store` (never cached). Readiness returns `503` until
the assets are present.

### Read-only and Security Posture

The server only reads from `$DIST_DIR` (its sibling `public/` dir) and writes
logs to stdout, so it runs unchanged with a fully read-only root filesystem (no
writable volume, not even `/tmp`). It is built to satisfy the restricted Pod
Security Standard:

- runs as numeric non-root `1001:1001` (so `runAsNonRoot` verifies statically)
- `readOnlyRootFilesystem: true`, `allowPrivilegeEscalation: false`
- all Linux capabilities dropped, `seccompProfile: RuntimeDefault`
- HTTP security headers (CSP, HSTS, `X-Content-Type-Options`, `X-Frame-Options`,
  `Referrer-Policy`, `Permissions-Policy`) set on every response
- listens on `:$PORT` (default `8080`); graceful shutdown on `SIGTERM`

Configuration is provided via environment variables: `DIST_DIR` (default
`public`), `IP` (default `0.0.0.0`), `PORT` (default `8080`), and `RUST_LOG`
(default `info`).

### Reproducible Builds

The build is pinned end-to-end so the image is reproducible:

- base images pinned by digest; Rust toolchain pinned via the base image
- the Dioxus CLI (`dx`) is pinned to an exact version (build arg
  `DIOXUS_CLI_VERSION`)
- `cargo` / `dx bundle` build `--locked` against the committed `Cargo.lock`; the
  Tailwind toolchain uses `npm ci` against `package-lock.json`
- pass `SOURCE_DATE_EPOCH` (and the OCI metadata build args `VCS_REF`, `VERSION`,
  `CREATED`, `SOURCE_URL`) for deterministic, self-describing images:

```bash
docker build \
  --build-arg SOURCE_DATE_EPOCH=$(git log -1 --format=%ct) \
  --build-arg VCS_REF=$(git rev-parse HEAD) \
  --build-arg VERSION=$(git describe --tags --always) \
  -t portfolio .
```

## Project Data (`repos.json`)

The projects section reads `repos.json`, which is embedded into the binary at
build time via `include_str!` (see `apps/web/build.rs`). The file is **generated
during the build**: the `update-repos` builder runs before the web build and
refreshes `apps/web/repos.json`. When it is absent (dev builds, `cargo check`),
`build.rs` substitutes an empty default so the `include_str!` always resolves.

To avoid hitting the GitHub API on every rebuild (and its rate limits), the
builder reuses the existing file while it is still fresh, deciding from its own
`generated_at` timestamp: the fetch is skipped when the file is younger than the
cache TTL — **10 hours on CI** (when the `CI` environment variable is set) and
**60 minutes** otherwise. CI additionally persists the file across runs with
`actions/cache` (keyed by a ~10-hour window) so a fresh copy is restored before
the build.

When it does run, the builder lists every repository the user owns
(`GET /users/{user}/repos`, paginated), drops the archived ones, the repositories
blacklisted in `CONFIG.blacklisted_repos`, and any repository with no update in
the last 365 days. It deserializes the rest directly into the shared
`portfolio_data::Repo`/`ReposFile` models and writes the pretty-printed JSON,
surfacing failures through a dedicated `UpdateReposError` model. An explicit
repository set can be requested at runtime via `GITHUB_REPOS` (comma-separated), in
which case each named repository is fetched directly (without filtering):

```bash
# Defaults: user = CONFIG.github_username, repos = all active repos
#           (archived/blacklisted/>1y-stale excluded),
#           output = apps/web/repos.json
GH_TOKEN=<token> cargo run --release -p update-repos -- apps/web/repos.json

# Override the repo set for a one-off run
GITHUB_REPOS=Portfolio,actions cargo run --release -p update-repos
```

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](./CONTRIBUTING.md) for the
development setup and the checks that must pass before opening a pull request.

## Security

To report a vulnerability, please follow the process described in
[SECURITY.md](./SECURITY.md).

## License

Proprietary. See [LICENSE](./LICENSE). The bundled Liberation Sans fonts are
licensed under the SIL OFL (see `apps/resume-generator/fonts/LICENSE`).
