# syntax=docker/dockerfile:1@sha256:87999aa3d42bdc6bea60565083ee17e86d1f3339802f543c0d03998580f9cb89

ARG USER_ID=1001
ARG GROUP_ID=1001

# Pinned build-tool versions. Bump alongside the base image digests and the
# committed Cargo.lock / package-lock.json.
ARG DIOXUS_CLI_VERSION=0.7.9

# ── shared build tools ────────────────────────────────────────────────────────
# Rust + the wasm target + Node (Tailwind CLI) + the Dioxus CLI (`dx`).
FROM rust:1.97-slim@sha256:41a5647cbdc067365b49e71f6a416789ab2ff884e501577643de6e99cb11b61d AS tools

ARG DIOXUS_CLI_VERSION
# Honoured by tooling that supports it so embedded timestamps stay deterministic.
ARG SOURCE_DATE_EPOCH
ENV SOURCE_DATE_EPOCH=${SOURCE_DATE_EPOCH}

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev curl nodejs npm ca-certificates musl-tools \
    && rm -rf /var/lib/apt/lists/*

# wasm32 for the client; musl for a fully static SSR server (see runtime stage).
RUN rustup target add wasm32-unknown-unknown x86_64-unknown-linux-musl

# Pin the binstalled Dioxus CLI for reproducible builds.
RUN curl -L --proto '=https' --tlsv1.2 -sSf \
    https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash \
    && cargo binstall --no-confirm --locked "dioxus-cli@${DIOXUS_CLI_VERSION}"

WORKDIR /app

# ── generate build-time data: repos.json, resume PDFs + fingerprint ───────────
FROM tools AS generate
COPY . .
# repos.json from the GitHub API (build.rs embeds it into the web binary). CI=1
# selects the long cache TTL so a fresh committed/cached file is reused; an
# optional `gh_token` build secret lifts the API rate limit.
RUN --mount=type=secret,id=gh_token,env=GH_TOKEN \
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
# explicitly keeps proc-macros/build-scripts on the host toolchain. `musl-gcc`
# is the C compiler ring's build script uses for the musl target.
ENV CC_x86_64_unknown_linux_musl=musl-gcc
# `--debug-symbols=false` strips the client wasm's DWARF debug info and `name`
# section for production: it shrinks the payload substantially and avoids the
# malformed `name` custom section that Firefox rejects at validation. We already
# don't `--keep-names`, so no readable backtraces are lost.
RUN dx bundle --release \
    @client --platform web --debug-symbols=false \
    @server --platform server --target x86_64-unknown-linux-musl
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

# Incremental static regeneration (ISR) is ON by default: ISR_CACHE_DIR points at
# /tmp/isr, which the deployment (Helm chart) already mounts as a writable volume
# (so it works under `readOnlyRootFilesystem: true` with no extra setup); a plain
# `docker run` falls back to the baked-in copy above. The server is locale-aware
# — it tags each render with the negotiated language, so the cache keeps a
# separate entry per locale — and the site has only a handful of pages built
# entirely from compile-time data, so caching every rendered page is safe and
# cheap. If /tmp/isr is ever not writable, the server fails safe and renders
# every request fresh.
#
# The cache is permanent by default (no time-based TTL): the only thing that
# changes a page is a new build, which starts from an empty cache. Set
# ISR_TTL_SECS to a positive number of seconds only when you share a *persistent*
# cache volume across deploys and want time-based revalidation; set ISR_CACHE_DIR
# empty to turn ISR off entirely. Keep the path outside /app/public so the
# immutable, content-hashed assets stay read-only.
#   ISR_CACHE_DIR=/tmp/isr   # empty disables ISR
#   ISR_TTL_SECS=0           # 0/unset = permanent; positive = finite TTL
ENV PORT=8080 \
    IP=0.0.0.0 \
    RUST_LOG=info \
    ISR_CACHE_DIR=/tmp/isr

EXPOSE 8080
USER ${USER_ID}:${GROUP_ID}
ENTRYPOINT ["/app/server"]
