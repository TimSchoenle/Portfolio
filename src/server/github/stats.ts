import { siteConfig } from '@/config/site'
import type { UserStats } from '@/types/github'

import { octokit } from './client'

interface PaginatedRepository {
  readonly forks_count?: number | null
  readonly stargazers_count?: number | null
}

const sumRepositoryMetric: (
  repositories: readonly PaginatedRepository[],
  selector: (repository: PaginatedRepository) => number
) => number = (
  repositories: readonly PaginatedRepository[],
  selector: (repository: PaginatedRepository) => number
): number =>
  repositories.reduce(
    (total: number, repository: PaginatedRepository): number =>
      total + selector(repository),
    0
  )

export const fetchUserStats: () => Promise<UserStats> = async (): Promise<UserStats> => {
  try {
    const repositories: readonly PaginatedRepository[] = await octokit.paginate(
      octokit.repos.listForUser,
      {
        per_page: 100,
        type: 'owner',
        username: siteConfig.githubUsername,
      }
    )

    const stars: number = sumRepositoryMetric(
      repositories,
      (repository: PaginatedRepository): number => repository.stargazers_count ?? 0
    )

    const forks: number = sumRepositoryMetric(
      repositories,
      (repository: PaginatedRepository): number => repository.forks_count ?? 0
    )

    return {
      forks,
      repositories: repositories.length,
      stars,
    }
  } catch {
    return { forks: 0, repositories: 0, stars: 0 }
  }
}
