# syntax=docker/dockerfile:1@sha256:4edf897a3ffa55b89f906fc8cc78afdb3f1834cc9c7083565e611a8a7d5fe99e

# How each stage is built, and why, is in docs/DEPLOYMENT.md § How the image is built.

ARG USER_ID=1001
ARG GROUP_ID=1001

# `dx` must match the `dioxus` crates in Cargo.lock exactly, because the CLI writes the asset
# manifest and hydration bootstrap the library reads. `apps/web/tests/toolchain_pins.rs` fails
# when they disagree, and Renovate's "dioxus" group moves both together.
# renovate: datasource=crate depName=dioxus-cli
ARG DIOXUS_CLI_VERSION=0.7.10
# renovate: datasource=crate depName=cargo-about
ARG CARGO_ABOUT_VERSION=0.9.2
# renovate: datasource=crate depName=cargo-chef
ARG CARGO_CHEF_VERSION=0.1.78

# ── shared build tools ────────────────────────────────────────────────────────
# Every build stage runs natively on $BUILDPLATFORM and cross-compiles the SSR server to
# $TARGETPLATFORM. Only the server binary is architecture-dependent.
FROM --platform=$BUILDPLATFORM rust:1.99-slim@sha256:aab42391a986424e22d6a3ec14f37bc6dd3dacff4d457ca7cccd781e02297cc6 AS tools

ARG DIOXUS_CLI_VERSION
ARG CARGO_ABOUT_VERSION
ARG CARGO_CHEF_VERSION
ARG TARGETARCH
ARG SOURCE_DATE_EPOCH
ENV SOURCE_DATE_EPOCH=${SOURCE_DATE_EPOCH}

# The one place that maps Docker's TARGETARCH to a Rust musl triple. Any other architecture
# fails here.
RUN case "${TARGETARCH}" in \
      amd64) echo 'x86_64-unknown-linux-musl' ;; \
      arm64) echo 'aarch64-unknown-linux-musl' ;; \
      *) echo "unsupported TARGETARCH: '${TARGETARCH}'" >&2; exit 1 ;; \
    esac > /etc/rust-target

# The C toolchains compile ring and drive the link, while rustc supplies the musl libc, so the
# binary stays fully static. `musl-gcc` wraps the builder's own gcc, so an amd64 image needs an
# amd64 builder. `libc6-dev-arm64-cross` is only recommended by the arm64 cross gcc,
# and without it ring's first C file fails on <stdlib.h>.
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl nodejs npm ca-certificates musl-tools \
    && if [ "${TARGETARCH}" = "arm64" ]; then \
         apt-get install -y --no-install-recommends \
           gcc-aarch64-linux-gnu libc6-dev-arm64-cross; \
       fi \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Installs the exact toolchain `rust-toolchain.toml` pins, copied alone so no other change
# invalidates the layer.
COPY rust-toolchain.toml ./
RUN rustup toolchain install --no-self-update

RUN rustup target add "$(cat /etc/rust-target)"

# The bootstrap script is pinned to a commit because it is piped straight into a shell.
# renovate: datasource=github-tags depName=cargo-bins/cargo-binstall
ARG CARGO_BINSTALL_REF=e00d2c94cc0067b77737821097a62d91c0301baa # tag=v1.21.1
RUN curl -L --proto '=https' --tlsv1.2 -sSf \
    "https://raw.githubusercontent.com/cargo-bins/cargo-binstall/${CARGO_BINSTALL_REF}/install-from-binstall-release.sh" | bash \
    && cargo binstall --no-confirm --locked "dioxus-cli@${DIOXUS_CLI_VERSION}" \
    && cargo binstall --no-confirm --locked "cargo-about@${CARGO_ABOUT_VERSION}" \
    && cargo binstall --no-confirm --locked "cargo-chef@${CARGO_CHEF_VERSION}"

# Compiled dependencies live in the cooked layers below, not in cache mounts, which start empty
# on every CI runner.
ENV CARGO_TARGET_DIR=/cargo-target

# ── dependency recipe ─────────────────────────────────────────────────────────
FROM tools AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ── dependencies of the build-time tools ──────────────────────────────────────
# Each cook names the package, profile and features of the build it stands in for, or a crate
# whose features differ compiles a second time. `cargo fetch` puts every crate `cargo about`
# reads into this layer.
FROM tools AS deps
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --locked --profile tools -p portfolio-update-repos --recipe-path recipe.json \
    && cargo chef cook --locked --profile tools -p portfolio-resume --recipe-path recipe.json \
    && cargo chef cook --locked -p portfolio-config --features config-schema --examples \
         --recipe-path recipe.json \
    && cargo fetch --locked

# ── generate build-time data: repos.json, resume PDFs + fingerprint ───────────
FROM deps AS generate
COPY . .
# The `gh_token` secret is passed as a file path, so the token never enters the process
# environment, and only when BuildKit mounted it. The cache mount carries `repos.json` across
# builds, so `CI=1`'s ten-hour freshness window can skip the API call.
RUN --mount=type=secret,id=gh_token \
    --mount=type=cache,target=/repos-cache,sharing=locked \
    if [ -s /run/secrets/gh_token ]; then \
      export PORTFOLIO_GITHUB__TOKEN_FILE=/run/secrets/gh_token; \
    fi; \
    cp /repos-cache/repos.json apps/web/repos.json 2>/dev/null || true; \
    CI=1 cargo run --profile tools --locked -p portfolio-update-repos -- apps/web/repos.json \
    && cp apps/web/repos.json /repos-cache/repos.json
# The license inventory for `/licenses`, generated from the graph this build links. It is also
# the license gate: a dependency under terms missing from `accepted` in apps/web/about.toml fails
# the image.
RUN mkdir -p apps/web/generated \
    && cargo about generate --locked --all-features \
         --manifest-path apps/web/Cargo.toml \
         --output-file apps/web/generated/licenses.json \
         apps/web/about.hbs
# The resume PDFs, fingerprint, Open Graph card and app icons, which build.rs embeds. The
# optional `resume_photo` secret is the German sheet's application photo, kept out of the
# repository and the build context. The German PDF carries it into the image all the same:
#
#     docker build --secret id=resume_photo,src=./photo.jpg .
RUN --mount=type=secret,id=resume_photo \
    cargo run --profile tools --locked -p portfolio-resume -- /app/resume-out \
      $(test -s /run/secrets/resume_photo && echo "--photo /run/secrets/resume_photo") \
    && mkdir -p apps/web/generated/resume \
    && cp /app/resume-out/resume/*.pdf apps/web/generated/resume/ \
    && cp /app/resume-out/resume-fingerprint.json apps/web/generated/ \
    && cp /app/resume-out/*.png apps/web/generated/ \
    && rm -rf /cargo-target /app/resume-out

# ── the configuration contract ────────────────────────────────────────────────
# Its own stage on `deps`, because it reads nothing `generate` produces. Nothing the
# `config-schema` feature links reaches the runtime image.
FROM deps AS contract-builder
COPY . .
RUN mkdir -p /out \
    && cargo run --locked -p portfolio-config --features config-schema \
         --example config-schema -- --format contract > /out/contract.json \
    && rm -rf /cargo-target

# ── dependencies of the fullstack app ─────────────────────────────────────────
FROM deps AS web-deps
# Per-target C toolchain for ring and the link. The x86_64 linker is set in `.cargo/config.toml`
# instead, and the two must agree.
ENV CC_x86_64_unknown_linux_musl=musl-gcc \
    CC_aarch64_unknown_linux_musl=aarch64-linux-gnu-gcc \
    AR_aarch64_unknown_linux_musl=aarch64-linux-gnu-ar \
    CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=aarch64-linux-gnu-gcc
# The two compilations `dx bundle` runs, cooked exactly the way dioxus-cli 0.7.10 calls cargo.
# A `dx` bump that changes its invocation shows up as `Compiled` lines for dependencies under
# `dx bundle`.
RUN RUSTFLAGS='--cfg getrandom_backend="wasm_js"' \
    cargo chef cook --locked --profile wasm-release --target wasm32-unknown-unknown \
      --no-default-features --features web -p portfolio-web --recipe-path recipe.json \
    && CARGO_PROFILE_SERVER_RELEASE_INHERITS=release \
    CARGO_PROFILE_SERVER_RELEASE_STRIP=false \
    cargo chef cook --locked --profile server-release --target "$(cat /etc/rust-target)" \
      --no-default-features --features server -p portfolio-web --recipe-path recipe.json

# The Tailwind toolchain, from the lockfile alone so the layer survives source changes.
# `--ignore-scripts`, because no install script is needed and none gets to run.
COPY apps/web/package.json apps/web/package-lock.json apps/web/
RUN npm ci --prefix apps/web --ignore-scripts --no-audit --no-fund

# ── build the fullstack app (client wasm + native SSR server) ─────────────────
FROM web-deps AS web-builder
COPY . .
COPY --from=generate /app/apps/web/repos.json apps/web/repos.json
COPY --from=generate /app/apps/web/generated apps/web/generated
WORKDIR /app/apps/web
RUN npm run build:css
# Makes `apps/web/build.rs` fail on any generated input the stages above did not produce.
ENV PORTFOLIO_REQUIRE_GENERATED=1
# Bundles the wasm client and the static musl server. `--debug-symbols=false` drops the client's
# DWARF and `name` section, which Firefox rejects when malformed. `RUSTUP_AUTO_INSTALL=0` stops
# `dx` installing a 620 MB nightly to size its progress bar. Only the bundle leaves this layer.
RUN RUSTUP_AUTO_INSTALL=0 dx bundle --release \
    @client --platform web --debug-symbols=false \
    @server --platform server --target "$(cat /etc/rust-target)" \
    && mkdir -p /out \
    && cp -r /cargo-target/dx/portfolio-web/release/web /out/web \
    && rm -rf /cargo-target /root/.local/share/.dx
# The ISR cache directory, staged here because the `scratch` runtime has no shell to create it.
RUN mkdir -p /isr-cache

# ── runtime ───────────────────────────────────────────────────────────────────
# `scratch` holds nothing but the static server, its assets and the contract. The stage takes
# the default $TARGETPLATFORM, which is the architecture the server was cross-compiled to.
FROM scratch AS runtime

ARG USER_ID
ARG GROUP_ID
ARG VCS_REF=unknown
ARG VERSION=dev
ARG CREATED=unknown
ARG SOURCE_URL
# Generated: `just regenerate` rewrites the region between the markers from the config types, and
# the Config Contract job checks it against the committed contract and a built image. Edit the
# types, not these lines.
# terrace-config:labels:begin
LABEL dev.terrace.config.contract.version="1" \
      dev.terrace.config.contract.path="/config/contract.json" \
      dev.terrace.config.prefix="PORTFOLIO_"
# terrace-config:labels:end
LABEL org.opencontainers.image.title="portfolio" \
      org.opencontainers.image.description="Dioxus fullstack (SSR + hydration) portfolio served by Axum." \
      org.opencontainers.image.url="https://tim-schoenle.de" \
      org.opencontainers.image.source="${SOURCE_URL}" \
      org.opencontainers.image.licenses="LicenseRef-Proprietary" \
      org.opencontainers.image.revision="${VCS_REF}" \
      org.opencontainers.image.version="${VERSION}" \
      org.opencontainers.image.created="${CREATED}"

# The server resolves `public/` relative to itself, so the bundle stays together.
COPY --from=web-builder /out/web /app
# Owned by the runtime user, which cannot create a directory on the read-only root.
COPY --from=web-builder --chown=${USER_ID}:${GROUP_ID} /isr-cache /tmp/isr
COPY --from=contract-builder /out/contract.json /config/contract.json
WORKDIR /app

# PORT, IP and RUST_LOG belong to the Dioxus toolchain, outside the `PORTFOLIO_` namespace. Every
# other key, including the required `legal.*` documents, is described in config.example.toml.
ENV PORT=8080 \
    IP=0.0.0.0 \
    RUST_LOG=info \
    PORTFOLIO_ISR__CACHE_DIR=/tmp/isr

EXPOSE 8080
USER ${USER_ID}:${GROUP_ID}
ENTRYPOINT ["/app/server"]
