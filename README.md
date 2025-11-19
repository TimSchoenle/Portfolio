# Portfolio

[![codecov](https://codecov.io/gh/Timmi6790/Portfolio/branch/main/graph/badge.svg)](https://codecov.io/gh/Timmi6790/Portfolio)

## Tech Stack

- **Framework**: Next.js 16 (App Router)
- **UI**: React 19 + shadcn/ui, Radix UI, Lucide Icons
- **Styling**: Tailwind CSS
- **Language**: TypeScript
- **i18n**: next-intl
- **Testing**: Vitest, Playwright (+ axe-core)
- **CI**: GitHub Actions (tests, bundle analysis, Lighthouse)
- **Package manager**: pnpm

## Project Structure

```text
├── .github/
├── messages/
├── public/
├── scripts/
├── src/
│   ├── app/
│   │   ├── [locale]/
│   │   │   ├── imprint/
│   │   │   ├── privacy/
│   │   │   ├── layout.tsx
│   │   │   └── page.tsx
│   │   ├── api/
│   │   ├── globals.css
│   │   ├── robots.ts
│   │   └── sitemap.ts
│   ├── components/
│   ├── i18n/
│   ├── lib/
│   ├── types/
│   └── proxy.ts
├── test/
├── tests/
├── commitlint.config.ts
├── components.json
…
```

## Getting Started

### Prerequisites

- Node.js
- pnpm

### Install & Run

```bash
pnpm install
pnpm dev            # http://localhost:3000
```

### Build & Start (production)

```bash
pnpm build
pnpm start          # runs standalone server script
```

## Environment Variables

| Name           | Required | Default | Purpose                                           |
| -------------- | -------- | ------- | ------------------------------------------------- |
| `GITHUB_TOKEN` | Yes      | —       | Token to access private data for the chart graph. |

Create an `.env.local` and set variables as needed.

## Docker

Build a production image and run:

```bash
pnpm docker:build
pnpm docker:run
# open http://localhost:3000
```
