# Security posture

The Content-Security-Policy the server builds per response, the headers around it, and what the process needs from the filesystem it runs on.

## Runtime

The server reads its bundle directory and writes to stdout. Nothing else. It runs with a read-only
root filesystem and no writable volume, apart from the ISR cache described in
[DEPLOYMENT.md](DEPLOYMENT.md), which it does without when the path is not writable.

It is built to satisfy the restricted Pod Security Standard:

- Numeric non-root `1001:1001`, so `runAsNonRoot` verifies statically without a passwd lookup.
- `readOnlyRootFilesystem: true` and `allowPrivilegeEscalation: false`.
- Every Linux capability dropped, `seccompProfile: RuntimeDefault`.
- Listens on `:$PORT`, default `8080`.

Everything else it reads comes from the layered configuration, so a `ConfigMap` fragment or a
`Secret` volume is a first-class source rather than something the chart has to flatten into
environment variables.

## Headers

Five headers are set on every response, overriding whatever a handler produced:

| Header | Value |
| --- | --- |
| `X-Content-Type-Options` | `nosniff` |
| `X-Frame-Options` | `DENY` |
| `Referrer-Policy` | `no-referrer` |
| `Strict-Transport-Security` | `max-age=31536000; includeSubDomains; preload` |
| `Permissions-Policy` | `camera=(), microphone=(), geolocation=(), interest-cohort=()` |

`Content-Security-Policy` is the exception. The subresource policy is set only when the document
middleware has not already set a stricter one, which is what the section below is about.

## Content-Security-Policy

Built with [csp-shell](https://github.com/TimSchoenle/csp-shell) in `apps/web/src/server/csp.rs`,
from the bytes of the document being served rather than written out as a string. Two policies leave
the server.

**Documents** get one derived from what they carry. Dioxus renders its hydration data inline as
`window.initial_dioxus_hydration_data="…"`, so the text of that script is only known per response.
The body is buffered anyway, to stamp `<html lang>`, and each inline script is hashed out of that
same string. `'unsafe-inline'` is therefore absent from `script-src`, and an injected `<script>` no
longer runs merely because the hydration bootstrap has to.

**Everything else** — assets, the JSON API, the SEO documents — gets a policy admitting no inline
script at all, because none of them has any.

`'unsafe-eval'` remains, for the one reason it has ever been here: `dioxus-web`'s document provider
applies `document::Title`, `document::Stylesheet` and friends through `new Function(…)`, which
throws an uncaught `EvalError` without it and freezes client-side navigation.

### Cloudflare

The bot products in front of this origin inject an inline `<script>` at the edge, after this server
has hashed what it rendered. No hash can cover it. A nonce can, because Cloudflare parses the
`Content-Security-Policy` response header and copies the nonce onto what it injects. That is what
`csp.cloudflare.script_nonce` reserves, on by default.

It brings one obligation the server discharges and one it cannot. Every document is served
`Cache-Control: no-cache`, so a nonce is never shared between readers. The other:

> **Deployment checklist.** No Cloudflare Cache Rule may cache the shell. A "Cache Everything" rule
> overrides the origin's `Cache-Control` and pins one nonce across every reader for the lifetime of
> the cache entry, and nothing inside this process can see that happening.

The ISR cache is unaffected. It stores the body, and the nonce lives only in the response header,
minted after the cached render is read.

Turnstile and Web Analytics are off. Switching either on admits its origins in the directives that
product actually needs them in — Turnstile in `script-src` **and** `frame-src`, since admitting only
the first renders an empty box.

### If a page renders blank

That is the failure mode this design has. Setting both `csp.hash_inline_scripts` and
`csp.cloudflare.script_nonce` to false restores `'unsafe-inline'` on a restart rather than a
redeploy. The two must move together: a browser ignores `'unsafe-inline'` as soon as the policy
carries a nonce, and supplying one without the other fails the boot with a message saying so.

## The one secret

`github.token` is the only secret in the workspace, and the server never loads it — it belongs to
the build-time repository lister. In the process it is a `secrecy::SecretString`, which has no
`Debug` and no `Display`, and the single place it becomes a `&str` is the `Authorization` header.

Supply it as a file. `/proc/<pid>/environ`, a crash dump and `docker inspect` all carry the
environment, and child processes inherit it. The Docker build already does this, mounting the
BuildKit secret and handing `update-repos` the path.

Only the loader half of `terrace-config` is used; its hot-reload supervisor is not. The reasoning —
the server holds no secrets, and `dioxus::serve` owns an accept loop with no shutdown handle — is
recorded in `crates/config/src/lib.rs`.
