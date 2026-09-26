# Architecture

Six packages in one Cargo workspace: two libraries every binary reads, three binaries, and a placeholder at the root that exists so release-please has a version to move.

## The packages

| Package | Purpose |
| --- | --- |
| `crates/config` | The typed configuration blocks each binary reads, plus the Portfolio dialect of the [terrace-config](https://github.com/TimSchoenle/terrace-config) layered loader |
| `crates/data` | Language-neutral data — the site languages (`SITE_LANGUAGES`), site config, skills, experience, the `repos.json` schema — and the embedded `i18n/{en,de}.json` translations |
| `apps/web` | The site. One crate that builds twice: a WASM client under the `web` feature and a native Axum SSR server under `server`, carrying the JSON API, the SEO documents, the security headers and the probes |
| `apps/resume-generator` | Typesets one resume PDF per language, writes `resume-fingerprint.json`, and rasterizes the 1200×630 social card and the 192/512 px app icons, with its fonts embedded |
| `apps/update-repos` | Lists the owner's active GitHub repositories and rewrites `apps/web/repos.json` through the shared `Repo`/`ReposFile` models |
| `.` (`portfolio-platform`) | `src/lib.rs` is a placeholder. release-please's Rust strategy needs a root package to bump, and the version it writes there is the one the README payload and the image's contract document both read |

## One crate, two builds

`apps/web` selects its renderer with a feature rather than splitting into two crates, because the
route table, the page components and the `<head>` metadata are shared and would otherwise be a
third crate that both depend on. `web` pulls `dioxus-web` and `web-sys`; `server` pulls axum,
`tower-http`, `csp-shell` and the Sentry SDK. The Dioxus CLI (`dx`) drives both halves.

Sentry is on the server side only, and there is deliberately no browser SDK: reporting a client
error would mean a third-party script on every page load, a `connect-src` in the
Content-Security-Policy and a line in the privacy page. It is compiled into every server build and
switched on by `sentry.enabled` rather than by a Cargo feature, so an image cannot accept the key
and silently do nothing with it — see [SECURITY_POSTURE.md](SECURITY_POSTURE.md).

Server-side rendering is not a fallback here. The locale is negotiated from the request in
`apps/web/src/i18n.rs` and applied before the document is serialized, so nothing arrives in the
wrong language and gets swapped once hydration runs.

Every document then passes through `apps/web/src/server/page.rs`, which sits outside the Dioxus
router and so also sees pages the incremental cache answers without rendering. It stamps
`<html lang>`, writes the `lang` cookie when the request's disagrees, and gives the document a
Content-Security-Policy hashed from the bytes it is about to send — see
[SECURITY_POSTURE.md](SECURITY_POSTURE.md). A finished page is memoized per path and language, so a
repeat request is answered without reaching the router at all; only the nonce and the cookie are
per response. `apps/web/src/server/isr.rs` is the on-disk render cache behind it, keyed per
language and restricted to an allowlist of real pages.

## Internationalization

EN and DE, through [i18nrs](https://crates.io/crates/i18nrs) with only its `dio` component set
enabled. `dio-ssr` is left out: it negotiates the locale through a `#[server]` round-trip whose
`get_cookie` result is discarded upstream, so this workspace negotiates on the server itself.

The server decides in this order: a `?lang=` query parameter, a valid `lang` cookie,
`Accept-Language` (quality values honored), the default language. The query parameter is what
gives each language an address of its own: crawlers send neither a cookie nor, usually, an
`Accept-Language`, so without it only the default language would ever be indexed. Every page
declares its variants as `<link rel="alternate" hreflang>` and the sitemap lists each page once per
language with the same alternates.

The wasm client starts in the language `<html lang>` names and falls back to the cookie. The
attribute is guaranteed to describe the HTML being hydrated; the cookie is not, for example on a
first visit answered from the render cache.

Everything about a language that is not prose — its code, `og:locale`, translation file and resume
file name — is one `Language` entry in `crates/data`, and every consumer iterates that table.

`translation_key_sets_match` in `crates/data` compares the key sets of both translation files and
fails the build when they differ. i18nrs falls back to an arbitrary language for a missing key,
which shows up as one English string in a German page rather than as an error, so the test is the
only thing that catches it.

## What the client ships

`dx bundle --release` does not build under Cargo's `release` profile. It injects two profiles of
its own, `wasm-release` for the client and `server-release` for the server, and the tuning in the
root `[profile.release]` (`opt-level = "z"`, fat LTO, one codegen unit) reaches the shipped
binaries only through them. The profile sets `panic = "unwind"` because it also applies to the SSR
server, where a panic in one handler has to fail that request rather than the process. The wasm
target has no unwinding, so the client keeps its size either way.

Tailwind CSS v4 is compiled by `@tailwindcss/cli` through npm, which is the only reason Node.js is
a prerequisite.

## Generated artifacts

Several files reach the binary from outside the source tree, and `apps/web/build.rs` embeds them.
Each has an empty default it falls back to, so a bare `cargo check` outside the image build still
compiles and the affected page renders its empty state.

| Artifact | Written by | Read by |
| --- | --- | --- |
| `apps/web/repos.json` | `apps/update-repos` | the projects section — see [PROJECT_DATA.md](PROJECT_DATA.md) |
| `apps/web/generated/licenses.json` | `cargo about`, through `just licenses` | `/licenses` — see [DEPLOYMENT.md](DEPLOYMENT.md) |
| `apps/web/generated/resume/*.pdf` | `apps/resume-generator` | `/resume/<file>`, linked from the contact card and the command palette |
| `apps/web/generated/resume-fingerprint.json` | `apps/resume-generator` | the contact card, which shows each PDF's SHA-256 |
| `apps/web/generated/og-image.png` | `apps/resume-generator` | `/og-image.png`, the `og:image` of every page |

The resume generator is a Typst document builder rather than a PDF library. It embeds Inter, with
Liberation Sans as a metric-compatible last resort, and emits a tagged PDF 1.7 with live link
annotations. `fit` is the part worth knowing about: it scales the type down and re-typesets until
the content lands on one A4 page, because a resume that runs to a second page is a resume an ATS
truncates. The main column is emitted before the sidebar so a text extractor reads identity,
summary and experience in that order despite the sidebar sitting on the left.

The German sheet can carry an application photo at the top of its sidebar. The photo is a build
input, never a committed file: pass `--photo <file>` (or `PORTFOLIO_RESUME__PHOTO_FILE`), which
the image build supplies as the BuildKit secret `resume_photo`. Without it, both sheets render
without a photo.

The same run rasterizes `og-image.png` at 1200×630. Every link unfurler refuses SVG, and Typst is
already here with the brand font and a layout engine, so the social card is a second small document
rather than a second toolchain.

## Routes

`/` is a single page of sections `s1`–`s5`. `/licenses` and `/legal/:slug` are routes in the same
shell, translated and server-rendered like any other, with a 404 page under the catch-all. Which
legal documents exist is configuration (`legal.*`, validated by `terrace-legal` plus the rules in
`crates/config/src/legal.rs`): the shell fetches their index with a server future so the footer
links are in the first response, and `/legal/:slug` renders an unknown slug as the 404 page.
`/imprint` and `/privacy` redirect permanently to their `/legal/` pages. Beside them the server
registers `/api/v1/profile` and its JSON schema, `/api/v1/legal` (the same catalog as JSON, with
`ETag` and `304`), the probes documented in the README, and `robots.txt`, `sitemap.xml` and
`site.webmanifest`.
