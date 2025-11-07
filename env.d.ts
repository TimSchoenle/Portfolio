// This file ensures type-safe environment variables

// eslint-disable-next-line unicorn/prevent-abbreviations
declare global {
  namespace NodeJS {
    interface ProcessEnvironment {
      // Enable bundle analyzer
      readonly ANALYZE?: 'false' | 'true'

      // GitHub API Token (optional but recommended)
      readonly GITHUB_TOKEN?: string

      // Next.js built-in
      readonly NODE_ENV: 'development' | 'production' | 'test'
    }
  }
}

// Export empty object to make this a module
export {}
