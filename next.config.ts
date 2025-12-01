import { execSync } from 'node:child_process'

import type { NextConfig } from 'next'

import withSerwistInit from '@serwist/next'
import createNextIntlPlugin from 'next-intl/plugin'

import type { Header } from 'next/dist/lib/load-custom-routes'

// Typedef via ReturnType to avoid unused param identifiers
const withNextIntl: ReturnType<typeof createNextIntlPlugin> =
  createNextIntlPlugin({
    experimental: {
      createMessagesDeclaration: './messages/en.json',
    },
  })

// Use git commit hash as cache version
const revision: string = (
  process.env['GIT_SHA'] ??
  ((): string => {
    try {
      // eslint-disable-next-line sonarjs/no-os-command-from-path
      return execSync('git rev-parse HEAD', { encoding: 'utf8' })
    } catch {
      return 'unknown'
    }
  })()
)
  .trim()
  .slice(0, 7)

const withSerwist: ReturnType<typeof withSerwistInit> = withSerwistInit({
  additionalPrecacheEntries: [
    { revision: revision, url: '/en' },
    { revision: revision, url: '/de' },
    { revision: revision, url: '/en/imprint' },
    { revision: revision, url: '/de/imprint' },
    { revision: revision, url: '/en/privacy' },
    { revision: revision, url: '/de/privacy' },
  ],
  cacheOnNavigation: true,
  disable: process.env.NODE_ENV === 'development',
  reloadOnOnline: true,
  swDest: 'public/sw.js',
  swSrc: 'src/app/sw.ts',
})

type HeaderValues = Header['headers'][0]['value']

const DEV_CSP: HeaderValues = [
  "default-src 'self'",
  "base-uri 'self'",
  "object-src 'none'",
  "img-src 'self' data: https:",
  "font-src 'self' data: https:",
  "style-src 'self' 'unsafe-inline'",
  "script-src 'self' 'unsafe-inline' 'unsafe-eval' blob: data:",
  "connect-src 'self'",
  "worker-src 'self' blob:",
  "frame-ancestors 'none'",
].join('; ')

const PROD_CSP_FALLBACK: HeaderValues = [
  "default-src 'self'",
  "base-uri 'self'",
  "object-src 'none'",
  "img-src 'self' data: https:",
  "font-src 'self' data: https:",
  "style-src 'self' 'unsafe-inline'",
  "script-src 'self' 'unsafe-inline'",
  "connect-src 'self'",
  "worker-src 'self' blob:",
  "frame-ancestors 'none'",
].join('; ')

const nextConfig: NextConfig = {
  env: {
    NEXT_PUBLIC_REVISION: revision,
  },

  experimental: {
    inlineCss: true,
    optimizeCss: true,
    optimizePackageImports: [
      'lucide-react',
      '@radix-ui/react-icons',
      'clsx',
      'tailwind-merge',
    ],
    turbopackFileSystemCacheForDev: true,
    typedEnv: true,
  },

  headers(): Header[] {
    const isDevelopment: boolean = process.env.NODE_ENV !== 'production'
    const contentSecurityPolicy: HeaderValues = isDevelopment
      ? DEV_CSP
      : PROD_CSP_FALLBACK

    return [
      {
        headers: [
          { key: 'Referrer-Policy', value: 'strict-origin-when-cross-origin' },
          { key: 'X-Content-Type-Options', value: 'nosniff' },
          { key: 'X-Frame-Options', value: 'DENY' },
          {
            key: 'Permissions-Policy',
            value: 'camera=(), microphone=(), geolocation=()',
          },
          // Security hardening
          {
            key: 'Strict-Transport-Security',
            value: 'max-age=31536000; includeSubDomains; preload',
          },
          { key: 'X-DNS-Prefetch-Control', value: 'off' },
          { key: 'Cross-Origin-Resource-Policy', value: 'same-origin' },
          { key: 'Cross-Origin-Opener-Policy', value: 'same-origin' },
          { key: 'Cross-Origin-Embedder-Policy', value: 'require-corp' },
          { key: 'Content-Security-Policy', value: contentSecurityPolicy },
        ],
        source: '/:path*',
      },
      {
        headers: [
          {
            key: 'Content-Type',
            value: 'application/javascript; charset=utf-8',
          },
          {
            key: 'Cache-Control',
            value: 'no-cache, no-store, must-revalidate',
          },
          {
            key: 'Content-Security-Policy',
            value: "default-src 'self'; script-src 'self'",
          },
        ],
        source: '/sw.js',
      },
    ]
  },

  images: {
    formats: ['image/avif', 'image/webp'],
  },

  output: 'standalone',

  poweredByHeader: false,

  reactCompiler: true,

  // Enable strict mode for better development experience
  reactStrictMode: true,

  serverExternalPackages: ['@tailwindcss/oxide'],

  // Enable typed routes for better TypeScript support
  typedRoutes: true,
}

let config: NextConfig = withNextIntl(nextConfig)

if (process.env['ANALYZE'] === 'true') {
  const bundleAnalyzer: (options: {
    enabled: boolean
  }) => (config: NextConfig) => NextConfig =
    // eslint-disable-next-line @typescript-eslint/no-require-imports, unicorn/prefer-module
    require('@next/bundle-analyzer') as (options: {
      enabled: boolean
    }) => (config: NextConfig) => NextConfig

  const withBundleAnalyzer: (config: NextConfig) => NextConfig = bundleAnalyzer(
    {
      enabled: true,
    }
  )
  config = withBundleAnalyzer(config)
}

export default withSerwist(config)
