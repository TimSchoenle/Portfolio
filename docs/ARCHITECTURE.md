# Architecture

Six packages in one Cargo workspace: two libraries every binary reads, three binaries, and a placeholder at the root that exists so release-please has a version to move.

## The packages

| Package | Purpose |
| --- | --- |
| `crates/config` | The typed configuration blocks each binary reads, plus the Portfolio dialect of the [terrace-config](https://github.com/TimSchoenle/terrace-config) layered loader |
| `crates/data` | Language-neutral data — site config, skills, experience, the `repos.json` schema — and the embedded `i18n/{en,de}.json` translations |
| `apps/web` | The site. One crate that builds twice: a WASM client under the `web` feature and a native Axum SSR server under `server`, carrying the JSON API, the SEO documents, the security headers and the probes |
| `apps/resume-generator` | Typesets one resume PDF per language, writes `resume-fingerprint.json`, and rasterises the 1200×630 social card, with its fonts embedded |
| `apps/update-repos` | Lists the owner's active GitHub repositories and rewrites `apps/web/repos.json` through the shared `Repo`/`ReposFile` models |
| `.` (`portfolio-platform`) | `src/lib.rs` is a placeholder. release-please's Rust strategy needs a root package to bump, and the version it writes there is the one the README payload and the image's contract document both read |

## One crate, two builds

`apps/web` selects its renderer with a feature rather than splitting into two crates, because the
route table, the page components and the `<head>` metadata are shared and would otherwise be a
third crate that both depend on. `web` pulls `dioxus-web` and `web-sys`; `server` pulls axum,
`tower-http` and `csp-shell`. The Dioxus CLI (`dx`) drives both halves.

Server-side rendering is not a fallback here. The locale is negotiated from request headers in
`apps/web/src/i18n.rs` and applied before the document is serialised, so nothing arrives in the
wrong language and gets swapped once hydration runs. The `<html lang>` attribute is stamped into
the buffered body, which is also where the Content-Security-Policy gets the inline scripts it
hashes — see [SECURITY_POSTURE.md](SECURITY_POSTURE.md).

## Internationalization

EN and DE, through [i18nrs](https://crates.io/crates/i18nrs) with only its `dio` component set
enabled. `dio-ssr` is left out: it negotiates the locale through a `#[server]` round-trip whose
`get_cookie` result is discarded upstream, so this workspace negotiates on the server from request
headers and reads `document.cookie` directly on the client instead.

`translation_key_sets_match` in `crates/data` compares the key sets of both translation files and
fails the build when they differ. i18nrs falls back to an arbitrary language for a missing key,
which shows up as one English string in a German page rather than as an error, so the test is the
only thing that catches it.

## What the client ships

The WASM bundle is built under the `release` profile: `opt-level = "z"`, fat LTO, one codegen unit.
The profile sets `panic = "unwind"` even so, because it also builds the SSR server, where a panic
in one handler has to fail that request rather than the process. The wasm target has no unwinding,
so the client keeps its size either way.

Tailwind CSS v4 is compiled by `@tailwindcss/cli` through npm, which is the only reason Node.js is
a prerequisite.

## Generated artefacts

Three things reach the binary from outside the source tree, and `apps/web/build.rs` embeds all
three with `include_str!`. Each has an empty default it falls back to, so a bare `cargo check`
outside the image build still compiles and the affected page renders its empty state.

| Artefact | Written by | Read by |
| --- | --- | --- |
| `apps/web/repos.json` | `apps/update-repos` | the projects section — see [PROJECT_DATA.md](PROJECT_DATA.md) |
| `apps/web/generated/licenses.json` | `cargo about`, through `just licenses` | `/licenses` — see [DEPLOYMENT.md](DEPLOYMENT.md) |
| `resume-fingerprint.json` | `apps/resume-generator` | the contact card, which shows each PDF's SHA-256 |

The resume generator is a Typst document builder rather than a PDF library. It embeds Inter, with
Liberation Sans as a metric-compatible last resort, and emits a tagged PDF 1.7 with live link
annotations. `fit` is the part worth knowing about: it scales the type down and re-typesets until
the content lands on one A4 page, because a resume that runs to a second page is a resume an ATS
truncates. The main column is emitted before the sidebar so a text extractor reads identity,
summary and experience in that order despite the sidebar sitting on the left.

The same run rasterises `og-image.png` at 1200×630. Every link unfurler refuses SVG, and Typst is
already here with the brand font and a layout engine, so the social card is a second small document
rather than a second toolchain.

## Routes

`/` is a single page of sections `s1`–`s5`. `/imprint`, `/privacy` and `/licenses` are routes in
the same shell, translated and server-rendered like any other, with a 404 page under the catch-all.
Beside them the server registers `/api/v1/profile` and its JSON schema, the probes documented in
the README, and `robots.txt`, `sitemap.xml` and `site.webmanifest`.
