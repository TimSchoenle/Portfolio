/**
 * Site Configuration
 *
 * Centralized configuration for easy customization
 */

export const siteConfig = {
  // Personal Information
  name: 'Tim',
  username: 'Timmi6790',
  title: 'Software Developer',

  // Contact Information
  email: 'contact@timmi6790.de',
  github: 'https://github.com/Timmi6790',
  twitter: '@Timmi6790',

  // Site URLs
  url: 'https://timmi6790.de',

  // GitHub Configuration
  githubUsername: 'Timmi6790',
  featuredRepos: [
    'cloudflare-access-webhook-redirect',
    's3-bucket-perma-link',
    'helm-charts',
  ],

  // SEO Configuration
  seo: {
    keywords: [
      'Tim',
      'Timmi6790',
      'Software Developer',
      'Java',
      'Rust',
      'Next.js',
      'Portfolio',
      'Open Source',
      'Germany',
    ],
    ogImage: '/og-image.png',
    twitterCard: 'summary_large_image',
  },

  // Analytics Configuration (optional)
  analytics: {
    // Google Analytics ID
    googleAnalyticsId: undefined,
    // Plausible domain
    plausibleDomain: undefined,
    // Vercel Analytics (automatically enabled on Vercel)
    vercelAnalytics: true,
  },

  // Features Toggle
  features: {
    commandPalette: true,
    themeToggle: true,
    languageSwitcher: true,
    cookieBanner: true,
    easterEggs: true,
    webVitals: true,
  },
} as const

export type SiteConfig = typeof siteConfig

