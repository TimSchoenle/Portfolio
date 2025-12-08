# syntax=docker/dockerfile:1.20@sha256:26147acbda4f14c5add9946e2fd2ed543fc402884fd75146bd342a7f6271dc1d

FROM node:24-bookworm-slim@sha256:0afb7822fac7bf9d7c1bf3b6e6c496dee6b2b64d8dfa365501a3c68e8eba94b2 AS build_base
WORKDIR /app
ENV PNPM_HOME=/root/.local/share/pnpm
ENV PATH="${PNPM_HOME}:${PATH}"

FROM build_base AS deps
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
  && rm -rf /var/lib/apt/lists/*
COPY package.json pnpm-lock.yaml ./
RUN corepack enable
RUN --mount=type=cache,id=pnpm-store,target=/root/.local/share/pnpm/store \
    pnpm install --frozen-lockfile --prefer-offline

FROM build_base AS builder

ENV NODE_ENV=production NEXT_TELEMETRY_DISABLED=1
ARG GIT_SHA=unknown
ENV GIT_SHA=$GIT_SHA

COPY --from=deps /app/node_modules ./node_modules
COPY . .

RUN corepack enable
RUN --mount=type=cache,target=/app/.next/cache pnpm run build
RUN find .next/static -type f -name '*.map' -delete

FROM gcr.io/distroless/nodejs24-debian12:nonroot@sha256:b5bad30c810389860685e58663b073b89e547ca8d0805cbd881abbacaab6dcfe AS runner
WORKDIR /app

ENV NODE_ENV=production NEXT_TELEMETRY_DISABLED=1 HOSTNAME=0.0.0.0 PORT=3000

COPY --from=builder --chown=nonroot:nonroot /app/public ./public
COPY --from=builder --chown=nonroot:nonroot /app/.next/standalone ./
COPY --from=builder --chown=nonroot:nonroot /app/.next/static ./.next/static

EXPOSE 3000

# Healthcheck hits /api/health and fails on non-200 or error
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD ["/nodejs/bin/node","-e","require('http').get({host:'127.0.0.1',port:process.env.PORT||3000,path:'/api/health'},r=>process.exit(r.statusCode===200?0:1)).on('error',()=>process.exit(1))"]

# Distroless node entry is /nodejs/bin/node; no shell, no package manager
ENTRYPOINT ["/nodejs/bin/node"]
CMD ["server.js"]
