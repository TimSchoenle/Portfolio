import bundleAnalyzer from '@next/bundle-analyzer'
import type { NextConfig } from 'next'
import createNextIntlPlugin from 'next-intl/plugin'

// Typedef via ReturnType to avoid unused param identifiers
const withNextIntl: ReturnType<typeof createNextIntlPlugin> =
  createNextIntlPlugin({
    experimental: {
      createMessagesDeclaration: './messages/en.json',
    },
  })

const withBundleAnalyzer: ReturnType<typeof bundleAnalyzer> = bundleAnalyzer({
  enabled: process.env.ANALYZE === 'true',
})

const nextConfig: NextConfig = {
  output: 'standalone',

  // Enable strict mode for better development experience
  reactStrictMode: true,

  // Enable typed routes for better TypeScript support
  typedRoutes: true,

  images: {
    formats: ['image/avif', 'image/webp'],
    remotePatterns: [
      { protocol: 'https', hostname: 'github.com' },
      { protocol: 'https', hostname: 'avatars.githubusercontent.com' },
    ],
  },

  experimental: {
    typedEnv: true,
    turbopackFileSystemCacheForDev: true,
    optimizeCss: true,
    optimizePackageImports: ['lucide-react', '@radix-ui/react-icons'],
  },

  async headers(): Promise<
    Array<{
      source: string
      headers: Array<{ key: string; value: string }>
    }>
  > {
    return [
      {
        source: '/:path*',
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
      },
    ]
  },
}

export default withBundleAnalyzer(withNextIntl(nextConfig))
