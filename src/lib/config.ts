/**
 * Site Configuration
 *
 * Centralized configuration for easy customization
 */
export const siteConfig = {
  // Personal Information
  name: 'Tim',
  fullName: 'Tim Schönle',
  description:
    'Portfolio of Tim - Software Developer specializing in Java, learning Rust and Next.js. Open-source contributor and passionate about building great software.',
  username: 'Timmi6790',
  title: 'Tim - Software Developer',

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
  },

  // List of skills to display on the skills section
  skills: {
    expertise: ['Java', 'Spring Boot', 'Maven', 'Gradle'],
    learning: ['Rust', 'Next.js', 'React', 'TypeScript'],
    tools: ['Git', 'GitHub', 'Docker', 'Linux'],
  },
} as const

export type SiteConfig = typeof siteConfig
