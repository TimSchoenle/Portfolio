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
| `apps/resume-generator` | Generates `resume/{en,de}.pdf` + `resume-fingerprint.json` (genpdf, embedded subset of Liberation Sans); the fingerprint is embedded into the frontend at build time |
| `apps/site-meta` | Build-time generator for the frontend's static metadata (`head.html`, `robots.txt`, `sitemap.xml`, web manifest) from `CONFIG` |
| `apps/update-repos` | Builder that fetches the configured GitHub repos (`CONFIG.repos`) and refreshes `apps/frontend/repos.json` using the shared `Repo`/`ReposFile` models |

## Stack

- **Frontend:** Rust, [Yew](https://yew.rs) 0.22 (WASM CSR), [yew-router](https://crates.io/crates/yew-router)
- **i18n:** EN/DE via [i18nrs](https://crates.io/crates/i18nrs); translations live in
  `crates/data/i18n/`, language is persisted in `localStorage` (`lang`) and
  detected from the browser language on first visit. A unit test in
  `crates/data` enforces key parity between both languages.
- **Styling:** Tailwind CSS v3 + custom design tokens
- **Build:** [Trunk](https://trunkrs.dev)
- **Data:** `apps/frontend/repos.json` rebuilt daily by GitHub Actions from the
  GitHub API (only the repos listed in `CONFIG.repos`) and embedded into the
  WASM binary at build time (alongside `resume-fingerprint.json`) instead of
  being fetched at runtime

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
  the generator scales typography down until the content fits one A4 page
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
time via `include_str!`. A placeholder is committed at
`apps/frontend/repos.json`; the `update-repos.yml` workflow refreshes it daily
from the GitHub API and commits the result, so the next build picks it up.

The refresh runs the `update-repos` builder, which fetches each repository listed
in `CONFIG.repos` by name (one `GET /repos/{user}/{name}` request each),
deserializes them directly into the shared `portfolio_data::Repo`/`ReposFile`
models and writes the pretty-printed JSON via a dedicated `UpdateReposError`
model. The repository set is configured centrally in `CONFIG.repos` and can be
overridden at runtime via `GITHUB_REPOS` (comma-separated):

```bash
# defaults: user = CONFIG.github_username, repos = CONFIG.repos,
#           output = apps/frontend/repos.json
GH_TOKEN=<token> cargo run --release -p update-repos -- apps/frontend/repos.json

# override the repo set for a one-off run
GITHUB_REPOS=Portfolio,actions cargo run --release -p update-repos
```

## License

Proprietary. See [LICENSE](./LICENSE). Bundled Liberation Sans fonts are
licensed under the SIL OFL (see `apps/resume-generator/fonts/LICENSE`).
