#!/usr/bin/env bash
# The mechanical half of the prose and doc-comment standards in TimSchoenle/actions:
#
#   https://github.com/TimSchoenle/actions/blob/main/docs/readme/PROSE.md#checks
#   https://github.com/TimSchoenle/actions/blob/main/docs/doc-comments/GUIDE.md#checks
#
# Three budgets, each copied from those documents: no doc comment opens with a banned opener, a
# Markdown document carries at most five em dashes outside tables, code and quotes, and at most
# three consecutive bullets share the `**Bold** —` frame. Everything else in those standards is a
# review criterion and is not checked here.
#
# Usage: scripts/prose-check.sh [markdown file...]   (default: README.md)
# Run locally with `just prose`; the `README` job in build.yaml runs the same script.
set -euo pipefail

cd "$(dirname "$0")/.."

readonly EM_DASH_BUDGET=5
readonly BOLD_FRAME_BUDGET=3

failed=0

# GUIDE.md's opener check, in POSIX ERE so it needs nothing the runner image lacks. Its `rg`
# spelling skips `target/`, `dist/` and `node_modules/`; so does this.
opener='^[[:space:]]*(///|\*|//!)[[:space:]]*(This (function|method|struct|class|enum|trait|module|field)|Used to |Helper (for|that)|Responsible for|A utility|Note that|In other words|Under the hood|Simply |Basically )'
if openers="$(grep -rnE "$opener" --include='*.rs' \
    --exclude-dir=target --exclude-dir=dist --exclude-dir=node_modules --exclude-dir=.git .)"; then
    echo "error: doc comments open with a phrase GUIDE.md bans (budget 0):" >&2
    echo "$openers" >&2
    failed=1
fi

# PROSE.md's counts, verbatim apart from reading one file at a time.
em_dashes() {
    awk '/^```/{f=!f; next} f{next} /<!--/{c=1} c{if (/-->/) c=0; next} 1' "$1" \
        | sed -e '/^|/d' -e '/^>/d' -e '/^[0-9]\+\. /d' -e 's/`[^`]*`//g' \
        | { grep -o '—' || true; } | wc -l
}

bold_frame_run() {
    awk '/^- \*\*.*\*\* —/{r++; if (r>m) m=r; next} {r=0} END{print m+0}' "$1"
}

if [ "$#" -eq 0 ]; then
    set -- README.md
fi

for file in "$@"; do
    dashes="$(em_dashes "$file")"
    if [ "$dashes" -gt "$EM_DASH_BUDGET" ]; then
        echo "error: $file carries $dashes em dashes outside tables, code and quotes (budget $EM_DASH_BUDGET)" >&2
        failed=1
    fi
    run="$(bold_frame_run "$file")"
    if [ "$run" -gt "$BOLD_FRAME_BUDGET" ]; then
        echo "error: $file has $run consecutive '**Bold** —' bullets (budget $BOLD_FRAME_BUDGET)" >&2
        failed=1
    fi
done

exit "$failed"
