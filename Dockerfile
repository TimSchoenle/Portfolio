# syntax=docker/dockerfile:1.24@sha256:87999aa3d42bdc6bea60565083ee17e86d1f3339802f543c0d03998580f9cb89

FROM dhi.io/bun:1-alpine3.22-dev@sha256:cd852cc01ae5f07a10d61854900fc38ed907c252736e7c6fbcb6da4cd2d9e473 AS build_base
WORKDIR /app

FROM build_base AS deps
COPY package.json bun.lock ./
RUN --mount=type=cache,id=bun-store,target=/root/.bun/install/cache \
    bun install --frozen-lockfile --production

FROM build_base AS dev_deps
COPY package.json bun.lock ./
RUN --mount=type=cache,id=bun-store,target=/root/.bun/install/cache \
    bun install --frozen-lockfile

FROM build_base AS builder

ARG GIT_SHA=unknown
ARG SENTRY_RELEASE
ENV NODE_ENV=production \
    NEXT_TELEMETRY_DISABLED=1 \
    GIT_SHA=$GIT_SHA \
    SENTRY_RELEASE=${SENTRY_RELEASE:-$GIT_SHA}

COPY --from=dev_deps /app/node_modules ./node_modules
COPY . .

# Build the Next.js app
# Use secret mounts for sensitive data during the build
# Remove source maps after build to reduce image size
# Remove any stray node_modules that might have been copied
RUN --mount=type=cache,target=/app/.next/cache \
    --mount=type=secret,id=resume_signing_cert_base64 \
    --mount=type=secret,id=resume_signing_cert_password \
    --mount=type=secret,id=SENTRY_AUTH_TOKEN \
    --mount=type=secret,id=SENTRY_ORG \
    --mount=type=secret,id=SENTRY_PROJECT \
    if [ -f /run/secrets/SENTRY_AUTH_TOKEN ]; then \
      SENTRY_AUTH_TOKEN="$(cat /run/secrets/SENTRY_AUTH_TOKEN)"; \
      export SENTRY_AUTH_TOKEN; \
    fi && \
    if [ -f /run/secrets/SENTRY_ORG ]; then \
      SENTRY_ORG="$(cat /run/secrets/SENTRY_ORG)"; \
      export SENTRY_ORG; \
    fi && \
    if [ -f /run/secrets/SENTRY_PROJECT ]; then \
      SENTRY_PROJECT="$(cat /run/secrets/SENTRY_PROJECT)"; \
      export SENTRY_PROJECT; \
    fi && \
    bun run build && \
    find .next/static -type f -name '*.map' -delete && \
    rm -rf node_modules



FROM dhi.io/bun:1-alpine3.22@sha256:b169efde3cf30151d66f3d7988cad69b4d08833cc4cfaeca7da6bda2bd0a89b3 AS runner
WORKDIR /app
USER nonroot

ENV NODE_ENV=production \
    NEXT_TELEMETRY_DISABLED=1 \
    HOSTNAME=0.0.0.0 \
    PORT=3000

COPY --from=builder --chown=nonroot:nonroot /app/public ./public

# Those directories need to be readble for ISR to work
COPY --from=builder --chown=nonroot:nonroot /app/.next/standalone ./
COPY --from=builder --chown=nonroot:nonroot /app/.next/static ./.next/static
# server.js executable only
COPY --from=builder --chown=nonroot:nonroot --chmod=500 /app/.next/standalone/server.js ./server.js

# Fix permissions using dedicated script
COPY --from=builder --chown=nonroot:nonroot /app/scripts/docker/fix-public-permissions.ts /tmp/fix-public-permissions.ts
RUN ["bun", "/tmp/fix-public-permissions.ts"]

EXPOSE 3000

# Healthcheck hits /api/health and fails on non-200 or error
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD ["bun","-e","require('http').get({host:'127.0.0.1',port:process.env.PORT||3000,path:'/api/health'},r=>process.exit(r.statusCode===200?0:1)).on('error',()=>process.exit(1))"]

ENTRYPOINT ["bun"]
CMD ["server.js"]
