/* eslint-disable max-lines */

export const SKILL_RENDER_AREAS: {
  readonly RESUME: 'resume'
  readonly SECTION: 'section'
  readonly TECH_RADAR: 'tech-radar'
} = {
  RESUME: 'resume',
  SECTION: 'section',
  TECH_RADAR: 'tech-radar',
} as const

export type SkillRenderArea =
  (typeof SKILL_RENDER_AREAS)[keyof typeof SKILL_RENDER_AREAS]

export interface Skill {
  readonly confidence: number
  readonly name: string
  readonly renderAreas?: SkillRenderArea[]
}

export interface SiteConfig {
  readonly description: string
  readonly email: string
  readonly featuredRepos: readonly string[]
  readonly fullName: string
  readonly github: string
  readonly githubUsername: string
  readonly jobTitle: string
  readonly legals: {
    readonly vatId: string
    readonly address: string
    readonly logRetentionDays: number
    readonly serverLocationCountry: string
    readonly imprintLastChange: Date
    readonly privacyPolicyLastChange: Date
    readonly cloudflare: {
      readonly address: string
      readonly policyUrl: string
    }
  }
  readonly linkedin?: string
  readonly location: string
  readonly name: string
  readonly resumeDirectory: string
  readonly seo: {
    readonly keywords: readonly string[]
  }
  readonly skills: {
    readonly resumeMinimumConfidence: number
    readonly sectionSideMinimumConfidence: number
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
  email: 'contact@tim-schoenle.de',
  featuredRepos: [
    'cloudflare-access-webhook-redirect',
    's3-bucket-perma-link',
    'Portfolio',
    'helm-charts',
  ],
  fullName: 'Tim Schönle',
  github: 'https://github.com/timschoenle',
  githubUsername: 'timschoenle',
  jobTitle: 'Software Developer',
  legals: {
    address: '#TODO IMPLEMENT ME',
    cloudflare: {
      address:
        'Cloudflare, Inc., 101 Townsend St, San Francisco, CA 94107, USA',
      policyUrl: 'https://www.cloudflare.com/privacypolicy/',
    },
    imprintLastChange: new Date('2025-11-30'),
    logRetentionDays: 30,
    privacyPolicyLastChange: new Date('2025-11-30'),
    serverLocationCountry: 'Germany',
    vatId: '#TODO ADD ME',
  },
  linkedin: '',
  location: 'Germany',
  name: 'Tim',
  resumeDirectory: 'resume',
  seo: {
    keywords: [
      'Tim',
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
      { confidence: 0.9, name: 'Gradle' },
      { confidence: 0.9, name: 'Git' },
      { confidence: 0.9, name: 'GitHub Actions' },
      {
        confidence: 0.85,
        name: 'pnpm',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.8,
        name: 'ESLint',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.8,
        name: 'Prettier',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      { confidence: 0.6, name: 'Playwright' },
      {
        confidence: 0.75,
        name: 'Renovate',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.75,
        name: 'release-please',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.5,
        name: 'commitlint',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.5,
        name: 'Knip',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.85,
        name: 'Checkstyle',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      { confidence: 0.85, name: 'JUnit' },
      { confidence: 0.85, name: 'Mockito' },
      {
        confidence: 0.6,
        name: 'npm',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.55,
        name: 'Jest',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.4,
        name: 'Vitest',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.7,
        name: 'Flyway',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.5,
        name: 'Husky',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.45,
        name: 'lint-staged',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.6,
        name: 'Cargo',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.65,
        name: 'pre-commit',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.65,
        name: 'JaCoCo',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.75,
        name: 'SonarQube',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.7,
        name: 'Testcontainers',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.75,
        name: 'Docker Buildx',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
    ],
    frameworks: [
      { confidence: 0.86, name: 'Spring Boot' },
      { confidence: 0.8, name: 'Next.js' },
      { confidence: 0.76, name: 'React' },
      { confidence: 0.75, name: 'Tailwind CSS' },
      { confidence: 0.82, name: 'gRPC' },
      { confidence: 0.65, name: 'Node.js' },
      { confidence: 0.85, name: 'PaperMC' },
      {
        confidence: 0.7,
        name: 'shadcn/ui',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.6,
        name: 'Serwist',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.6,
        name: 'Express',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.4,
        name: 'tRPC',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.5,
        name: 'Prisma',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.55,
        name: 'React Query',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.72,
        name: 'Zod',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.45,
        name: 'NextAuth.js',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.6,
        name: 'Tokio',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.45,
        name: 'Axum',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.45,
        name: 'Actix Web',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.3,
        name: 'Rocket',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.95,
        name: 'Bukkit API',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.95,
        name: 'Spigot API',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.75,
        name: 'next-intl',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.7,
        name: 'Lucide React',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.5,
        name: 'Radix UI',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.4,
        name: 'Lineicons',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.5,
        name: 'ratatui',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.55,
        name: 'reqwest',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.4,
        name: 'sqlx',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.35,
        name: 'wasm-bindgen',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
    ],
    infrastructure: [
      { confidence: 0.85, name: 'Docker' },
      { confidence: 0.8, name: 'Kubernetes' },
      { confidence: 0.8, name: 'ArgoCD' },
      { confidence: 0.75, name: 'Linux' },
      { confidence: 0.8, name: 'PostgreSQL' },
      { confidence: 0.75, name: 'TimescaleDB' },
      { confidence: 0.65, name: 'MongoDB' },
      { confidence: 0.6, name: 'Redis' },
      { confidence: 0.8, name: 'Helm' },
      { confidence: 0.8, name: 'Docker Compose' },
      { confidence: 0.7, name: 'MariaDB' },
      {
        confidence: 0.7,
        name: 'MySQL',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.65,
        name: 'SQLite',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.65,
        name: 'Prometheus',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.7,
        name: 'Sentry',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.65,
        name: 'Codecov',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.5,
        name: 'Grafana',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.4,
        name: 'Loki',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.45,
        name: 'Apache Kafka',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.35,
        name: 'RabbitMQ',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.5,
        name: 'Nginx',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.75,
        name: 'Traefik',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.45,
        name: 'Cloudflare Workers',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.6,
        name: 'Cloudflare Tunnels',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.35,
        name: 'Elasticsearch',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.8,
        name: 'OpenTelemetry',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.3,
        name: 'Pingora',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },

      {
        confidence: 0.55,
        name: 'AWS S3',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.6,
        name: 'Webhooks',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.5,
        name: 'Docker Hub',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.45,
        name: 'Reverse Proxies (general)',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
    ],
    languages: [
      { confidence: 0.95, name: 'Java' },
      { confidence: 0.7, name: 'Rust' },
      { confidence: 0.6, name: 'TypeScript' },
      { confidence: 0.5, name: 'JavaScript' },
      { confidence: 0.3, name: 'Python' },
      { confidence: 0.25, name: 'Lua' },
      { confidence: 0.85, name: 'SQL' },
      { confidence: 0.1, name: 'C++' },
      { confidence: 0.2, name: 'Go' },
      { confidence: 0.35, name: 'C#' },
      { confidence: 0.55, name: 'Kotlin' },
      {
        confidence: 0.45,
        name: 'CSS',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.7,
        name: 'Markdown',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.5,
        name: 'Bash / Shell',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.5,
        name: 'HTML',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.6,
        name: 'TOML',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.75,
        name: 'Dockerfile',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.7,
        name: 'Regular Expressions',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },

      {
        confidence: 0.35,
        name: 'WebAssembly (WASM)',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
      {
        confidence: 0.5,
        name: 'Properties / INI',
        renderAreas: [SKILL_RENDER_AREAS.TECH_RADAR],
      },
    ],
    resumeMinimumConfidence: 0.6,
    sectionSideMinimumConfidence: 0.6,
  },
  title: 'Tim Schönle - Software Developer',

  twitter: '@Timmi6790',

  url: 'https://tim-schoenle.de',
  username: 'timschoenle',
}
