# syntax=docker/dockerfile:1.25@sha256:0adf442eae370b6087e08edc7c50b552d80ddf261576f4ebd6421006b2461f12

ARG USER_ID=1001
ARG GROUP_ID=1001

# Pinned versions of the binstalled build tools. Bumping these (or the base
# image digests above and the committed Cargo.lock / package-lock.json) is the
# only thing that changes the produced artifacts — builds are otherwise
# reproducible.
ARG TRUNK_VERSION=0.21.14
ARG CARGO_CHEF_VERSION=0.1.77

# ── shared build tools ────────────────────────────────────────────────────────
FROM rust:1.94-slim@sha256:cf09adf8c3ebaba10779e5c23ff7fe4df4cccdab8a91f199b0c142c53fef3e1a AS tools

ARG TRUNK_VERSION
ARG CARGO_CHEF_VERSION
# Honoured by tooling that supports it (and forwarded to the build stages) so
# embedded timestamps are deterministic across rebuilds. Inject with
# `--build-arg SOURCE_DATE_EPOCH=$(git log -1 --format=%ct)`.
ARG SOURCE_DATE_EPOCH
ENV SOURCE_DATE_EPOCH=${SOURCE_DATE_EPOCH}

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev curl musl-tools nodejs npm \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add wasm32-unknown-unknown x86_64-unknown-linux-musl

# Pin the binstalled tools to exact versions for reproducible builds; the
# bootstrap script only fetches cargo-binstall itself, which selects the
# pinned binaries.
RUN curl -L --proto '=https' --tlsv1.2 -sSf \
    https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash \
    && cargo binstall --no-confirm --locked \
        "trunk@${TRUNK_VERSION}" "cargo-chef@${CARGO_CHEF_VERSION}"

WORKDIR /app

# ── capture dependency graph for whole workspace ──────────────────────────────
FROM tools AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ── pre-build native dependencies (musl, static) ──────────────────────────────
# Runs in parallel with the frontend build once planner is done.
FROM tools AS server-deps
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --locked --target x86_64-unknown-linux-musl \
    -p server -p resume-generator --recipe-path recipe.json

# ── build server + resume generator, then generate the resume PDFs ────────────
FROM server-deps AS server-builder
COPY . .
RUN cargo build --release --locked --target x86_64-unknown-linux-musl -p server -p resume-generator
# Generate the resume PDFs + resume-fingerprint.json.
RUN ./target/x86_64-unknown-linux-musl/release/resume-generator /app/resume-out

# ── build WASM frontend ───────────────────────────────────────────────────────
# The resume PDFs + resume-fingerprint.json from the server stage are dropped
# into the frontend's `generated/` dir before the build, so the fingerprint is
# embedded into the WASM (build.rs) and the PDFs are served (copy-dir in
# index.html).
FROM tools AS frontend-builder
COPY . .
COPY --from=server-builder /app/resume-out/ /app/apps/frontend/generated/
WORKDIR /app/apps/frontend
# The update-repos Trunk hook regenerates repos.json from the GitHub API during
# the build and fails the build if it cannot be produced. CI=1 selects the 10h
# cache TTL so the committed/cached repos.json is reused when fresh; an optional
# `gh_token` build secret authenticates the call and lifts the API rate limit:
#   docker build --secret id=gh_token,env=GH_TOKEN .
# `npm ci` installs exactly the committed package-lock.json (reproducible),
# unlike `npm install` which may resolve newer compatible versions.
RUN --mount=type=secret,id=gh_token,env=GH_TOKEN \
    CI=1 sh -c 'npm ci && trunk build --release --locked'

# ── pull CA certs for future outbound HTTPS calls ─────────────────────────────
FROM alpine:3.23@sha256:25109184c71bdad752c8312a8623239686a9a2071e8825f20acb8f2198c3f659 AS certs
RUN apk add --no-cache ca-certificates

# ── scratch runtime ───────────────────────────────────────────────────────────
# A `scratch` base with a single statically-linked (musl) binary and read-only
# asset directory: the container holds no shell, package manager or writable
# system paths, so it runs unchanged under a hardened Kubernetes
# `securityContext` (runAsNonRoot, readOnlyRootFilesystem: true,
# allowPrivilegeEscalation: false, capabilities drop ALL, seccomp
# RuntimeDefault). The server only reads from `/dist` and writes logs to stdout,
# so no writable volume (not even /tmp) is required at runtime.
FROM scratch AS runtime

ARG USER_ID
ARG GROUP_ID

# OCI image metadata for provenance/registry scanning. Concrete values
# (revision, version, created, source) are injected by the release pipeline via
# `--build-arg`; sensible defaults keep local builds self-describing.
ARG VCS_REF=unknown
ARG VERSION=dev
ARG CREATED=unknown
ARG SOURCE_URL
LABEL org.opencontainers.image.title="portfolio" \
      org.opencontainers.image.description="Static Yew (WASM) portfolio served by a tiny Axum server on scratch." \
      org.opencontainers.image.url="https://tim-schoenle.de" \
      org.opencontainers.image.source="${SOURCE_URL}" \
      org.opencontainers.image.licenses="LicenseRef-Proprietary" \
      org.opencontainers.image.revision="${VCS_REF}" \
      org.opencontainers.image.version="${VERSION}" \
      org.opencontainers.image.created="${CREATED}" \
      org.opencontainers.image.base.name="scratch"

COPY --from=certs \
    /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt

COPY --from=server-builder \
    /app/target/x86_64-unknown-linux-musl/release/server /server

# Includes the resume PDFs (served via the index.html copy-dir link).
COPY --from=frontend-builder /app/apps/frontend/dist /dist

ENV DIST_DIR=/dist \
    PORT=8080 \
    RUST_LOG=info

EXPOSE 8080
# Numeric uid:gid (not a username) so Kubernetes' `runAsNonRoot` check can
# statically verify the container does not run as root.
USER ${USER_ID}:${GROUP_ID}
ENTRYPOINT ["/server"]
