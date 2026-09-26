# Local tooling. `just` with no arguments lists what there is.
#
# Everything a contributor has to run by hand lives here rather than in a script under
# `.github/scripts/`, so that the command a README quotes, the command CI runs and the command a
# developer types are one string. Recipes that only wrap `cargo` are here for the same reason:
# the flags are the part people get wrong.
#
#     https://github.com/casey/just
#
# There is deliberately no recipe that *checks* the generated artifacts. Checking is
# `TimSchoenle/actions/actions/rust/config-contract`, which does it in three places this file
# cannot reach — against the Dockerfile, against the committed document, and against the labels a
# built image actually carries. A second implementation here would be a second opinion, and the
# whole point of the shared action is that there is only one.

# The configuration generator, and where its output belongs. These five lines are the only
# per-repository part of the contract recipes below.
example := "config-schema"
features := "config-schema"
package := "portfolio-config"
contract := "docs/config.contract.json"
dockerfile := "Dockerfile"

# The markers `--format dockerfile` emits around the LABEL block. Defined by terrace-config, not
# by this repository: cutting the region by line count reads correctly right up until a fourth
# label is added, and then compares two of three lines and passes.
begin := "# terrace-config:labels:begin"
end := "# terrace-config:labels:end"

[private]
default:
    @just --list --unsorted

[doc('Rewrite everything generated from src/config.rs')]
regenerate: contract-json dockerfile-labels

[doc('Print one rendering: json|markdown|markdown-loader|markdown-keys|toml|json-schema|contract|labels|dockerfile')]
[group('generate')]
render format:
    #!/usr/bin/env bash
    set -euo pipefail
    args=(run --quiet --example "{{ example }}")
    [ -n "{{ package }}" ] && args+=(-p "{{ package }}")
    [ -n "{{ features }}" ] && args+=(--features "{{ features }}")
    cargo "${args[@]}" -- --format "{{ format }}"

# Rendered without `--version`, `--revision` or `--created`, so it is byte-reproducible across
# rebuilds and releases: the committed copy describes the configuration surface, and the copy
# inside an image additionally names the build it came from.

[doc('Rewrite the committed contract document')]
[group('generate')]
contract-json:
    #!/usr/bin/env bash
    set -euo pipefail
    mkdir -p "$(dirname "{{ contract }}")"
    just render contract > "{{ contract }}"
    echo "wrote {{ contract }}"

# The file is rebuilt around the markers rather than substituted in place: `sed` cannot replace a
# multi-line block portably, and `--format dockerfile` emits both markers along with the block
# between them.

[doc('Rewrite the LABEL region in the Dockerfile')]
[group('generate')]
dockerfile-labels:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! grep -qF '{{ begin }}' "{{ dockerfile }}" || ! grep -qF '{{ end }}' "{{ dockerfile }}"; then
        echo "error: {{ dockerfile }} carries no '{{ begin }}' … '{{ end }}' region, so the" >&2
        echo "       generated LABEL block has nowhere to go. Paste 'just render dockerfile'" >&2
        echo "       into it once, markers included." >&2
        exit 1
    fi
    block="$(mktemp)"
    rewritten="$(mktemp)"
    trap 'rm -f "$block" "$rewritten"' EXIT
    just render dockerfile > "$block"
    {
        sed -n "1,/^{{ begin }}\$/p" "{{ dockerfile }}" | sed '$d'
        cat "$block"
        sed -n "/^{{ end }}\$/,\$p" "{{ dockerfile }}" | sed '1d'
    } > "$rewritten"
    mv "$rewritten" "{{ dockerfile }}"
    echo "wrote the LABEL region in {{ dockerfile }}"

# The third-party license inventory the `/licenses` page renders. Not a committed artifact: the
# image build runs this same command in its `generate` stage (see the Dockerfile), because the
# attribution a build publishes has to describe the dependency set that build linked. This recipe
# is here so a developer can render the page locally — without it `cargo run` embeds the empty
# default and the page says it has nothing to show.
#
#     cargo install --locked cargo-about
#
# `--all-features` is not optional: the crate's platform features are what pull in the wasm
# client's `web-sys` and the server's axum, and a run without them attributes neither. The
# accepted-license list in apps/web/about.toml is a gate — an unlisted license exits non-zero
# here and fails the image build there.

[doc('Render the third-party license inventory the /licenses page is built from')]
[group('generate')]
licenses:
    #!/usr/bin/env bash
    set -euo pipefail
    mkdir -p apps/web/generated
    cargo about generate --locked --all-features \
        --manifest-path apps/web/Cargo.toml \
        --output-file apps/web/generated/licenses.json \
        apps/web/about.hbs
    echo "wrote apps/web/generated/licenses.json"

# The whole local setup in one command: it generates the resume artifacts on first run, installs
# and builds the stylesheet, watches it, and starts `dx serve` against the legal templates. See
# apps/web/scripts/dev.mjs. The body runs under Node rather than a shell because a POSIX shell is
# not guaranteed on Windows, and Node is a prerequisite anyway.
[doc('Serve the site locally with SSR + hydration and live reload, against the legal templates')]
[group('dev')]
[script('node')]
dev port="8080":
    process.argv[2] = "{{ port }}";
    const { resolve } = require("node:path");
    const { pathToFileURL } = require("node:url");
    import(pathToFileURL(resolve("apps/web/scripts/dev.mjs")).href);

[doc('Format, lint and test — what a pull request is going to run anyway')]
[group('check')]
verify: fmt lint docs test deny

[group('check')]
fmt:
    cargo fmt --all

# Three passes, because `--all-features` alone never compiles what only one platform sees: with
# `web` and `server` both on, every `cfg(all(feature = "web", not(feature = "server")))` item —
# the wasm client's language detection, `dioxus::launch` — is compiled out.

[doc('Clippy over the workspace, then each platform of the web app on its own')]
[group('check')]
lint:
    cargo clippy --workspace --all-features --all-targets -- -D warnings
    cargo clippy -p web --no-default-features --features server --all-targets -- -D warnings
    cargo clippy -p web --no-default-features --features web --target wasm32-unknown-unknown -- -D warnings

[doc('Advisories, licenses, bans and sources, per deny.toml')]
[group('check')]
deny:
    cargo deny --all-features check

# `--workspace` because the workspace root is a package, so without it cargo builds the
# release-please placeholder in `src/lib.rs` and nothing else. `RUSTDOCFLAGS` is where the
# `[workspace.lints.rustdoc]` table is actually enforced: `cargo check` never runs those lints.

[doc('Build the API documentation with the rustdoc lints denied')]
[group('check')]
docs:
    RUSTDOCFLAGS='-D warnings' cargo doc --workspace --all-features --no-deps
    RUSTDOCFLAGS='-D warnings' cargo doc -p web --no-default-features --features web --no-deps
    cargo test --workspace --doc --all-features

[group('check')]
test:
    cargo test --workspace --all-features

# Both need a built image; `docker build -t portfolio:local .` makes one.

[doc('Run the container smoke test against an image')]
[group('image')]
smoke image="portfolio:local" port="8080":
    bash scripts/smoke-test.sh "{{ image }}" "{{ port }}"

[doc('Run the headless-browser hydration test against a running server')]
[group('image')]
browser-smoke base="http://localhost:8080":
    cd tests/browser && npm ci && npx playwright install chromium && node smoke.mjs "{{ base }}"
