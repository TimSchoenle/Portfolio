# Portfolio

Personal portfolio at [tim-schoenle.de](https://tim-schoenle.de).

A static single-page application written in **Rust + Yew (WebAssembly)** and
**Tailwind CSS**, compiled to WASM and served by a minimal static **Axum** server
running from a `scratch` container.

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
- [Resume Signing](#resume-signing)
- [Contributing](#contributing)
- [Security](#security)
- [License](#license)

## Architecture

The project is a Cargo workspace composed of one shared library crate and five
applications.

| Crate | Purpose |
| --- | --- |
| `crates/data` | Shared, language-neutral data (config, skills, experience, repos schema) plus embedded `i18n/{en,de}.json` translations |
| `apps/frontend` | Yew 0.22 client-side-rendered (CSR) application built with Trunk |
| `apps/server` | Axum static server with SPA fallback, security headers, and health/liveness/readiness probe endpoints |
| `apps/resume-generator` | Generates `resume/{en,de}.pdf` and `resume-fingerprint.json` (Typst, embedded subset of Liberation Sans). On CI each PDF is signed keylessly via [pdf-sign](https://github.com/0x77dev/pdf-sign) (Sigstore) and the signer identity is recorded in the fingerprint |
| `apps/site-meta` | Build-time generator for the frontend's static metadata (`head.html`, `robots.txt`, `sitemap.xml`, web manifest) from `CONFIG` |
| `apps/update-repos` | Builder that fetches the user's active GitHub repositories (skipping archived, blacklisted, and >1-year-stale ones) and refreshes `apps/frontend/repos.json` using the shared `Repo`/`ReposFile` models |

## Technology Stack

- **Frontend:** Rust, [Yew](https://yew.rs) 0.22 (WASM CSR), [yew-router](https://crates.io/crates/yew-router)
- **Internationalization:** EN/DE via [i18nrs](https://crates.io/crates/i18nrs).
  Translations live in `crates/data/i18n/`; the active language is persisted in
  `localStorage` (`lang`) and detected from the browser language on first visit.
  A unit test in `crates/data` enforces key parity between both languages.
- **Styling:** Tailwind CSS v3 with custom design tokens
- **Build:** [Trunk](https://trunkrs.dev)
- **Data:** `apps/frontend/repos.json` is regenerated at build time by a Trunk
  hook from the GitHub API (all active repositories — archived, blacklisted

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
  generator scales typography down until the content fits one A4 page. On CI the
  PDFs are signed keylessly through Sigstore (via
  [pdf-sign](https://github.com/0x77dev/pdf-sign)), and the signer identity is
  shown next to the fingerprint in the contact card's info popup.
- Legal pages (imprint, privacy policy) localized and rendered from the translation files
- SEO: meta/OG tags, JSON-LD, `robots.txt`, `sitemap.xml`, and a web manifest
- Security headers (CSP, HSTS, and others) set by the server
- WASM binary tuned for size: `opt-level = "z"`, LTO, and a single codegen unit

## Getting Started

### Prerequisites

- Rust (stable) with the `wasm32-unknown-unknown` target:
  `rustup target add wasm32-unknown-unknown`
- [Trunk](https://trunkrs.dev): `cargo install trunk` (or `cargo binstall trunk`)
- Node.js (required only for the Tailwind CSS build step)
- Docker (optional, for containerized builds)

### Development

```bash
# Resume PDFs + resume-fingerprint.json. Run BEFORE the frontend build so the
# fingerprint is embedded and the PDFs are served via index.html's copy-dir.
cargo run -p resume-generator -- apps/frontend/generated

# Frontend dev server (http://localhost:8080)
cd apps/frontend
npm install
trunk serve

# Tests (including i18n key parity)
cargo test -p portfolio-data
```

### Production Build

```bash
cargo run --release -p resume-generator -- apps/frontend/generated
cd apps/frontend && npm install && trunk build --release
```

### Container

The container performs all of the above and serves the result on port 8080:

```bash
docker build -t portfolio .
```

## Deployment

The image is built on a `scratch` base holding a single statically linked (musl)
`server` binary plus the read-only `/dist` assets — no shell, package manager, or
writable system paths. The Helm chart lives in a separate repository; the
application and image here are prepared to run under a hardened pod spec out of
the box.

### Probe Endpoints

| Endpoint | Alias | Purpose |
| --- | --- | --- |
| `GET /api/health` | — | General health report with the current UTC time |
| `GET /api/health/live` | `GET /livez` | **Liveness** — process is running; failure restarts the container |
| `GET /api/health/ready` | `GET /readyz` | **Readiness** — the SPA (`$DIST_DIR/index.html`) is present and servable; failure removes the pod from the Service endpoints |

All probe responses are `no-store` (never cached). Readiness returns `503` until
the assets are present.

### Read-only and Security Posture

The server only reads from `$DIST_DIR` and writes logs to stdout, so it runs
unchanged with a fully read-only root filesystem (no writable volume, not even
`/tmp`). It is built to satisfy the restricted Pod Security Standard:

- runs as numeric non-root `1001:1001` (so `runAsNonRoot` verifies statically)
- `readOnlyRootFilesystem: true`, `allowPrivilegeEscalation: false`
- all Linux capabilities dropped, `seccompProfile: RuntimeDefault`
- HTTP security headers (CSP, HSTS, `X-Content-Type-Options`, `X-Frame-Options`,
  `Referrer-Policy`, `Permissions-Policy`) set on every response
- listens on `:$PORT` (default `8080`); graceful shutdown on `SIGTERM`

Configuration is provided via environment variables: `DIST_DIR` (default `/dist`),
`PORT` (default `8080`), and `RUST_LOG` (default `info`).

### Reproducible Builds

The build is pinned end-to-end so the image is reproducible:

- base images pinned by digest; Rust toolchain pinned via the base image
- `trunk` and `cargo-chef` pinned to exact versions (build args `TRUNK_VERSION`
  and `CARGO_CHEF_VERSION`)
- `cargo build` / `cargo chef cook` run `--locked` against the committed
  `Cargo.lock`; the frontend uses `npm ci` against `package-lock.json`
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

The projects section reads `repos.json`, which is embedded into the WASM binary at
build time via `include_str!`. The file is **generated during the build**: the
`update-repos` builder runs as a Trunk `pre_build` hook (see
`apps/frontend/Trunk.toml`) and refreshes `apps/frontend/repos.json` before the
WASM is compiled. If it cannot be generated, the build fails.

To avoid hitting the GitHub API on every rebuild (and its rate limits), the hook
reuses the existing file while it is still fresh, deciding from its own
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
#           output = apps/frontend/repos.json
GH_TOKEN=<token> cargo run --release -p update-repos -- apps/frontend/repos.json

# Override the repo set for a one-off run
GITHUB_REPOS=Portfolio,actions cargo run --release -p update-repos
```

## Resume Signing

The resume PDFs are signed keylessly with **Sigstore** via
[pdf-sign](https://github.com/0x77dev/pdf-sign), which appends the signature after
the PDF's `%%EOF` (the file remains a valid, readable PDF).

Signing is **opt-in via the `SIGSTORE_IDENTITY_TOKEN` environment variable** and
only happens when it is set, so local builds need no token, network access, or
`pdf-sign` binary. Only the production release build signs: the
`release-please.yaml` workflow mints a GitHub Actions OIDC token (audience
`sigstore`, requiring `id-token: write`) and passes it as the `sigstore_token`
build secret to the Docker build, where `pdf-sign` is installed and the generator
signs each PDF *before* it is hashed, so the SHA-256 fingerprint shown on the
contact card always matches the signed download. PR/test images are built without
the token and stay unsigned. The signer identity (the release workflow ref) and
OIDC issuer are recorded in `resume-fingerprint.json` and shown next to the
fingerprint in the contact card's info popup.

Verify a downloaded resume against the published identity:

```bash
pdf-sign verify Tim-Schönle-Resume.pdf \
  --certificate-identity https://github.com/<owner>/<repo>/.github/workflows/release-please.yaml@refs/heads/main \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com
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
