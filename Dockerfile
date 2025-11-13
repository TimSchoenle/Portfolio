# syntax=docker/dockerfile:1.20

FROM node:24-bookworm-slim AS build_base
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

COPY --from=deps /app/node_modules ./node_modules
COPY . .

RUN corepack enable
RUN --mount=type=cache,target=/app/.next/cache pnpm run build
RUN find .next/static -type f -name '*.map' -delete

FROM gcr.io/distroless/nodejs24-debian12:nonroot AS runner
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
