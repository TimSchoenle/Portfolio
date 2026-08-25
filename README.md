<!--
Generated from .github/templates/README.md.hbs. Edit that file, never this one.

Two payloads are merged into one render. The repository's own facts — name, description, version,
licence, toolchain and the docs/ index — come from
TimSchoenle/actions/actions/common/readme-variables, which reads the root Cargo.toml and walks
docs/. It deliberately does not read the description from the GitHub API: one edited in the web UI
would change this file with no commit behind it, and the next unrelated pull request would fail the
drift gate for a reason absent from its own diff.

The configuration half comes from one command:

    cargo run -q -p portfolio-config --features config-schema \
      --example config-schema -- --format variables

which walks the `Describe` derives on `ServerConfig` and `BuilderConfig`, the aggregates the two
binaries actually load, and prints both tables, the example configuration and the spellings the
prose below names. The two tables are two on purpose: one list of every key reads as though a
deployment needs a GitHub token, and it does not, because `github.*` belongs to a build-time tool
that exits during the image build.

The Update Files workflow renders this on every pull request and commits the result back to the
branch. The README job in Build re-renders with `check: true` and fails when the committed file
does not match it, so a README edited by hand does not merge.

Prefer an injected value to a typed one anywhere the two would say the same thing. A rename in
`crates/config` then reaches the sentences as well as the tables, and a key the prose names and the
types no longer have fails the generator instead of rendering a blank. `KEYS` in
`crates/config/examples/config-schema.rs` is where a key joins that set.

Nothing in this comment may contain a mustache that is not a real reference.
-->

# Portfolio

Dioxus fullstack (SSR + hydration) portfolio served by Axum.

[![Release](https://img.shields.io/github/v/release/TimSchoenle/Portfolio?sort=semver)](https://github.com/TimSchoenle/Portfolio/releases)
[![Build](https://img.shields.io/github/actions/workflow/status/TimSchoenle/Portfolio/build.yaml?branch=main)](https://github.com/TimSchoenle/Portfolio/actions/workflows/build.yaml)
[![License](https://img.shields.io/badge/license-LicenseRef--Proprietary-blue)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.97-orange)](https://www.rust-lang.org)

## What this is

The source of <https://tim-schoenle.de>, as one Rust workspace.

`apps/web` is a single crate with two feature-selected builds. The `server` build is a native Axum
binary that renders every route. The `web` build is a WASM bundle that hydrates it in the browser.
The Dioxus CLI produces both.

The configuration tables below are generated, not written. They come out of the Rust types that
load the configuration, as do `config.example.toml` and the contract document the image publishes
about itself, so renaming a field corrects all three in the commit that renames it.

## Quick start

```bash
docker run --rm -p 8080:8080 timschoenle/portfolio:v2.7.1
```

Then open <http://localhost:8080>. The runtime image is `FROM scratch`: one statically linked
binary, the client bundle beside it, and the configuration contract at `/config/contract.json`. It
runs as `1001:1001` and needs nothing writable.

## Table of contents

- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
- [Configuration](#configuration)
- [Operations](#operations)
- [Compatibility](#compatibility)
- [Documentation](#documentation)
- [Contributing](#contributing)
- [Security](#security)
- [License](#license)

## Features

- Every route renders on the server and then hydrates. Per-route `<head>` metadata and JSON-LD ship
  in the first response, and the locale is negotiated from request headers before the document is
  serialised, so nothing arrives in the wrong language and gets swapped a moment later.
- EN and DE throughout, including both legal pages and both resumes. `translation_key_sets_match`
  in `crates/data` fails the build when the two translation files disagree on a key, which
  otherwise shows up as one English string in a German page rather than as an error.
- **The resumes are typeset during the build.** Typst lays out one A4 page per language, scaling
  the type down and re-typesetting until the content fits, and each PDF carries a SHA-256
  fingerprint that the contact card shows.
- **The Content-Security-Policy is built per response.** The server hashes the inline scripts that
  document actually carries and reserves a nonce for the script Cloudflare injects at the edge, so
  `script-src` needs no `'unsafe-inline'`.
- `/licenses` reproduces the licence of every crate the client and the server link. The notices are
  grouped by licence and each distinct text is reproduced once, naming the dependencies that ship
  it, with a flat inventory beneath listing every dependency and the terms it offers. cargo-about
  produces the data during the image build, and an unlisted licence fails that build.
- The project list is fetched from the GitHub API at build time, with archived, blacklisted and
  year-stale repositories dropped, then embedded into the binary by `build.rs`.
- Cmd+K opens a command palette with fuzzy search and keyboard navigation. The stack section is a
  skill radar with per-skill tooltips and category filtering.

## Installation

### Docker

```bash
docker pull timschoenle/portfolio:v2.7.1
```

Both architectures are pushed as one manifest list, so `docker pull` resolves the right image per
node. The push carries an SBOM and max-mode provenance and is signed with cosign. Pin by digest in
production. The Helm chart does.

### Helm

```bash
helm repo add timschoenle https://timschoenle.github.io/helm-charts
helm install portfolio timschoenle/portfolio
```

The chart is in
[TimSchoenle/helm-charts](https://github.com/TimSchoenle/helm-charts/tree/main/charts/portfolio) and
each release bumps it to the new image digest.

### From source

```bash
rustup target add wasm32-unknown-unknown
cargo install --locked dioxus-cli cargo-about
git clone https://github.com/TimSchoenle/Portfolio.git
cd Portfolio
```

Node.js is needed for the Tailwind step, and `just` runs every recipe CI runs.

## Usage

Two artefacts have to exist before the web build, because `apps/web/build.rs` embeds both. Without
them it substitutes empty defaults and the pages render their empty state.

```bash
cargo run -p resume-generator -- apps/web/generated   # resume PDFs, fingerprints, social card
just licenses                                         # third-party inventory for /licenses
cd apps/web && npm ci && npm run build:css
dx serve --platform web                               # SSR + hydration on http://localhost:8080
```

Run the checks CI runs, in one recipe:

```bash
just verify   # fmt, lint, test
```

Refresh the project list from the GitHub API:

```bash
PORTFOLIO_GITHUB__TOKEN_FILE=/run/secrets/gh_token \
  cargo run --release -p update-repos -- apps/web/repos.json
```

Without a token the run still works, against the anonymous rate limit.
[docs/PROJECT_DATA.md](docs/PROJECT_DATA.md) has the filtering rules and the caching.

## Configuration

Values are resolved in five layers, each overriding the one above it. The last three are mutually
exclusive per key: a key supplied by two of them fails the boot rather than letting one win,
because a stale environment variable shadowing a rotated mounted secret keeps the process running
on the old credential.

1. **Defaults** — the `serde` defaults of each typed block.
2. **TOML** at `$PORTFOLIO_CONFIG` — a file, or every `*.toml` inside it when it names a directory.
3. **Environment** — `PORTFOLIO_`-prefixed variables, `__` for nesting.
4. **Secrets directory** at `$PORTFOLIO_SECRETS_DIR` — one file per key, named after it
   (`github__token`). This is what a Kubernetes `Secret` volume mounts.
5. **File indirection** — `PORTFOLIO_<KEY>_FILE=/path` names a file holding the value.

An empty value counts as unset in every layer, because container platforms routinely inject `KEY=`
for a declared-but-unset variable.

### Server

The two variables the loader reads before any layer exists, then every key `apps/web` loads. Each
environment spelling also accepts a `_FILE` suffix naming a file that holds the value.

| Variable | Role | Default | Purpose |
|---|---|---|---|
| `PORTFOLIO_CONFIG` | config | `config.toml` | Names the TOML layer: a file, or a directory whose `*.toml` files are all merged in name order. |
| `PORTFOLIO_SECRETS_DIR` | secrets dir | — | Names a directory of key-named files — a mounted Kubernetes `Secret` volume. Each file supplies the key its name spells. |

| TOML | Type | Environment | Default | Flags | Purpose |
|---|---|---|---|---|---|
| `assets.dist_dir` | `PathBuf` | `PORTFOLIO_ASSETS__DIST_DIR` | `public` | — | Directory holding the `dx bundle` output, relative to the working directory. |
| `csp.hash_inline_scripts` | `bool` | `PORTFOLIO_CSP__HASH_INLINE_SCRIPTS` | `true` | — | Hash every inline `<script>` in the document being served instead of admitting all inline script with `'unsafe-inline'`. |
| `csp.cloudflare.script_nonce` | `bool` | `PORTFOLIO_CSP__CLOUDFLARE__SCRIPT_NONCE` | `true` | — | Reserve a per-response nonce in `script-src` for the script Cloudflare injects at the edge. |
| `csp.cloudflare.turnstile` | `bool` | `PORTFOLIO_CSP__CLOUDFLARE__TURNSTILE` | `false` | — | Admit `https://challenges.cloudflare.com` in `script-src` and `frame-src`, for a Turnstile widget. |
| `csp.cloudflare.web_analytics` | `bool` | `PORTFOLIO_CSP__CLOUDFLARE__WEB_ANALYTICS` | `false` | — | Admit the Cloudflare Web Analytics beacon and the endpoint it reports to. |
| `isr.cache_dir` | `PathBuf` | `PORTFOLIO_ISR__CACHE_DIR` | unset (ISR off; the image sets `/tmp/isr`) | — | Writable directory rendered HTML is cached into. Unset or empty disables ISR. |
| `isr.ttl_secs` | `u64` | `PORTFOLIO_ISR__TTL_SECS` | `0` (permanent) | — | Revalidation interval in seconds. Zero means a permanent cache. |

### Builder

What `update-repos` loads. It runs during the image build and exits. The server never reads these
keys, so a deployment needs no GitHub token. That is why there are two tables and not one.

| TOML | Type | Environment | Default | Flags | Purpose |
|---|---|---|---|---|---|
| `github.username` | `String` | `PORTFOLIO_GITHUB__USERNAME` | unset (the site's own `CONFIG.github_username`) | — | User whose repositories to list. |
| `github.token` | `SecretString` | `PORTFOLIO_GITHUB__TOKEN` | unset | secret | Bearer token lifting the GitHub API rate limit. |
| `github.repos` | `Vec<String>` | `PORTFOLIO_GITHUB__REPOS` | `[]` (every active repository the user owns) | — | Explicit repository set, bypassing the "every active repository" listing and its filtering. |

`github.token` is the only secret in the workspace, and it should arrive as a file
rather than as an environment variable. `/proc/<pid>/environ`, a crash dump and `docker inspect` all
carry the environment, and child processes inherit it. The Docker build mounts it as a BuildKit
secret and hands `update-repos` the path.

`IP`, `PORT` and `RUST_LOG` sit outside this namespace deliberately. They are the Dioxus toolchain's
contract with the binary, and it keeps reading them itself.

[`config.example.toml`](config.example.toml) carries every key at its default, commented out, and is
rendered from the same payload as these tables.

## Operations

### Probes

| Endpoint | Alias | Purpose |
| --- | --- | --- |
| `GET /api/health` | — | Health report with the current UTC time |
| `GET /api/health/live` | `GET /livez` | **Liveness** — the process is running; failure restarts the container |
| `GET /api/health/ready` | `GET /readyz` | **Readiness** — `index.html` under `assets.dist_dir` (default `public/`) is present and servable; failure removes the pod from the Service endpoints |

All probe responses are `no-store`. Readiness returns `503` until the assets are there.

### Runtime posture

The server reads its bundle directory and writes to stdout. It runs as numeric non-root
`1001:1001`, so `runAsNonRoot` verifies statically, and it satisfies the restricted Pod Security
Standard with a read-only root filesystem, no privilege escalation, every capability dropped and
`seccompProfile: RuntimeDefault`.

Incremental static regeneration is the one thing that wants a writable path. The image points the
cache at `/tmp/isr` and bakes an empty one owned by the runtime user; where that path is not
writable, every request is rendered fresh instead.

## Compatibility

| | Supported |
| --- | --- |
| Rust | 1.97 (edition 2024) |
| Platforms | `linux/amd64`, `linux/arm64` |
| Pod Security Standard | restricted |

## Documentation

| Document | Purpose |
| --- | --- |
| [Architecture](docs/ARCHITECTURE.md) | Six packages in one Cargo workspace: two libraries every binary reads, three binaries, and a placeholder at the root that exists so release-please has a version to move. |
| [Deployment](docs/DEPLOYMENT.md) | What the image contains, how it is built reproducibly, what it publishes about its own configuration, and where the chart that runs it lives. |
| [Project data](docs/PROJECT_DATA.md) | How apps/web/repos.json is fetched, filtered, cached and embedded, and what the projects section reads out of it. |
| [Security posture](docs/SECURITY_POSTURE.md) | The Content-Security-Policy the server builds per response, the headers around it, and what the process needs from the filesystem it runs on. |
| [docs/config.contract.json](docs/config.contract.json) | — |

## Contributing

[CONTRIBUTING.md](CONTRIBUTING.md) has the development setup, the commit convention and the checks
a pull request has to pass. Note what the licence grants before you fork: reading the source and
running it locally, and nothing about redistributing or deploying it.

Several files here are generated, this one included. Each says so in its opening lines, and an edit
made to the output instead of to the template is replaced by the next render.

## Security

Do not open a public issue for a vulnerability. [SECURITY.md](SECURITY.md) has the reporting route
and the supported versions.

## License

`LicenseRef-Proprietary`. Viewing the source and running it locally for personal, non-commercial
evaluation is granted. Copying, modifying, redistributing, deploying and training on it are not.
[LICENSE](LICENSE) has the terms. The bundled fonts keep their own: Inter and Liberation Sans are
both under the SIL Open Font License, in `apps/resume-generator/fonts/`.
