import 'server-only'
import { Octokit } from '@octokit/rest'
import { unstable_cache } from 'next/cache'

const octokit = new Octokit({
  auth: process.env.GITHUB_TOKEN,
})

export interface GitHubProject {
  name: string
  description: string
  html_url: string
  homepage?: string
  stargazers_count: number
  forks_count: number
  language: string
  topics: string[]
}

const getFeaturedProjectsUncached = async (
  username: string,
  featuredRepos: string[]
): Promise<GitHubProject[]> => {
  try {
    return await Promise.all(
      featuredRepos.map(async (repo) => {
        const { data } = await octokit.repos.get({
          owner: username,
          repo,
        })

        return {
          name: data.name,
          description: data.description || '',
          html_url: data.html_url,
          homepage: data.homepage || undefined,
          stargazers_count: data.stargazers_count,
          forks_count: data.forks_count,
          language: data.language || 'Unknown',
          topics: data.topics || [],
        }
      })
    )
  } catch (error) {
    console.error('Error fetching GitHub projects:', error)
    return []
  }
}

export const getFeaturedProjects = unstable_cache(
  getFeaturedProjectsUncached,
  ['featured-projects'],
  { revalidate: 86400 } // Cache for 24 hours
)

const getUserStatsUncached = async (username: string) => {
  try {
    const { data: repos } = await octokit.repos.listForUser({
      username,
      per_page: 100,
    })

    const totalStars = repos.reduce(
      (acc, repo) => acc + (repo.stargazers_count ?? 0),
      0
    )
    const totalForks = repos.reduce(
      (acc, repo) => acc + (repo.forks_count ?? 0),
      0
    )

    return {
      repositories: repos.length,
      stars: totalStars,
      forks: totalForks,
    }
  } catch (error) {
    console.error('Error fetching GitHub stats:', error)
    return {
      repositories: 0,
      stars: 0,
      forks: 0,
    }
  }
}

export const getUserStats = unstable_cache(
  getUserStatsUncached,
  ['user-stats'],
  { revalidate: 86400 } // Cache for 24 hours
)

export async function fetchContributionGraph(
  username: string
): Promise<string | null> {
  try {
    const response = await fetch(
      `https://ghchart.rshah.org/2563eb/${username}`,
      {
        next: { revalidate: 86400 }, // Cache for 24 hours
      }
    )

    if (!response.ok) {
      console.error('Failed to fetch contribution graph')
      return null
    }

    const arrayBuffer = await response.arrayBuffer()
    const base64 = Buffer.from(arrayBuffer).toString('base64')
    return `data:image/svg+xml;base64,${base64}`
  } catch (error) {
    console.error('Error fetching contribution graph:', error)
    return null
  }
}
