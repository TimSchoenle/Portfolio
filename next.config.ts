import type { NextConfig } from 'next'

import bundleAnalyzer from '@next/bundle-analyzer'
import createNextIntlPlugin from 'next-intl/plugin'

import type { Header } from 'next/dist/lib/load-custom-routes'

// Typedef via ReturnType to avoid unused param identifiers
const withNextIntl: ReturnType<typeof createNextIntlPlugin> =
  createNextIntlPlugin({
    experimental: {
      createMessagesDeclaration: './messages/en.json',
    },
  })

const withBundleAnalyzer: ReturnType<typeof bundleAnalyzer> = bundleAnalyzer({
  enabled: process.env['ANALYZE'] === 'true',
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
    ]
  },

  images: {
    formats: ['image/avif', 'image/webp'],
  },

  output: 'standalone',

  poweredByHeader: false,

  // Enable strict mode for better development experience
  reactStrictMode: true,

  serverExternalPackages: ['@tailwindcss/oxide'],

  // Enable typed routes for better TypeScript support
  typedRoutes: true,
}

export default withBundleAnalyzer(withNextIntl(nextConfig))
