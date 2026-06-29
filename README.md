# Portfolio

Personal portfolio at [tim-schoenle.de](https://tim-schoenle.de).

Built with **Rust + Yew (WASM)** and **Tailwind CSS**. A static SPA compiled to
WebAssembly, served by a tiny static Axum server in a scratch container.

## Workspace

| Crate | Purpose |
| --- | --- |
| `crates/data` | Shared language-neutral data (config, skills, experience, repos schema) + embedded `i18n/{en,de}.json` translations |
| `apps/frontend` | Yew 0.22 CSR app (Trunk build) |
| `apps/server` | Axum static server with SPA fallback, security headers, `/api/health` |
| `apps/resume-generator` | Generates `resume/{en,de}.pdf` + `resume-fingerprint.json` (Typst, embedded subset of Liberation Sans); the fingerprint is embedded into the frontend at build time. On CI each PDF is signed keylessly via [pdf-sign](https://github.com/0x77dev/pdf-sign) (Sigstore) and the signer identity is recorded in the fingerprint |
| `apps/site-meta` | Build-time generator for the frontend's static metadata (`head.html`, `robots.txt`, `sitemap.xml`, web manifest) from `CONFIG` |
| `apps/update-repos` | Builder that fetches all of the user's active GitHub repos (skipping archived, blacklisted and >1-year-stale ones) and refreshes `apps/frontend/repos.json` using the shared `Repo`/`ReposFile` models |

## Stack

- **Frontend:** Rust, [Yew](https://yew.rs) 0.22 (WASM CSR), [yew-router](https://crates.io/crates/yew-router)
- **i18n:** EN/DE via [i18nrs](https://crates.io/crates/i18nrs); translations live in
  `crates/data/i18n/`, language is persisted in `localStorage` (`lang`) and
  detected from the browser language on first visit. A unit test in
  `crates/data` enforces key parity between both languages.
- **Styling:** Tailwind CSS v3 + custom design tokens
- **Build:** [Trunk](https://trunkrs.dev)
- **Data:** `apps/frontend/repos.json` regenerated at build time by a Trunk
  hook from the GitHub API (all of the user's active repositories — archived,
  blacklisted and >1-year-stale repos excluded) and embedded into the WASM
  binary at build time (alongside `resume-fingerprint.json`) instead of being
  fetched at runtime. The result is cached by its `generated_at` timestamp (10h
  on CI, 60min locally) to stay under the GitHub API rate limits; the build
  fails if it cannot be (re)generated

## Features

Implements the "Portfolio v4" design (editorial grid with sticky label rails):

- Routes: `/` (single-page sections `s1`–`s5`), `/imprint`, `/privacy`, 404
- Hero with two-line display name, scroll parallax and live meta card
- Fixed chapter rail with scroll tracking; staggered reveal-on-scroll blocks
- Stack section: interactive skill radar (per-skill hover tooltips, category
  filtering/dimming) + chip matrix with confidence bars
- Projects: GitHub stats strip, language filter, loading skeletons fed by `repos.json`
- Experience accordion with year badges and animated bodies (real career data)
- Contact: terminal with type-in `ssh` animation, oversized email, action buttons
- Command palette (⌘K / Ctrl+K) with fuzzy search and keyboard navigation
- Language switcher (EN/DE) — every visible string is translated
- Localized, single-page, ATS-readable resume PDFs (`Tim-Schönle-Resume.pdf`,
  `Tim-Schönle-Lebenslauf.pdf`) with SHA-256 fingerprints on the contact card;
  the generator scales typography down until the content fits one A4 page. On CI
  the PDFs are signed keylessly through Sigstore (via
  [pdf-sign](https://github.com/0x77dev/pdf-sign)) and the signer identity is
  shown next to the fingerprint in the contact card's info popup
- Legal pages (imprint, privacy policy) localized and rendered from the translation files
- SEO: meta/OG tags, JSON-LD, `robots.txt`, `sitemap.xml`, web manifest
- Security headers (CSP, HSTS, …) set by the server
- WASM binary tuned for size: `opt-level = "z"`, LTO, single codegen unit

## Development

```bash
# resume PDFs + resume-fingerprint.json (run BEFORE the frontend build so the
# fingerprint is embedded and the PDFs are served via index.html's copy-dir)
cargo run -p resume-generator -- apps/frontend/generated

# frontend (http://localhost:8080)
cd apps/frontend
npm install
trunk serve

# tests (incl. i18n key parity)
cargo test -p portfolio-data
```

### Production build

```bash
cargo run --release -p resume-generator -- apps/frontend/generated
cd apps/frontend && npm install && trunk build --release
```

Or build the container, which does all of the above and serves the result on
port 8080:

```bash
docker build -t portfolio .
```

## repos.json

The projects section reads `repos.json`, embedded into the WASM binary at build
time via `include_str!`. It is **generated during the build**: the `update-repos`
builder runs as a Trunk `pre_build` hook (see `apps/frontend/Trunk.toml`) and
refreshes `apps/frontend/repos.json` before the WASM is compiled. If it cannot be
generated, the build fails.

To avoid hitting the GitHub API on every rebuild (and its rate limits), the hook
reuses the existing file while it is still fresh, deciding from its own
`generated_at` timestamp: the fetch is skipped when the file is younger than the
cache TTL — **10 hours on CI** (`CI` env var set) and **60 minutes** otherwise.
CI additionally persists the file across runs with `actions/cache` (keyed by a
~10h window) so a fresh copy is restored before the build.

When it does run, the builder lists every repository the user owns
(`GET /users/{user}/repos`, paginated), drops the archived ones, the repos
blacklisted in `CONFIG.blacklisted_repos` and any repo with no update in the last
365 days, deserializes the rest directly into the shared
`portfolio_data::Repo`/`ReposFile` models and writes the pretty-printed JSON via
a dedicated `UpdateReposError` model. An explicit repository set can be requested
at runtime via `GITHUB_REPOS` (comma-separated), in which case each named repo is
fetched directly (without filtering):

```bash
# defaults: user = CONFIG.github_username, repos = all active repos
#           (archived/blacklisted/>1y-stale excluded),
#           output = apps/frontend/repos.json
GH_TOKEN=<token> cargo run --release -p update-repos -- apps/frontend/repos.json

# override the repo set for a one-off run
GITHUB_REPOS=Portfolio,actions cargo run --release -p update-repos
```

## Resume signing

The resume PDFs are signed keylessly with **Sigstore** via
[pdf-sign](https://github.com/0x77dev/pdf-sign), which appends the signature
after the PDF's `%%EOF` (the file stays a valid, readable PDF).

Signing is **opt-in via the `SIGSTORE_IDENTITY_TOKEN` environment variable** and
only happens when it is set, so local builds need no token, network access or
`pdf-sign` binary. On CI the workflow installs `pdf-sign`, mints a GitHub Actions
OIDC token (audience `sigstore`, requiring `id-token: write`) and passes it to
the generator. Each PDF is then signed *before* it is hashed, so the SHA-256
fingerprint shown on the contact card always matches the signed download. The
signer identity (the CI workflow ref) and OIDC issuer are recorded in
`resume-fingerprint.json` and shown next to the fingerprint in the contact
card's info popup.

Verify a downloaded resume against the published identity:

```bash
pdf-sign verify Tim-Schönle-Resume.pdf \
  --certificate-identity https://github.com/<owner>/<repo>/.github/workflows/ci.yaml@refs/heads/main \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com
```

## License

Proprietary. See [LICENSE](./LICENSE). Bundled Liberation Sans fonts are
licensed under the SIL OFL (see `apps/resume-generator/fonts/LICENSE`).
