# Project data

How `apps/web/repos.json` is fetched, filtered, cached and embedded, and what the projects section reads out of it.

## Where the file comes from

`apps/update-repos` writes it and `apps/web/build.rs` embeds it with `include_str!`. The builder
runs before the web build — inside the image build, and by hand during development. When the file
is absent, `build.rs` substitutes an empty default so the `include_str!` always resolves and the
projects section renders its empty state.

```bash
PORTFOLIO_GITHUB__TOKEN_FILE=/run/secrets/gh_token \
  cargo run --release -p update-repos -- apps/web/repos.json
```

Without a token it still runs, against the anonymous rate limit. The token exists only to lift that
limit; the server never loads it. See the configuration tables in the [README](../README.md).

## What it keeps

The builder lists every repository the owner has (`GET /users/{user}/repos`, paginated) and drops
three kinds:

- Archived repositories.
- Repositories named in `CONFIG.blacklisted_repos`.
- Repositories with no update in the last 365 days (`MAX_AGE_DAYS` in `builder.rs`).

What survives is deserialized straight into the shared `portfolio_data::Repo` and `ReposFile`
models and written as pretty-printed JSON, with failures surfaced through a dedicated
`UpdateReposError`. Sharing the models with `crates/data` is what stops the writer and the reader
from disagreeing about a field name.

An explicit set bypasses the listing and its filtering, and each named repository is fetched
directly:

```bash
PORTFOLIO_GITHUB__REPOS='[Portfolio,actions]' cargo run --release -p update-repos
```

The bracketed array is figment's syntax, so the environment spelling and the TOML spelling stay the
same shape.

## Freshness

A rebuild does not mean a fetch. The builder reads the `generated_at` timestamp out of the existing
file and skips the fetch while the file is younger than the cache TTL: 10 hours when the `CI`
environment variable is set to a non-empty value, 60 minutes otherwise. Deriving freshness from the
file's own timestamp rather than from a marker means it works the same whether the file was written
locally or restored from a CI cache.

CI additionally persists the file across runs with `actions/cache`, keyed by a roughly ten-hour
window, so a fresh copy is usually restored before the build even asks.
