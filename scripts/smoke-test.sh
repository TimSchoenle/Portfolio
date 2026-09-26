#!/usr/bin/env bash
# Functional smoke test for a built image: starts it against the legal templates, then checks
# every route and header contract the server makes. Run locally with `just smoke <image>`; CI
# runs the same script (build.yaml, release-please.yaml).
#
# Usage: scripts/smoke-test.sh <image> [port]
#
# Exit status is non-zero when any check fails; every failure is reported, not only the first.

set -uo pipefail

image="${1:?usage: $0 <image> [port]}"
port="${2:-8080}"
name="portfolio-smoke-$$"
base="http://localhost:${port}"
repo_root="$(cd "$(dirname "$0")/.." && pwd)"

# GitHub annotations on CI, plain text elsewhere.
error() {
  if [ -n "${GITHUB_ACTIONS:-}" ]; then echo "::error::$*"; else echo "FAIL: $*"; fi
}

docker run -d --rm --name "$name" -p "${port}:8080" \
  -v "${repo_root}/legal:/config/legal:ro" \
  -e PORTFOLIO_CONFIG=/config/legal \
  "$image" >/dev/null || { error "could not start $image"; exit 1; }

cleanup() {
  echo "--- container logs ---"
  docker logs "$name" 2>&1 || true
  docker stop "$name" >/dev/null 2>&1 || true
}
trap cleanup EXIT

ready=0
for _ in $(seq 1 60); do
  if curl -fsS "$base/api/health" >/dev/null 2>&1; then ready=1; break; fi
  sleep 1
done
if [ "$ready" != "1" ]; then
  error "container did not become healthy within 60s"
  exit 1
fi

fail=0

expect_status() {
  local path="$1" want="$2" got
  got=$(curl -s -o /dev/null -w '%{http_code}' "$base$path")
  if [ "$got" = "$want" ]; then
    echo "ok: GET $path -> HTTP $got"
  else
    error "GET $path expected HTTP $want, got $got"
    fail=1
  fi
}

expect_body_contains() {
  local path="$1" needle="$2" body
  shift 2
  body=$(curl -fsS "$@" "$base$path" 2>/dev/null || true)
  if grep -qF -- "$needle" <<<"$body"; then
    echo "ok: GET $path body contains '$needle'"
  else
    error "GET $path body did not contain '$needle'"
    fail=1
  fi
}

# The value of response header `name` for `path`; extra curl arguments follow.
header_of() {
  local path="$1" name="$2"
  shift 2
  curl -fsS -D - -o /dev/null "$@" "$base$path" 2>/dev/null \
    | tr -d '\r' \
    | awk -F': ' -v h="$name" 'tolower($1)==tolower(h){print $2; exit}'
}

expect_header() {
  local path="$1" name="$2" want="$3" got
  got=$(header_of "$path" "$name")
  if [ "$got" = "$want" ]; then
    echo "ok: GET $path header '$name: $got'"
  else
    error "GET $path header '$name' expected '$want', got '$got'"
    fail=1
  fi
}

# ── API ──────────────────────────────────────────────────────────────────────
expect_status /api/health 200
expect_body_contains /api/health '"status":"healthy"'
expect_header /api/health cache-control no-store
expect_status /api/v1/profile 200
expect_body_contains /api/v1/profile '"email"'
expect_header /api/v1/profile cache-control 'public, max-age=3600'
expect_status /api/v1/profile/schema 200
expect_body_contains /api/v1/profile/schema '"type":"object"'

# ── legal documents: configuration, rendered server-side, linked from every page ──
expect_status /legal/imprint 200
expect_body_contains /legal/imprint '§ 5 DDG'
expect_body_contains / 'href="/legal/imprint"'
expect_body_contains / 'href="/legal/privacy"'
expect_status /legal/privacy 200
expect_body_contains /legal/privacy 'Art. 4(7) GDPR'
expect_status /imprint 308
expect_status /privacy 308
expect_status /api/v1/legal 200
expect_body_contains /api/v1/legal '"slug":"imprint"'

# ── SSR pages and their language ─────────────────────────────────────────────
expect_status / 200
expect_body_contains / '<html'
expect_status /some/client/route 200
expect_body_contains /some/client/route '<html'
expect_body_contains / '<html lang="en"'
expect_body_contains /some/client/route '<html lang="en"'
expect_body_contains / '<html lang="de"' -H 'Accept-Language: de-DE,de'
expect_body_contains / '<html lang="de"' -H 'Accept-Language: en;q=0.1, de;q=0.9'
expect_body_contains '/?lang=de' '<html lang="de"'
expect_header / vary 'accept-language, cookie'

# The cookie the client falls back to must agree with the HTML on every response — including
# the second, which the render cache answers without running the render that used to set it.
for attempt in first second; do
  cookie=$(header_of / set-cookie -H 'Accept-Language: de')
  case "$cookie" in
    lang=de\;*) echo "ok: $attempt German render sets '$cookie'" ;;
    *) error "$attempt German render set cookie '$cookie', expected lang=de"; fail=1 ;;
  esac
done
cookie=$(header_of / set-cookie -H 'Cookie: lang=de')
if [ -z "$cookie" ]; then
  echo "ok: a cookie that already agrees is not rewritten"
else
  error "a matching lang cookie was rewritten: '$cookie'"
  fail=1
fi

# Every language has its own URL, and each page names all of them.
expect_body_contains / 'hreflang="de"'
expect_body_contains / 'hreflang="x-default"'
expect_body_contains / '?lang=de'

# ── SEO documents and embedded assets ────────────────────────────────────────
expect_status /robots.txt 200
expect_status /sitemap.xml 200
expect_body_contains /sitemap.xml 'xhtml:link rel="alternate" hreflang="de"'
expect_status /site.webmanifest 200
expect_body_contains /site.webmanifest '"sizes": "512x512"'
expect_status /favicon.svg 200
expect_status /icon-192.png 200
expect_status /icon-512.png 200
expect_header /icon-512.png content-type image/png
expect_status /licenses 200
expect_body_contains /licenses 'Permission is hereby granted, free of charge'
expect_body_contains /licenses 'MIT License'
expect_status /og-image.png 200
expect_header /og-image.png content-type image/png
expect_body_contains / '<meta content="image/png" property="og:image:type"/>'

# The hashed client bundle is immutable for a year.
bundle=$(grep -oE '/\./assets/web-dxh[0-9a-f]+\.js' <<<"$(curl -fsS "$base/" || true)" \
  | head -1 | sed 's|^/\.||')
if [ -n "$bundle" ]; then
  expect_header "$bundle" cache-control 'public, max-age=31536000, immutable'
else
  error "no hashed client bundle referenced by the served page"
  fail=1
fi

# An unchanged embedded asset revalidates to 304.
favicon_etag=$(header_of /favicon.svg etag)
if [ -n "$favicon_etag" ]; then
  got=$(curl -s -o /dev/null -w '%{http_code}' -H "If-None-Match: $favicon_etag" "$base/favicon.svg")
  if [ "$got" = "304" ]; then
    echo "ok: an unchanged embedded asset revalidates to 304"
  else
    error "conditional GET /favicon.svg returned $got, expected 304"
    fail=1
  fi
else
  error "/favicon.svg carried no ETag"
  fail=1
fi

# The SVG favicon is text and is compressed; raster images are not.
svg_encoding=$(header_of /favicon.svg content-encoding -H 'Accept-Encoding: gzip')
if [ -n "$svg_encoding" ]; then
  echo "ok: the SVG favicon is compressed ($svg_encoding)"
else
  error "/favicon.svg was served uncompressed"
  fail=1
fi

# ── security headers ─────────────────────────────────────────────────────────
expect_header / x-content-type-options nosniff
expect_header / x-frame-options DENY
expect_header / referrer-policy no-referrer
expect_header / strict-transport-security 'max-age=31536000; includeSubDomains; preload'
expect_header / cross-origin-opener-policy same-origin
expect_header / cross-origin-resource-policy same-origin

# Every inline script on the page is admitted by a hash, and none by 'unsafe-inline'.
page_csp=$(header_of / content-security-policy)
page_html=$(curl -fsS "$base/" || true)
inline_scripts=$(grep -oF '<script>' <<<"$page_html" | wc -l | tr -d ' ')
page_hashes=$(grep -oF "'sha256-" <<<"$page_csp" | wc -l | tr -d ' ')
if [ "$inline_scripts" -gt 0 ] && [ "$page_hashes" = "$inline_scripts" ]; then
  echo "ok: all $inline_scripts inline scripts on / are covered by a hash"
else
  error "/ carries $inline_scripts inline scripts but $page_hashes hashes; a browser will refuse the difference"
  fail=1
fi
script_src=$(tr ';' '\n' <<<"$page_csp" | grep -F 'script-src' || true)
case "$script_src" in
  *"'unsafe-inline'"*) error "script-src still admits all inline script:$script_src"; fail=1 ;;
  *) echo "ok: script-src admits no blanket inline script" ;;
esac

# The Cloudflare nonce is minted per response, memoized page or not.
first_nonce=$(grep -o "'nonce-[^']*'" <<<"$page_csp" || true)
second_nonce=$(grep -o "'nonce-[^']*'" <<<"$(header_of / content-security-policy)" || true)
if [ -n "$first_nonce" ] && [ "$first_nonce" != "$second_nonce" ]; then
  echo "ok: script-src carries a nonce, freshly minted per response"
else
  error "expected a per-response nonce, got '$first_nonce' then '$second_nonce'"
  fail=1
fi
expect_header / cache-control no-cache

# Subresources carry the policy with no inline-script allowance at all.
asset_csp=$(header_of /favicon.svg content-security-policy)
case "$asset_csp" in
  "") error "/favicon.svg carried no Content-Security-Policy"; fail=1 ;;
  *"sha256-"* | *"nonce-"*) error "a subresource carried a document policy: $asset_csp"; fail=1 ;;
  *) echo "ok: subresources carry the policy with no inline-script allowance" ;;
esac

if [ "$fail" != "0" ]; then
  error "container functional smoke tests failed"
  exit 1
fi
echo "All container functional smoke tests passed."
