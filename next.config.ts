import { execSync } from 'node:child_process'

import type { NextConfig } from 'next'

import { PHASE_PRODUCTION_SERVER } from 'next/constants'

import type { Header } from 'next/dist/lib/load-custom-routes'

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

function createStandaloneOutputExclusions(dependencies: string[]): string[] {
  return dependencies.map((element: string): string =>
    createStandaloneOutputExclusion(element)
  )
}

function createStandaloneOutputExclusion(dependency: string): string {
  return `node_modules/.pnpm/${dependency}`
}

function getCspHeader(): string {
  return `
    default-src 'self';
    script-src 'self' 'unsafe-inline' 'unsafe-eval';
    style-src 'self' 'unsafe-inline';
    img-src 'self' blob: data: https:;
    font-src 'self' data: https:;
    object-src 'none';
    base-uri 'self';
    form-action 'self';
    frame-ancestors 'none';
    connect-src 'self' ${process.env.NODE_ENV === 'development' ? 'ws:' : ''};
    worker-src 'self' blob:;
    upgrade-insecure-requests;
  `
    .replaceAll(/\s{2,}/g, ' ')
    .trim()
}

const nextConfig: NextConfig = {
  compiler: {
    removeConsole:
      process.env.NODE_ENV === 'production' ? { exclude: ['error'] } : false,
  },

  env: {
    NEXT_PUBLIC_REVISION: revision,
  },

  experimental: {
    inlineCss: true,
    optimizeCss: true,
    optimizePackageImports: [
      'lucide-react',
      'date-fns',
      '@radix-ui/react-slot',
      'class-variance-authority',
      'clsx',
      'tailwind-merge',
    ],
    sri: {
      algorithm: 'sha512',
    },
    webVitalsAttribution: ['CLS', 'LCP'],
  },

  headers(): Header[] {
    return [
      {
        headers: [
          {
            key: 'Content-Security-Policy',
            value: getCspHeader(),
          },
          { key: 'Referrer-Policy', value: 'no-referrer' },
          { key: 'X-Content-Type-Options', value: 'nosniff' },
          { key: 'X-Frame-Options', value: 'DENY' },
          { key: 'X-Permitted-Cross-Domain-Policies', value: 'none' },
          {
            key: 'Permissions-Policy',
            value:
              'accelerometer=(), camera=(), browsing-topics=(), geolocation=(), gyroscope=(), magnetometer=(), microphone=(), payment=(), usb=(), interest-cohort=(), sync-xhr=(), autoplay=(), display-capture=(), picture-in-picture=()',
          },
          { key: 'Origin-Agent-Cluster', value: '?1' },
          // Security hardening
          {
            key: 'Strict-Transport-Security',
            value: 'max-age=31536000; includeSubDomains; preload',
          },
          { key: 'X-DNS-Prefetch-Control', value: 'off' },
          { key: 'Cross-Origin-Resource-Policy', value: 'same-origin' },
          { key: 'Cross-Origin-Opener-Policy', value: 'same-origin' },
          { key: 'Cross-Origin-Embedder-Policy', value: 'require-corp' },
        ],
        source: '/:path*',
      },
      {
        headers: [
          {
            key: 'Content-Security-Policy',
            value: getCspHeader(),
          },
          {
            key: 'Content-Type',
            value: 'application/javascript; charset=utf-8',
          },
          {
            key: 'Cache-Control',
            value: 'no-cache, no-store, must-revalidate',
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

  // Exclude build-time dependencies from the output bundle (.next/standalone/.node_modules/.pnpm)
  // https://nextjs.org/docs/advanced-features/output-file-tracing
  // The tracing itself is sadly not catching all our dev only dependencies,
  // this causes the docker build to increase in size and trigger our vulnerability scanner.
  // This list needs to be updated whenever a new dependency is added.
  outputFileTracingExcludes: {
    '*': [
      ...createStandaloneOutputExclusions([
        // Babel + AST / source-map tooling
        '@babel+*',
        '@jridgewell+*',
        '@webassemblyjs+*',
        '@xtuc+*',

        // SWC (compiler) – only needed at build time
        '@swc+core@*',
        '@swc+core-*@*',
        // Required during docker image runtime
        // '@swc+helpers@*',

        // Parser / validator / misc build tooling
        'acorn@*',
        'acorn-import-phases@*',
        'ajv@*',
        'ajv-formats@*',
        'ajv-keywords@*',
        'babel-plugin-react-compiler@*',
        'chrome-trace-event@*',
        'es-module-lexer@*',
        'eslint-scope@*',
        'esrecurse@*',
        'estraverse@*',
        'jest-worker@*',
        'schema-utils@*',
        'serialize-javascript@*',

        // esbuild (bundler/minifier)
        'esbuild@*',

        // webpack & friends
        'webpack@*',
        'webpack-sources@*',
        'loader-runner@*',
        'tapable@*',
        'watchpack@*',
        'terser@*',
        'terser-webpack-plugin@*',

        // Browserslist / caniuse / node-releases (build-time env targeting)
        'baseline-browser-mapping@*',
        'browserslist@*',
        'caniuse-lite@*',
        'electron-to-chromium@*',
        'node-releases@*',

        // PostCSS + tiny helpers – build-time only (Tailwind / CSS pipeline)
        'postcss@*',
        'nanoid@*',
        'picocolors@*',
        'source-map-js@*',

        // TypeScript – compile-time only
        'typescript@*',
      ]),
    ],
  },

  poweredByHeader: false,

  reactCompiler: true,

  // Enable strict mode for better development experience
  reactStrictMode: true,

  serverExternalPackages: ['@tailwindcss/oxide'],

  // Enable typed routes for better TypeScript support
  typedRoutes: true,
}

async function applySerwist(config: NextConfig): Promise<NextConfig> {
  // eslint-disable-next-line @typescript-eslint/typedef
  const serwistModule = await import('@serwist/next')
  const withSerwist: (config: NextConfig) => NextConfig = serwistModule.default(
    {
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
    }
  )
  return withSerwist(config)
}

async function applyNextIntl(config: NextConfig): Promise<NextConfig> {
  // eslint-disable-next-line @typescript-eslint/typedef
  const nextIntlModule = await import('next-intl/plugin')
  const withNextIntl: (config: NextConfig) => NextConfig =
    nextIntlModule.default({
      experimental: {
        createMessagesDeclaration: './messages/en.json',
      },
    })
  return withNextIntl(config)
}

function applyBundleAnalyzer(config: NextConfig): NextConfig {
  if (process.env['ANALYZE'] !== 'true') {
    return config
  }

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
  return withBundleAnalyzer(config)
}

async function buildConfig(phase: string): Promise<NextConfig> {
  if (phase === PHASE_PRODUCTION_SERVER) {
    return nextConfig
  }

  let config: NextConfig = nextConfig

  config = await applySerwist(config)
  config = await applyNextIntl(config)
  config = applyBundleAnalyzer(config)

  return config
}

export default buildConfig
