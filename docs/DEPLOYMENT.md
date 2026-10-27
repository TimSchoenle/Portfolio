# Deployment

What the image contains, how it is built reproducibly, what it publishes about its own configuration, and where the chart that runs it lives.

## The image

The runtime stage is `FROM scratch`. There is no shell, no package manager, no CA bundle, no
tzdata and no `/etc/passwd` — a numeric `USER` needs no passwd entry. What gets copied in is the
`server` binary, the `public/` bundle beside it, and `/config/contract.json`. The one outbound
connection the server can make, to Sentry when error reporting is on, verifies TLS against the
`webpki-roots` bundle compiled into the binary, so the missing CA bundle costs nothing.

No legal documents are baked in. The server refuses to start without `legal.*`, and a deployment
mounts its own texts; `legal/` in this repository holds templates only.

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

## How the image is built

The Dockerfile keeps one short comment per instruction. The reasoning behind each stage is here.

### Cross-compilation

Every build stage is pinned to `$BUILDPLATFORM`. Running the Rust, WASM and npm build under QEMU
for a foreign architecture would be an order of magnitude slower, and only the server binary
depends on the architecture. The runtime stage is the exception: it takes the default
`$TARGETPLATFORM`, because it has to carry the requested architecture. Writing
`--platform=$TARGETPLATFORM` out explicitly trips BuildKit's `RedundantTargetPlatform` check.

`musl-tools` provides `musl-gcc` for x86_64, and `gcc-aarch64-linux-gnu` is the arm64 cross
compiler. `musl-gcc` is not a cross compiler: it wraps the builder's own gcc, so an amd64 image
needs an amd64 builder, which then produces both architectures. Both compilers only compile ring's
C sources and drive the link. rustc links its own bundled musl
`libc.a` and crt objects (`self-contained` linking), so the glibc-flavoured cross gcc never
contributes a libc and the result stays static. `libc6-dev-arm64-cross` is easy to miss: the cross
gcc only recommends it, so under `--no-install-recommends` it has no target headers and ring's
first C file fails on `<stdlib.h>`. Its headers are only preprocessed, and nothing from it reaches
the binary. No `libssl-dev` is installed, because ureq and sentry use rustls on ring.

The x86_64 linker is named in `.cargo/config.toml`, which Cargo also reads for the `dx bundle` run
in `apps/web`. The arm64 linker is set by `ENV` in the `web-deps` stage, because its toolchain
exists only on that leg. A linker named in only one of the two is a silent build break.

### Toolchain and build tools

The `rust:<version>-slim` base carries the same `major.minor` as `rust-toolchain.toml`, which
`apps/web/tests/toolchain_pins.rs` asserts. Installing from the file is what fixes the patch
release. Without it, every later stage that runs `COPY . .` would switch to the pin halfway through
the build.

`dx`, `cargo-about` and `cargo-chef` come from cargo-binstall at exact versions. The binstall
bootstrap script is fetched by commit, because it is piped into a shell and `main` would make every
build depend on a repository this one does not control.

### Dependency layers

Compiled dependencies live in layers that `cargo chef cook` produces, not in cache mounts. A cache
mount starts empty on every CI runner, and it hides whatever layer sits under its target path. The
recipe changes only when a manifest or `Cargo.lock` does, so the cooked layers survive every
source-only commit.

Each cook names exactly the package, profile and features of the build it stands in for. Cooking
two packages in one call resolves features across both, while `cargo run -p` resolves them for one,
so every crate whose features differ would compile again after `COPY . .`. For the same reason the
two `web-deps` cooks copy how dioxus-cli 0.7.10 calls cargo (`build/request.rs`): `wasm-release` is
declared in `Cargo.toml` and used as written, `server-release` is defined by `dx` through
`--config` as `release` with `strip = false`, which the `CARGO_PROFILE_*` variables repeat, and the
client gets `--cfg getrandom_backend="wasm_js"`.

`cargo fetch` downloads every crate in `Cargo.lock` into the `deps` layer. `cargo about` reads all
of them, and without the fetch it downloaded the rest itself into a 540 MB layer that changed with
every commit.

`generate`, `contract-builder` and `web-builder` delete the target directory in their last `RUN`.
Only their output files are used, and the compiled workspace crates would otherwise add a new layer
of 50 to 100 MB on every commit. `web-builder` also deletes the 114 MB of tools `dx` downloads into
`~/.local/share/.dx`, because `mode=max` exports that layer.

### Generated data

`portfolio-update-repos` receives the GitHub token as `PORTFOLIO_GITHUB__TOKEN_FILE`, a path,
never as a variable holding the token. A token in the process environment is visible to a crash
dump, `ps e` and every child process. BuildKit omits the secret mount when no secret was supplied,
as on a fork's pull request, so the variable is exported only when the file exists.

`repos.json` is not committed, so without help every build started from nothing and the ten-hour
CI freshness check could never hit. The `/repos-cache` mount carries the file between builds, which
brings the API calls down to one per ten hours per builder. An empty cache is a fresh builder's
normal state, hence `|| true` on the restore. The mount is `sharing=locked` because a release builds
both platforms at once and `tools` differs per `$TARGETARCH`.

The license inventory is generated, not committed, because the attribution a build publishes has to
describe the graph that build linked. `--locked` asserts the committed `Cargo.lock` resolves
unchanged, so the inventory describes what `dx bundle` is about to compile.

The resume generator's optional `resume_photo` secret is the German sheet's application photo. A
portrait is personal data, so it is never a build-context file, and the secret mount exists only
for that `RUN`. The German PDF the generator renders from it is embedded in the server binary and
served under `/resume/`, so an image built with the photo publishes it. The generator reads the format from the file header, so the
extensionless mount works.

The contract has its own stage on `deps`, because it reads nothing `generate` produces, and
`serde_json`, `syn` and the derive that `config-schema` links never reach the runtime image.

### The bundle

`npm ci` runs from `package.json` and `package-lock.json` alone, before `COPY . .`, so the Tailwind
toolchain is not downloaded again on every commit. `.dockerignore` keeps a local `node_modules` out
of the later copy. `--ignore-scripts` is safe because the one install script in the tree,
`@parcel/watcher`'s, serves `watch:css`, which no image build calls, and it denies a compromised
package code execution at install time.

`PORTFOLIO_REQUIRE_GENERATED=1` makes `apps/web/build.rs` refuse an empty default for any input the
earlier stages should have produced, so a missing license inventory or stylesheet fails the image.

`dx bundle --debug-symbols=false` strips the client's DWARF and `name` section, which shrinks the
payload and avoids the malformed `name` section Firefox rejects at validation. The build never
passes `--keep-names`, so no readable backtraces are lost. To size its progress bar, `dx` runs
`cargo +nightly rustc --unit-graph`, and rustup installed whatever nightly was current, about
620 MB, on every build. `RUSTUP_AUTO_INSTALL=0` makes that call fail, and `dx` estimates the unit
count from `cargo metadata` instead. Only the `[n/m]` totals in the log change.

The ISR cache directory is created in `web-builder` and copied into the runtime owned by the
runtime user, because `scratch` has no shell and a non-root user cannot create a directory under
`/`.

### Checking the labels

The contract labels are literal text, so the built image is checked as well as the source:

```bash
docker inspect -f '{{json .Config.Labels}}' "$image" |
  cargo run -p portfolio-config --features config-schema --example verify-labels
```

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

`portfolio-update-repos` runs inside the build and needs the GitHub token. It arrives as a BuildKit secret
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

## Third-party license inventory

`/licenses` renders a document produced by [cargo-about](https://github.com/EmbarkStudios/cargo-about)
during the image build, from `apps/web/about.toml` (which licenses are acceptable, which targets are
built) and `apps/web/about.hbs` (the JSON it writes).

It runs against `apps/web/Cargo.toml` rather than the workspace, because what the page has to report
is the dependency set a visitor actually receives — the WASM client and the SSR server — and not the
resume generator's Typst tree or the repository lister's HTTP client, neither of which ships.
`--all-features` covers both halves of the fullstack crate in one pass, since the client's `web-sys`
and the server's axum sit behind its platform features.

```bash
just licenses
```

The document stays normalized, one entry per distinct license file rather than one per dependency,
and the join happens while rendering. That is what keeps it to 340 KB inside the binary.

The `accepted` list in `about.toml` is a gate rather than a description. `cargo about` exits
non-zero on a license that is not on it and the image build fails, so a dependency arriving under
terms this site cannot ship stops the build instead of being published under a licenses page that
does not mention it. Adding an entry to that list is a deliberate decision to ship under those
terms.
