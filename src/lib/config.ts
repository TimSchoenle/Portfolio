export interface SiteConfig {
  readonly description: string
  readonly email: string
  readonly featuredRepos: readonly string[]
  readonly fullName: string
  readonly github: string
  readonly githubUsername: string
  readonly name: string
  readonly seo: {
    readonly keywords: readonly string[]
  }
  readonly skills: {
    readonly expertise: readonly string[]
    readonly learning: readonly string[]
    readonly tools: readonly string[]
  }
  readonly title: string
  readonly twitter: string
  readonly url: string
  readonly username: string
}

export const siteConfig: SiteConfig = {
  description:
    'Portfolio of Tim - Software Developer specializing in Java, learning Rust and Next.js. Open-source contributor and passionate about building great software.',
  email: 'contact@timmi6790.de',
  featuredRepos: [
    'cloudflare-access-webhook-redirect',
    's3-bucket-perma-link',
    'Portfolio',
    'helm-charts',
  ],
  fullName: 'Tim',
  github: 'https://github.com/Timmi6790',

  githubUsername: 'Timmi6790',
  name: 'Tim',
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
  skills: {
    expertise: ['Java', 'Spring Boot', 'Maven', 'Gradle'],
    learning: ['Rust', 'Next.js', 'React', 'TypeScript'],
    tools: ['Git', 'GitHub', 'Docker', 'Linux'],
  },

  title: 'Tim - Software Developer',
  twitter: '@Timmi6790',

  url: 'https://timmi6790.de',

  username: 'Timmi6790',
}
