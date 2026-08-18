<!--
Template for the repository README. CI renders it on every pull request and commits the result
to README.md, so edit this file — never README.md itself.

Two variables, `serverConfigTable` and `builderConfigTable`, one per binary, from

    cargo run -p portfolio-config --features config-schema --example config-schema \
      -- --format markdown --scope server
    cargo run -p portfolio-config --features config-schema --example config-schema \
      -- --format markdown --scope builder

which walk the `Describe` derives on `ServerConfig` and `BuilderConfig` — the aggregates the two
binaries actually load — and emit each key with its TOML path, type, environment spelling,
default and purpose. The server scope leads with the variables the loader reads before any layer
exists; the builder scope renders keys alone, because they are the same two variables. Both are
interpolated through a triple-stash, being Markdown to emit as-is rather than escape.

Neither table has a `_FILE` column: it is the environment spelling plus a constant suffix, which
the layer list above the tables already states once.

They are two tables and not one on purpose. A single list of every key reads as though a
deployment needs a GitHub token; it does not, because `github.*` belongs to a build-time tool
that exits during the image build.

That is what keeps the configuration reference honest across a rename: the key table is not a
copy of the types, it is the types, and a pull request that renames a field arrives with the
README already saying so. `.github/workflows/update-files.yaml` is the job that does it.

This is an HTML comment, not a template one, so Handlebars parses it like any other line and it
survives into README.md — which is the point, since that is where someone about to edit the
wrong file will be. Nothing in it may contain a mustache that is not a real reference.
-->
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
- [Configuration](#configuration)
- [Deployment](#deployment)
  - [Probe Endpoints](#probe-endpoints)
  - [Read-only and Security Posture](#read-only-and-security-posture)
  - [Content-Security-Policy](#content-security-policy)
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
| `crates/config` | The typed configuration blocks every binary reads, plus the Portfolio dialect of the [terrace-config](https://github.com/TimSchoenle/terrace-config) layered loader (see [Configuration](#configuration)) |
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
- Security headers set by the server: HSTS, `Referrer-Policy`, `Permissions-Policy`,
  and a Content-Security-Policy built per document from the inline scripts it
  actually carries, with a per-response nonce for Cloudflare's edge injection
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

## Configuration

Configuration is **layered and file-first**, via
[terrace-config](https://github.com/TimSchoenle/terrace-config). `crates/config`
owns the typed blocks and the `PORTFOLIO_` dialect; each binary declares the
aggregate it actually reads. Sources are merged in this order, lowest precedence
first:

1. **Struct defaults** — the `serde` defaults in `crates/config`.
2. **TOML** at `$PORTFOLIO_CONFIG` — a file, or every `*.toml` inside it when it
   names a directory (so a `ConfigMap` can be split into fragments).
3. **Environment** — `PORTFOLIO_`-prefixed variables, `__` for nesting.
4. **Secrets directory** at `$PORTFOLIO_SECRETS_DIR` — one file per key, named
   after it (`github__token`); this is what a Kubernetes `Secret` volume mounts.
5. **File indirection** — `PORTFOLIO_<KEY>_FILE=/path` names a file holding the
   value.

The last three are **mutually exclusive per key**: a key supplied by two of them
fails the boot instead of one silently winning, because a stale environment
variable shadowing a rotated mounted secret keeps the process running on the old
credential.

The tables below are **generated from the aggregates the binaries load** —
`ServerConfig` and `BuilderConfig` in `crates/config`. They are split the way the
binaries are, because the two configurations are not one: what a deployment sets
and what the image build sets have no overlap at all.

Regenerate them with:

```bash
cargo run -p portfolio-config --features config-schema --example config-schema \
  -- --format markdown --scope server
```

#### What the server reads

The two variables the loader itself reads, then every key `apps/web` loads. This
is the whole surface a **deployment** configures.

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

#### What the `update-repos` builder reads

`github.*` is read only by the build-time repository lister, which runs during the
image build and exits. **The server never loads it**, so a deployment needs no
GitHub token and no `github` block — the Docker build supplies the token as a
BuildKit secret and nothing survives into the image.

| TOML | Type | Environment | Default | Flags | Purpose |
|---|---|---|---|---|---|
| `github.username` | `String` | `PORTFOLIO_GITHUB__USERNAME` | unset (the site's own `CONFIG.github_username`) | — | User whose repositories to list. |
| `github.token` | `SecretString` | `PORTFOLIO_GITHUB__TOKEN` | unset | secret | Bearer token lifting the GitHub API rate limit. |
| `github.repos` | `Vec<String>` | `PORTFOLIO_GITHUB__REPOS` | `[]` (every active repository the user owns) | — | Explicit repository set, bypassing the "every active repository" listing and its filtering. |

The `config-schema` feature is off in every build that ships, so the derive and
the `serde_json` it pulls stay out of the image; CI's `--all-features` gates are
what keep the generator compiling.

An empty value counts as unset everywhere, because container platforms routinely
inject `KEY=` for a declared-but-unset variable. See
[`config.example.toml`](./config.example.toml) for a commented starting point.

`IP`, `PORT` and `RUST_LOG` are deliberately **outside** this namespace: they are
the Dioxus toolchain's contract with the binary (`dx serve` sets them to tell a
development build which port it is proxied on), so the framework keeps reading
them itself. `CI` likewise belongs to the CI provider.

### Secrets

`github.token` is the only secret in the workspace, and it should arrive as a
**file**, never as an environment variable — `/proc/<pid>/environ`, a crash dump
and `docker inspect` all carry the environment, and child processes inherit it:

```bash
PORTFOLIO_GITHUB__TOKEN_FILE=/run/secrets/gh_token cargo run --release -p update-repos
```

The Docker build already does this: the `gh_token` BuildKit secret is mounted as
a file and handed to `update-repos` by path. In the process it is a
`secrecy::SecretString`, so it has no `Debug` or `Display` and the one place it
becomes a `&str` is the `Authorization` header.

Only the loader half of `terrace-config` is used; its hot-reload supervisor is
not. The reasoning — the server holds no secrets, and `dioxus::serve` owns an
accept loop with no shutdown handle — is recorded in `crates/config/src/lib.rs`.

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
| `GET /api/health/ready` | `GET /readyz` | **Readiness** — the client bundle (`index.html` under `assets.dist_dir`, default `public/`) is present and servable; failure removes the pod from the Service endpoints |

All probe responses are `no-store` (never cached). Readiness returns `503` until
the assets are present.

### Read-only and Security Posture

The server only reads from its bundle directory (the sibling `public/` dir) and
writes logs to stdout, so it runs unchanged with a fully read-only root
filesystem (no
writable volume, not even `/tmp`). It is built to satisfy the restricted Pod
Security Standard:

- runs as numeric non-root `1001:1001` (so `runAsNonRoot` verifies statically)
- `readOnlyRootFilesystem: true`, `allowPrivilegeEscalation: false`
- all Linux capabilities dropped, `seccompProfile: RuntimeDefault`
- HTTP security headers (CSP, HSTS, `X-Content-Type-Options`, `X-Frame-Options`,
  `Referrer-Policy`, `Permissions-Policy`) set on every response — see
  [Content-Security-Policy](#content-security-policy)
- listens on `:$PORT` (default `8080`); graceful shutdown on `SIGTERM`

Everything else the server reads comes from the layered configuration described
under [Configuration](#configuration), so a `ConfigMap` fragment or a `Secret`
volume is a first-class source rather than something a chart has to flatten into
environment variables.

### Content-Security-Policy

Built with [csp-shell](https://github.com/TimSchoenle/csp-shell) rather than
written out as a string, in `apps/web/src/server/csp.rs`. Two policies leave the
server:

- **Documents** get one derived from the bytes they carry. Dioxus renders its
  hydration data inline (`window.initial_dioxus_hydration_data="…"`), so its text
  is only known per response — the body is buffered anyway to stamp `<html lang>`,
  and each inline script is hashed out of that same string. `'unsafe-inline'` is
  therefore gone from `script-src`: an injected `<script>` no longer runs just
  because the hydration bootstrap has to.
- **Everything else** — assets, the JSON API, the SEO documents — gets a policy
  that admits no inline script at all, because none of them has any.

`'unsafe-eval'` remains, and only for the one reason it has ever been here:
`dioxus-web`'s document provider applies `document::Title`, `document::Stylesheet`
and friends through `new Function(…)`, which throws an uncaught `EvalError`
without it and freezes client-side navigation.

**Cloudflare.** The bot products in front of this origin (Bot Fight Mode,
JavaScript Detections, the challenge platform — the `_cf_bm`, `cf_clearance` and
`cf_chl_rc_*` cookies the privacy page lists) inject an inline `<script>` at the
edge, after this server has hashed what it rendered. No hash can cover it; a
nonce can, because Cloudflare parses the `Content-Security-Policy` response
header and copies the nonce onto what it injects. That is
`csp.cloudflare.script_nonce`, on by default, and it brings one obligation the
server discharges — every document is `Cache-Control: no-cache`, so a nonce is
never shared between readers — and one it cannot:

> **Deployment checklist:** no Cloudflare Cache Rule may cache the shell. A
> "Cache Everything" rule overrides the origin's `Cache-Control`, pinning one
> nonce across every reader for the lifetime of the cache entry, and nothing
> inside this process can see that happening.

Turnstile and Web Analytics are off; switching either on admits its origins in
the directives that product actually needs them in (Turnstile in `script-src`
**and** `frame-src` — admitting only the first renders an empty box).

**If a page ever renders blank**, that is the failure mode this design has:
`PORTFOLIO_CSP__HASH_INLINE_SCRIPTS=false` together with
`PORTFOLIO_CSP__CLOUDFLARE__SCRIPT_NONCE=false` restores `'unsafe-inline'` on a
restart rather than a redeploy. The two must move together — a browser ignores
`'unsafe-inline'` as soon as the policy carries a nonce — and supplying one
without the other fails the boot with a message saying so.

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
repository set can be requested via `github.repos`, in which case each named
repository is fetched directly (without filtering):

```bash
# Defaults: user = CONFIG.github_username, repos = all active repos
#           (archived/blacklisted/>1y-stale excluded),
#           output = apps/web/repos.json
PORTFOLIO_GITHUB__TOKEN_FILE=/run/secrets/gh_token \
  cargo run --release -p update-repos -- apps/web/repos.json

# Override the repo set for a one-off run (figment's bracketed array syntax,
# so the environment and the TOML spelling stay the same shape)
PORTFOLIO_GITHUB__REPOS='[Portfolio,actions]' cargo run --release -p update-repos
```

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](./CONTRIBUTING.md) for the
development setup and the checks that must pass before opening a pull request.

**This file is generated.** Edit
[`.github/templates/README.md.hbs`](./.github/templates/README.md.hbs) instead —
CI renders it on every pull request and commits the result back to the branch, so
there is no toolchain to install locally.

## Security

To report a vulnerability, please follow the process described in
[SECURITY.md](./SECURITY.md).

## License

Proprietary. See [LICENSE](./LICENSE). The bundled Liberation Sans fonts are
licensed under the SIL OFL (see `apps/resume-generator/fonts/LICENSE`).
