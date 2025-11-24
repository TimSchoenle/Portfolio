export interface Skill {
  readonly confidence: number
  readonly name: string
}

export interface SiteConfig {
  readonly description: string
  readonly email: string
  readonly featuredRepos: readonly string[]
  readonly fullName: string
  readonly github: string
  readonly githubUsername: string
  readonly jobTitle: string
  readonly linkedin?: string
  readonly location: string
  readonly name: string
  readonly resumeDirectory: string
  readonly seo: {
    readonly keywords: readonly string[]
  }
  readonly skills: {
    readonly languages: readonly Skill[]
    readonly frameworks: readonly Skill[]
    readonly buildTools: readonly Skill[]
    readonly infrastructure: readonly Skill[]
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

  jobTitle: 'Software Developer',
  linkedin: '',
  location: 'Germany',
  name: 'Tim',
  resumeDirectory: 'resume',
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
    buildTools: [
      { confidence: 0.8, name: 'Maven' },
      { confidence: 0.8, name: 'Gradle' },
      { confidence: 0.85, name: 'Git' },
      { confidence: 0.8, name: 'GitHub' },
    ],
    frameworks: [
      { confidence: 0.85, name: 'Spring Boot' },
      { confidence: 0.7, name: 'Next.js' },
      { confidence: 0.75, name: 'React' },
    ],
    infrastructure: [
      { confidence: 0.8, name: 'Docker' },
      { confidence: 0.75, name: 'Linux' },
    ],
    languages: [
      { confidence: 0.9, name: 'Java' },
      { confidence: 0.6, name: 'Rust' },
      { confidence: 0.65, name: 'TypeScript' },
    ],
  },
  title: 'Tim - Software Developer',

  twitter: '@Timmi6790',

  url: 'https://timmi6790.de',
  username: 'Timmi6790',
}
