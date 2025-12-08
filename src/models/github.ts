/* eslint-disable @typescript-eslint/typedef */
import { z } from 'zod'

export const gitHubProjectSchema = z.object({
  description: z.string(),
  forks_count: z.number(),
  homepage: z.string().optional(),
  html_url: z.url(),
  language: z.string(),
  name: z.string(),
  stargazers_count: z.number(),
  topics: z.array(z.string()),
})

export const userStatsSchema = z.object({
  forks: z.number(),
  repositories: z.number(),
  stars: z.number(),
})

export const contributionLevelSchema = z.union([
  z.literal(0),
  z.literal(1),
  z.literal(2),
  z.literal(3),
  z.literal(4),
])

export const contributionPointSchema = z.object({
  count: z.number(),
  date: z.string(),
  level: contributionLevelSchema,
})

export const CONTRIBUTION_LEVELS: readonly [0, 1, 2, 3, 4] = [
  0, 1, 2, 3, 4,
] as const

/* ---------------------------------- types --------------------------------- */

export type GitHubProject = z.infer<typeof gitHubProjectSchema>
export type UserStats = z.infer<typeof userStatsSchema>
export type ContributionLevel = z.infer<typeof contributionLevelSchema>
export type ContributionPoint = z.infer<typeof contributionPointSchema>
