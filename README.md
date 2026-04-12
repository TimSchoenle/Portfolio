# Portfolio

[![Codecov](https://codecov.io/gh/timschoenle/Portfolio/branch/main/graph/badge.svg)](https://codecov.io/gh/timschoenle/Portfolio)
[![CI](https://github.com/timschoenle/Portfolio/actions/workflows/ci.yaml/badge.svg)](https://github.com/timschoenle/Portfolio/actions/workflows/ci.yaml)

Personal portfolio codebase for [tim-schoenle.de](https://tim-schoenle.de).

It includes:

- localized routes (`/en`, `/de`)
- live GitHub project and contribution data
- generated PDF resumes (optionally digitally signed)
- strict CI quality gates, security checks, and Docker build pipeline

## Before You Clone

This repository is under a **proprietary license**. You can inspect and run it locally for personal evaluation, but reuse and public deployment are restricted. See [LICENSE](./LICENSE).

## Stack

- Runtime: Bun
- Framework: Next.js App Router (React 19, TypeScript strict mode)
- Styling: Tailwind CSS v4 + Radix primitives + custom "blueprint" design system
- i18n: `next-intl`
- Monitoring/analytics: optional Sentry + Cloudflare Web Analytics
- PWA: Serwist service worker and web app manifest
- Testing: Vitest (unit/component), Playwright (e2e), fuzzy tests
- Quality: ESLint, Prettier, Knip, dependency-cruiser, Lighthouse CI, license check
- Delivery: multi-stage Docker image (`output: "standalone"`) + GitHub Actions

## What Is In The App

- Landing page sections for hero, about, skills, projects, experience, contact
- Command palette (`Ctrl+K` / `Cmd+K`) for navigation and quick actions
- GitHub integration for featured repos, profile stats, and contribution heatmap
- Public API endpoints:
  - `GET /api/health`
  - `GET /api/v1/profile`
  - `GET /api/v1/profile/schema`
- Resume generation at build time to `public/resume/{locale}.pdf`
- Optional resume signing with certificate export + fingerprint verification UI
- Localized legal pages (`/imprint`, `/privacy`)

## Repository Layout

```text
.
|- messages/                     # Translation files (en/de)
|- scripts/                      # Build helpers (resume generation, standalone startup, licenses)
|- src/
|  |- app/                       # App Router pages, metadata routes, API routes, service worker
|  |- components/
|  |  |- blueprint/              # Portfolio-specific design primitives
|  |  |- sections/               # Home page sections
|  |  |- features/               # Command palette, contribution graph, analytics, etc.
|  |- data/                      # Site configuration and skills data
|  |- i18n/                      # Locale routing and request helpers
|  |- lib/                       # GitHub client, logger, utility modules
|  |- models/                    # API/data schemas (zod)
|- test/                         # Vitest setup
|- tests/                        # Playwright tests + global setup
|- .github/workflows/            # CI/CD + security workflows
```

## Quick Start

### 1. Install dependencies

```bash
bun install
```

### 2. Create `.env.local` (recommended)

```env
GITHUB_TOKEN=ghp_xxx
```

Notes:

- `GITHUB_TOKEN` is optional, but without it GitHub data may be empty/rate-limited.

### 3. Run in development

```bash
bun run dev
```

Open:

- http://localhost:3000/en
- http://localhost:3000/de

### 4. Build and run production locally

```bash
bun run build
bun run start
```

`start` runs a standalone server helper (`scripts/start-standalone.ts`) with `HOSTNAME=0.0.0.0` and `PORT=3000`.

## API Endpoints

### `GET /api/health`

Returns runtime liveness:

```json
{
  "status": "healthy",
  "timestamp": "<ISO-8601 timestamp>"
}
```

### `GET /api/v1/profile`

Returns public profile data (contact, social links, typed skill groups). Response includes:

- validated payload based on zod schema
- `$schema` link pointing to `/api/v1/profile/schema`

### `GET /api/v1/profile/schema`

Returns the JSON schema for `/api/v1/profile`.

## Resume Generation And Signing

Builds always generate localized PDFs:

```bash
bun run build:resume
```

Unsigned mode:

- writes `public/resume/en.pdf` and `public/resume/de.pdf`

Signed mode:

1. Provide `RESUME_SIGNING_CERT_BASE64` and `RESUME_SIGNING_CERT_PASSWORD`
2. Build resumes again
3. Script exports:
   - signed PDFs
   - `public/resume/certificate.crt`
   - `public/resume-fingerprint.json`

For local certificate bootstrap:

```bash
bun scripts/generate-resume-certificate.ts
```

## Troubleshooting

- GitHub cards or contribution graph empty:
  - check `GITHUB_TOKEN`
  - check API rate limits

## Related Documents

- [CONTRIBUTING.md](./CONTRIBUTING.md)
- [SECURITY.md](./SECURITY.md)
- [CHANGELOG.md](./CHANGELOG.md)

## License

Proprietary license. See [LICENSE](./LICENSE) for terms.
