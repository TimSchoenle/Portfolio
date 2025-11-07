import type { NextConfig } from 'next'

import bundleAnalyzer from '@next/bundle-analyzer'
import createNextIntlPlugin from 'next-intl/plugin'

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

const nextConfig: NextConfig = {
  experimental: {
    optimizeCss: true,
    optimizePackageImports: ['lucide-react', '@radix-ui/react-icons'],
    turbopackFileSystemCacheForDev: true,
    typedEnv: true,
  },

  headers(): {
    source: string
    headers: { key: string; value: string }[]
  }[] {
    return [
      {
        headers: [
          { key: 'X-DNS-Prefetch-Control', value: 'on' },
          {
            key: 'Strict-Transport-Security',
            value: 'max-age=63072000; includeSubDomains; preload',
          },
          { key: 'X-Frame-Options', value: 'SAMEORIGIN' },
          { key: 'X-Content-Type-Options', value: 'nosniff' },
          { key: 'Referrer-Policy', value: 'origin-when-cross-origin' },
          {
            key: 'Permissions-Policy',
            value: 'camera=(), microphone=(), geolocation=()',
          },
        ],
        source: '/:path*',
      },
    ]
  },

  images: {
    formats: ['image/avif', 'image/webp'],
    remotePatterns: [
      { hostname: 'github.com', protocol: 'https' },
      { hostname: 'avatars.githubusercontent.com', protocol: 'https' },
    ],
  },

  output: 'standalone',

  // Enable strict mode for better development experience
  reactStrictMode: true,

  // Enable typed routes for better TypeScript support
  typedRoutes: true,
}

export default withBundleAnalyzer(withNextIntl(nextConfig))
