# syntax=docker/dockerfile:1@sha256:ecfaec9ed6d810b56388c508f4121597bfbba70d41a6dfeee4d8cad5f295fc32

ARG USER_ID=1001
ARG GROUP_ID=1001

# Pinned build-tool versions. Bump alongside the base image digests and the
# committed Cargo.lock / package-lock.json.
ARG DIOXUS_CLI_VERSION=0.7.9

# ── shared build tools ────────────────────────────────────────────────────────
# Rust + the wasm target + Node (Tailwind CLI) + the Dioxus CLI (`dx`).
#
# Pinned to $BUILDPLATFORM: every build stage runs natively on the builder and
# *cross-compiles* the SSR server to $TARGETPLATFORM. Running the whole Rust +
# wasm + npm build under QEMU for a foreign arch would be an order of magnitude
# slower, and nothing here but the final server binary is arch-dependent (the
# client is wasm; repos.json, the resume PDFs and the Tailwind CSS are data).
FROM --platform=$BUILDPLATFORM rust:1.97-slim@sha256:3b2879047d42784ca9403ad20c51ed3df361a50f1df96f5777d39b4e33aa65cd AS tools

ARG DIOXUS_CLI_VERSION
ARG TARGETARCH
# Honoured by tooling that supports it so embedded timestamps stay deterministic.
ARG SOURCE_DATE_EPOCH
ENV SOURCE_DATE_EPOCH=${SOURCE_DATE_EPOCH}

# Docker's TARGETARCH → Rust musl triple, resolved once and recorded so the
# mapping (and the list of supported architectures) lives in exactly one place.
# Unknown architectures fail loudly here rather than silently building the wrong
# thing further down.
RUN case "${TARGETARCH}" in \
      amd64) echo 'x86_64-unknown-linux-musl' ;; \
      arm64) echo 'aarch64-unknown-linux-musl' ;; \
      *) echo "unsupported TARGETARCH: '${TARGETARCH}'" >&2; exit 1 ;; \
    esac > /etc/rust-target

# `musl-tools` provides musl-gcc for the native x86_64 musl target;
# `gcc-aarch64-linux-gnu` is the cross toolchain for the arm64 target. Both are
# only ever used to compile ring's C and to drive the link — rustc supplies its
# own bundled musl libc.a and crt objects for the musl targets
# (`self-contained` linking), so the glibc-flavoured cross gcc never contributes
# a libc and the result stays fully static.
#
# `libc6-dev-arm64-cross` is required and easy to miss: gcc-aarch64-linux-gnu
# only *recommends* it, so under --no-install-recommends the cross compiler ends
# up with no target headers at all and ring's first .c file fails on <stdlib.h>.
# Its headers are glibc's, but they are only ever preprocessed — nothing from
# this package reaches the linked binary.
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev curl nodejs npm ca-certificates musl-tools \
    && if [ "${TARGETARCH}" = "arm64" ]; then \
         apt-get install -y --no-install-recommends \
           gcc-aarch64-linux-gnu libc6-dev-arm64-cross; \
       fi \
    && rm -rf /var/lib/apt/lists/*

# wasm32 for the client; musl for a fully static SSR server (see runtime stage).
RUN rustup target add wasm32-unknown-unknown "$(cat /etc/rust-target)"

# Pin the binstalled Dioxus CLI for reproducible builds.
#
# The bootstrap script is pinned to a commit, not to `main`: it is piped
# straight into a shell, so fetching whatever `main` happens to hold would make
# every build depend on the current state of a repository we do not control —
# the one unpinned input in a file where the base images, actions and CLI
# version are all pinned. Renovate tracks the SHA via the annotation below.
# renovate: datasource=github-tags depName=cargo-bins/cargo-binstall
ARG CARGO_BINSTALL_REF=e00d2c94cc0067b77737821097a62d91c0301baa # tag=v1.21.1
RUN curl -L --proto '=https' --tlsv1.2 -sSf \
    "https://raw.githubusercontent.com/cargo-bins/cargo-binstall/${CARGO_BINSTALL_REF}/install-from-binstall-release.sh" | bash \
    && cargo binstall --no-confirm --locked "dioxus-cli@${DIOXUS_CLI_VERSION}"

WORKDIR /app

# ── generate build-time data: repos.json, resume PDFs + fingerprint ───────────
FROM tools AS generate
COPY . .
# repos.json from the GitHub API (build.rs embeds it into the web binary). CI=1
# selects the long cache TTL so a fresh committed/cached file is reused; an
# optional `gh_token` build secret lifts the API rate limit.
#
# The secret is handed over as a *path*, not as an environment variable: the
# builder resolves `PORTFOLIO_GITHUB__TOKEN_FILE` through the layered config, so
# the token is never in the process environment where a crash dump, `ps e` or a
# child process would carry it. BuildKit omits the mount entirely when the
# secret was not supplied (a fork's pull request has none), so the path is only
# exported when it actually exists — pointing the indirection at a missing file
# is an error, and correctly so.
RUN --mount=type=secret,id=gh_token \
    if [ -s /run/secrets/gh_token ]; then \
      export PORTFOLIO_GITHUB__TOKEN_FILE=/run/secrets/gh_token; \
    fi; \
    CI=1 cargo run --release --locked -p update-repos -- apps/web/repos.json
# Resume PDFs + resume-fingerprint.json, both embedded into the web binary by
# build.rs (the PDFs are served at /resume/ from memory; the fingerprint is shown
# on the contact card). Embedding keeps the runtime a single self-contained
# binary with no on-disk asset tree.
RUN cargo run --release --locked -p resume-generator -- /app/resume-out \
    && mkdir -p apps/web/generated/resume \
    && cp /app/resume-out/resume/*.pdf apps/web/generated/resume/ \
    && cp /app/resume-out/resume-fingerprint.json apps/web/generated/

# ── build the fullstack app (client wasm + native SSR server) ─────────────────
FROM generate AS web-builder
WORKDIR /app/apps/web
# `npm ci` installs exactly the committed package-lock.json; `build:css` compiles
# Tailwind (scanning the .rs files) into assets/tailwind.css, which the app links
# via manganis `asset!`.
RUN npm ci && npm run build:css
# Produces target/dx/web/release/web/{server, public/} — the server binary plus
# the hashed client assets (the /resume PDFs are embedded, not bundled). The client stays wasm
# while `@server --target …-musl` cross-links the SSR server fully static
# (ring/rustls, no glibc), so it can run on `scratch`. Passing `--target`
# explicitly keeps proc-macros/build-scripts on the host toolchain.
#
# Per-target C toolchain for ring's build script (CC_*/AR_*) and for the link
# step (CARGO_TARGET_*_LINKER). Only the entries matching /etc/rust-target are
# consulted, so declaring both architectures unconditionally is harmless.
#
# The x86_64 target deliberately sets no linker here: the repository's
# `.cargo/config.toml` already points it at `x86_64-linux-musl-gcc` (installed
# with `musl-tools` above), and cargo picks that up from the workspace root. Keep
# the two in sync — a linker named in only one of them is a silent build break.
ENV CC_x86_64_unknown_linux_musl=musl-gcc \
    CC_aarch64_unknown_linux_musl=aarch64-linux-gnu-gcc \
    AR_aarch64_unknown_linux_musl=aarch64-linux-gnu-ar \
    CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=aarch64-linux-gnu-gcc
# `--debug-symbols=false` strips the client wasm's DWARF debug info and `name`
# section for production: it shrinks the payload substantially and avoids the
# malformed `name` custom section that Firefox rejects at validation. We already
# don't `--keep-names`, so no readable backtraces are lost.
RUN dx bundle --release \
    @client --platform web --debug-symbols=false \
    @server --platform server --target "$(cat /etc/rust-target)"
# An empty, staged ISR cache directory. The `scratch` runtime has no shell to
# `mkdir` with and its non-root user cannot create directories under `/`, so the
# directory (and its ownership) has to be baked in here and COPYed across to
# /tmp/isr (see the runtime stage).
RUN mkdir -p /isr-cache

# ── runtime ───────────────────────────────────────────────────────────────────
# `scratch`: the smallest possible attack surface — no shell, no package
# manager, no libc, nothing but our own files. This is viable because the SSR
# server is a fully static musl binary (see web-builder) and serves only
# compile-time data, so it makes no outbound TLS at runtime and therefore needs
# no CA bundle, tzdata, or /etc/passwd (a numeric USER needs no passwd entry).
#
# Unlike every stage above — which deliberately pins itself to $BUILDPLATFORM —
# this stage takes the default, $TARGETPLATFORM (stating it explicitly trips
# BuildKit's RedundantTargetPlatform check). It is the one stage that must carry
# the requested architecture, and the binary COPYed into it was cross-compiled
# to match.
FROM scratch AS runtime

# Re-declared so the globals above are in scope for the `USER` instruction.
ARG USER_ID
ARG GROUP_ID
ARG VCS_REF=unknown
ARG VERSION=dev
ARG CREATED=unknown
ARG SOURCE_URL
LABEL org.opencontainers.image.title="portfolio" \
      org.opencontainers.image.description="Dioxus fullstack (SSR + hydration) portfolio served by Axum." \
      org.opencontainers.image.url="https://tim-schoenle.de" \
      org.opencontainers.image.source="${SOURCE_URL}" \
      org.opencontainers.image.licenses="LicenseRef-Proprietary" \
      org.opencontainers.image.revision="${VCS_REF}" \
      org.opencontainers.image.version="${VERSION}" \
      org.opencontainers.image.created="${CREATED}"

# The whole bundle: the `server` binary and its sibling `public/` asset dir. The
# server resolves `public/` relative to itself, so keep them together and run
# from that directory.
COPY --from=web-builder /app/target/dx/web/release/web /app
# The ISR cache directory, owned by the runtime user so it is writable even on
# the read-only `scratch` root (the non-root user cannot create it at runtime).
# It lives under /tmp, which the deployment (Helm chart) already mounts as a
# writable volume; a plain `docker run` uses this baked-in copy instead.
COPY --from=web-builder --chown=${USER_ID}:${GROUP_ID} /isr-cache /tmp/isr
WORKDIR /app

# The server reads its own settings through the layered `PORTFOLIO_` config
# (struct defaults < TOML at $PORTFOLIO_CONFIG < PORTFOLIO_* environment <
# $PORTFOLIO_SECRETS_DIR < PORTFOLIO_<KEY>_FILE), so every key below can equally
# be supplied as a mounted ConfigMap fragment or Secret file. See README.md and
# config.example.toml. PORT, IP and RUST_LOG are *not* in that namespace: they
# belong to the Dioxus toolchain, which reads them itself.
#
# Incremental static regeneration (ISR) is ON by default: the cache directory
# points at /tmp/isr, which the deployment (Helm chart) already mounts as a
# writable volume (so it works under `readOnlyRootFilesystem: true` with no extra
# setup); a plain `docker run` falls back to the baked-in copy above. The server
# is locale-aware — it tags each render with the negotiated language, so the
# cache keeps a separate entry per locale — and the site has only a handful of
# pages built entirely from compile-time data, so caching every rendered page is
# safe and cheap. If /tmp/isr is ever not writable, the server fails safe and
# renders every request fresh.
#
# The cache is permanent by default (no time-based TTL): the only thing that
# changes a page is a new build, which starts from an empty cache. Set the TTL to
# a positive number of seconds only when you share a *persistent* cache volume
# across deploys and want time-based revalidation; set the cache directory empty
# to turn ISR off entirely. Keep the path outside /app/public so the immutable,
# content-hashed assets stay read-only.
#   PORTFOLIO_ISR__CACHE_DIR=/tmp/isr  # empty disables ISR
#   PORTFOLIO_ISR__TTL_SECS=0          # 0/unset = permanent; positive = finite TTL
#   PORTFOLIO_ASSETS__DIST_DIR=public  # what the readiness probe checks
#
# The Content-Security-Policy is built per document from the inline scripts that
# document carries, with a per-response nonce for the script Cloudflare's bot
# products inject at the edge. The ISR cache above is unaffected: it stores the
# body, and the nonce lives only in the response header, minted after the cached
# render is read. Both keys default to on and move together — see README.md.
#   PORTFOLIO_CSP__HASH_INLINE_SCRIPTS=true       # false restores 'unsafe-inline'
#   PORTFOLIO_CSP__CLOUDFLARE__SCRIPT_NONCE=true  # false drops the edge nonce
#   PORTFOLIO_CSP__CLOUDFLARE__TURNSTILE=false    # admit the Turnstile widget
#   PORTFOLIO_CSP__CLOUDFLARE__WEB_ANALYTICS=false
ENV PORT=8080 \
    IP=0.0.0.0 \
    RUST_LOG=info \
    PORTFOLIO_ISR__CACHE_DIR=/tmp/isr

EXPOSE 8080
USER ${USER_ID}:${GROUP_ID}
ENTRYPOINT ["/app/server"]
