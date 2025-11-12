import 'server-only'

import { unstable_cache } from 'next/cache'

import type {
  ContributionPoint,
  GitHubProject,
  UserStats,
} from '@/types/github'

import { fetchContributionData } from './contributions'
import { fetchFeaturedProjects } from './featured-projects'
import { fetchUserStats } from './stats'

const REVALIDATE: Readonly<{
  readonly CONTRIBUTIONS: number
  readonly FEATURED: number
  readonly STATS: number
}> = {
  CONTRIBUTIONS: 3600,
  FEATURED: 86_400,
  STATS: 86_400,
}

export const getFeaturedProjects: () => Promise<GitHubProject[]> = unstable_cache(
  fetchFeaturedProjects,
  ['featured-projects'],
  { revalidate: REVALIDATE.FEATURED }
)

export const getUserStats: () => Promise<UserStats> = unstable_cache(
  fetchUserStats,
  ['user-stats'],
  { revalidate: REVALIDATE.STATS }
)

export const getContributionData: () => Promise<ContributionPoint[]> =
  unstable_cache(fetchContributionData, ['contribution-data'], {
    revalidate: REVALIDATE.CONTRIBUTIONS,
  })
