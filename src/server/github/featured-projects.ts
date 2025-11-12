import { siteConfig } from '@/config/site'
import type { GitHubProject } from '@/types/github'

import { octokit } from './client'

const mapRepositoryToProject: (
  repository: Awaited<ReturnType<typeof octokit.repos.get>>
) => GitHubProject = (
  repository: Awaited<ReturnType<typeof octokit.repos.get>>
): GitHubProject => ({
  description: repository.data.description ?? '',
  forks_count: repository.data.forks_count,
  homepage: repository.data.homepage ?? undefined,
  html_url: repository.data.html_url,
  language: repository.data.language ?? 'Unknown',
  name: repository.data.name,
  stargazers_count: repository.data.stargazers_count,
  topics: Array.isArray(repository.data.topics)
    ? repository.data.topics
    : [],
})

const resolveFeaturedRepository: (
  repositoryName: string
) => Promise<GitHubProject> = async (
  repositoryName: string
): Promise<GitHubProject> => {
  const response: Awaited<ReturnType<typeof octokit.repos.get>> =
    await octokit.repos.get({
      owner: siteConfig.githubUsername,
      repo: repositoryName,
    })

  return mapRepositoryToProject(response)
}

export const fetchFeaturedProjects: () => Promise<GitHubProject[]> = async (): Promise<GitHubProject[]> => {
  try {
    const repositories: PromiseSettledResult<GitHubProject>[] =
      await Promise.allSettled(
        siteConfig.featuredRepos.map(
          (repositoryName: string): Promise<GitHubProject> =>
            resolveFeaturedRepository(repositoryName)
        )
      )

    return repositories
      .filter(
        (
          result: PromiseSettledResult<GitHubProject>
        ): result is PromiseFulfilledResult<GitHubProject> =>
          result.status === 'fulfilled'
      )
      .map(
        (result: PromiseFulfilledResult<GitHubProject>): GitHubProject =>
          result.value
      )
  } catch {
    return []
  }
}
