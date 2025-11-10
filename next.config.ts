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

  images: {
    formats: ['image/avif', 'image/webp'],
  },

  output: 'standalone',

  // Enable strict mode for better development experience
  reactStrictMode: true,

  // Enable typed routes for better TypeScript support
  typedRoutes: true,
}

export default withBundleAnalyzer(withNextIntl(nextConfig))
