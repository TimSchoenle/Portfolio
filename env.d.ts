// This file ensures type-safe environment variables

declare global {
  namespace NodeJS {
    interface ProcessEnv {
      // GitHub API Token (optional but recommended)
      GITHUB_TOKEN?: string

      // Next.js built-in
      NODE_ENV: 'development' | 'production' | 'test'

      // Vercel environment variables (if deployed on Vercel)
      VERCEL_URL?: string
      VERCEL_ENV?: 'production' | 'preview' | 'development'
    }
  }
}

// Export empty object to make this a module
export {}

