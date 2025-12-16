import { z } from 'zod'

export const CONTRIBUTION_LEVELS: readonly [0, 1, 2, 3, 4] = [
  0, 1, 2, 3, 4,
] as const

/* ---------------------------------- types --------------------------------- */

export type ContributionLevel = 0 | 1 | 2 | 3 | 4

export interface GitHubProject {
  description: string
  forks_count: number
  homepage?: string | undefined
  html_url: string
  language: string
  name: string
  stargazers_count: number
  topics: string[]
}

export interface UserStats {
  forks: number
  repositories: number
  stars: number
}

export interface ContributionPoint {
  count: number
  date: string
  level: ContributionLevel
}

export type ContributionCollection = Record<number, ContributionPoint[]>

/* --------------------------------schemas ---------------------------------- */

export const contributionLevelSchema: z.ZodType<ContributionLevel> = z.union([
  z.literal(0),
  z.literal(1),
  z.literal(2),
  z.literal(3),
  z.literal(4),
])

// eslint-disable-next-line @typescript-eslint/no-deprecated, sonarjs/deprecation
const urlSchema: z.ZodString = z.string().url()

export const gitHubProjectSchema: z.ZodType<GitHubProject> = z.object({
  description: z.string(),
  forks_count: z.number(),
  homepage: z.string().optional(),
  html_url: urlSchema,
  language: z.string(),
  name: z.string(),
  stargazers_count: z.number(),
  topics: z.array(z.string()),
})

export const userStatsSchema: z.ZodType<UserStats> = z.object({
  forks: z.number(),
  repositories: z.number(),
  stars: z.number(),
})

export const contributionPointSchema: z.ZodType<ContributionPoint> = z.object({
  count: z.number(),
  date: z.string(),
  level: contributionLevelSchema,
})
