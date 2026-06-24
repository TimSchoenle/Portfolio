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
| `apps/resume-generator` | Generates `resume/{en,de}.pdf` + `resume-fingerprint.json` (genpdf, embedded subset of Liberation Sans) |

## Stack

- **Frontend:** Rust, [Yew](https://yew.rs) 0.22 (WASM CSR), [yew-router](https://crates.io/crates/yew-router)
- **i18n:** EN/DE via [i18nrs](https://crates.io/crates/i18nrs); translations live in
  `crates/data/i18n/`, language is persisted in `localStorage` (`lang`) and
  detected from the browser language on first visit. A unit test in
  `crates/data` enforces key parity between both languages.
- **Styling:** Tailwind CSS v3 + custom design tokens
- **Build:** [Trunk](https://trunkrs.dev)
- **Data:** `apps/frontend/repos.json` rebuilt daily by GitHub Actions from the public GitHub API

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
# frontend (http://localhost:8080)
cd apps/frontend
npm install
trunk serve

# tests (incl. i18n key parity)
cargo test -p portfolio-data

# resume PDFs (written to dist/resume + dist/resume-fingerprint.json)
cargo run -p resume-generator -- apps/frontend/dist
```

### Production build

```bash
cd apps/frontend && npm install && trunk build --release
cargo run --release -p resume-generator -- apps/frontend/dist
```

Or build the container, which does all of the above and serves the result on
port 8080:

```bash
docker build -t portfolio .
```

## repos.json

The projects section loads `./repos.json` at runtime. A placeholder is
committed at `apps/frontend/repos.json`; the `update-repos.yml` workflow
refreshes it daily from the public GitHub API and commits the result.

## License

Proprietary. See [LICENSE](./LICENSE). Bundled Liberation Sans fonts are
licensed under the SIL OFL (see `apps/resume-generator/fonts/LICENSE`).
