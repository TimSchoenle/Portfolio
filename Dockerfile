# syntax=docker/dockerfile:1.23@sha256:2780b5c3bab67f1f76c781860de469442999ed1a0d7992a5efdf2cffc0e3d769

ARG USER_ID=1001
ARG GROUP_ID=1001

# ── shared build tools ────────────────────────────────────────────────────────
FROM rust:1.94-slim@sha256:cf09adf8c3ebaba10779e5c23ff7fe4df4cccdab8a91f199b0c142c53fef3e1a AS tools

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev curl musl-tools nodejs npm \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add wasm32-unknown-unknown x86_64-unknown-linux-musl

RUN curl -L --proto '=https' --tlsv1.2 -sSf \
    https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash \
    && cargo binstall trunk cargo-chef --no-confirm

WORKDIR /app

# ── capture dependency graph for whole workspace ──────────────────────────────
FROM tools AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ── pre-build native dependencies (musl, static) ──────────────────────────────
# Runs in parallel with the frontend build once planner is done.
FROM tools AS server-deps
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --target x86_64-unknown-linux-musl \
    -p server -p resume-generator --recipe-path recipe.json

# ── build server + resume generator, then generate the resume PDFs ────────────
FROM server-deps AS server-builder
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl -p server -p resume-generator
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
RUN npm install && trunk build --release

# ── pull CA certs for future outbound HTTPS calls ─────────────────────────────
FROM alpine:3.23@sha256:25109184c71bdad752c8312a8623239686a9a2071e8825f20acb8f2198c3f659 AS certs
RUN apk add --no-cache ca-certificates

# ── scratch runtime ───────────────────────────────────────────────────────────
FROM scratch AS runtime

ARG USER_ID
ARG GROUP_ID

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
USER ${USER_ID}:${GROUP_ID}
ENTRYPOINT ["/server"]
