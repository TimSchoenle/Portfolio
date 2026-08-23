# Deployment

What the image contains, how it is built reproducibly, what it publishes about its own configuration, and where the chart that runs it lives.

## The image

The runtime stage is `FROM scratch`. There is no shell, no package manager, no CA bundle, no
tzdata and no `/etc/passwd` — a numeric `USER` needs no passwd entry. What gets copied in is the
`server` binary, the `public/` bundle beside it, and `/config/contract.json`.

That works because the server is linked against musl and is fully static. Every build stage runs
natively on `$BUILDPLATFORM` and cross-compiles the binary to `$TARGETPLATFORM`; the mapping from
Docker's `TARGETARCH` to the Rust triple is resolved once into `/etc/rust-target`, and an
architecture that is not `amd64` or `arm64` fails there rather than building the wrong thing
further down. Nothing else in the build is architecture-dependent: the client is WASM, and
`repos.json`, the resume PDFs and the Tailwind CSS are data.

`WORKDIR` is `/app` and the server resolves `public/` relative to itself, so the two have to stay
together.

### Incremental static regeneration

The image sets the ISR cache directory to `/tmp/isr` and bakes an empty one owned by the runtime
user, because a non-root process on a `scratch` root cannot create it. Under
`readOnlyRootFilesystem: true` the chart mounts a writable volume there; a plain `docker run` uses
the baked-in copy. Where the path is not writable at all, the server renders every request fresh
instead of failing.

The cache is permanent by default. The only thing that changes a page is a new build, and a new
build starts from an empty cache. A positive TTL is for a persistent cache volume shared across
deploys. Each entry is keyed by the negotiated locale, so the German and English renders of a page
do not overwrite each other.

## The configuration contract

The image describes its own configuration surface twice, so a consumer can find it with or without
a registry.

`/config/contract.json` is the offline copy — a few kilobytes, readable from an exported tarball,
an air-gapped mirror or an init container. The canonical copy is the OCI referrer attached to the
pushed digest. Three labels make either one discoverable without pulling a layer:

```dockerfile
LABEL dev.terrace.config.contract.version="1" \
      dev.terrace.config.contract.path="/config/contract.json" \
      dev.terrace.config.prefix="PORTFOLIO_"
```

That block is generated, not typed. `config-schema --format dockerfile` emits it between two
markers, `just regenerate` rewrites the region between them, and the `Config Contract` job in
**Build** compares the Dockerfile, the committed `docs/config.contract.json` and the labels a built
image actually carries. Delimiting by marker rather than by line count is what makes a fourth label
added upstream get rewritten instead of falling outside the slice.

The committed contract is rendered without `--version`, `--revision` or `--created`. Those move
between builds of the same source, so the committed copy describes the configuration surface and
the copy inside an image additionally names the build it came from.

## Reproducible builds

Every input is pinned:

- Base images by digest, which is also what pins the Rust toolchain.
- The Dioxus CLI and cargo-about to exact versions, through the `DIOXUS_CLI_VERSION` and
  `CARGO_ABOUT_VERSION` build args.
- `cargo` and `dx bundle` run `--locked` against the committed `Cargo.lock`, and the Tailwind
  toolchain runs `npm ci` against `package-lock.json`.

Pass `SOURCE_DATE_EPOCH` for deterministic timestamps, and the four OCI metadata args to make the
image self-describing:

```bash
docker build \
  --build-arg SOURCE_DATE_EPOCH="$(git log -1 --format=%ct)" \
  --build-arg VCS_REF="$(git rev-parse HEAD)" \
  --build-arg VERSION="$(git describe --tags --always)" \
  --build-arg SOURCE_URL=https://github.com/TimSchoenle/Portfolio \
  -t portfolio .
```

`update-repos` runs inside the build and needs the GitHub token. It arrives as a BuildKit secret
mounted as a file, so nothing survives into a layer:

```bash
docker build --secret id=gh_token,src=./gh_token -t portfolio .
```

## Publishing

A release pushes both architectures as one manifest list, so `docker pull` resolves the right image
per node. The push carries max-mode provenance and an SBOM, and `cosign sign --recursive` signs the
index and each per-architecture child manifest — verification then still succeeds for a consumer or
an admission controller that resolved the index down to one architecture before checking.

The chart lives in
[TimSchoenle/helm-charts](https://github.com/TimSchoenle/helm-charts/tree/main/charts/portfolio) and
is bumped by the release workflow with the image **digest** alongside the tag, so a deployment is
pinned to bytes rather than to a moving name.

## Third-party licence inventory

`/licenses` renders a document produced by [cargo-about](https://github.com/EmbarkStudios/cargo-about)
during the image build, from `apps/web/about.toml` (which licences are acceptable, which targets are
built) and `apps/web/about.hbs` (the JSON it writes).

It runs against `apps/web/Cargo.toml` rather than the workspace, because what the page has to report
is the dependency set a visitor actually receives — the WASM client and the SSR server — and not the
resume generator's Typst tree or the repository lister's HTTP client, neither of which ships.
`--all-features` covers both halves of the fullstack crate in one pass, since the client's `web-sys`
and the server's axum sit behind its platform features.

```bash
just licenses
```

The document stays normalised, one entry per distinct licence file rather than one per dependency,
and the join happens while rendering. That is what keeps it to 340 KB inside the binary.

The `accepted` list in `about.toml` is a gate rather than a description. `cargo about` exits
non-zero on a licence that is not on it and the image build fails, so a dependency arriving under
terms this site cannot ship stops the build instead of being published under a licences page that
does not mention it. Adding an entry to that list is a deliberate decision to ship under those
terms.
