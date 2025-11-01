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
  featuredRepos: readonly string[]
): Promise<GitHubProject[]> => {
  try {
    const projects = await Promise.allSettled(
      featuredRepos.map(async (repo) => {
        const { data } = await octokit.repos.get({
          owner: username,
          repo,
        })

        return {
          name: data.name,
          description: data.description ?? '',
          html_url: data.html_url,
          homepage: data.homepage ?? undefined,
          stargazers_count: data.stargazers_count,
          forks_count: data.forks_count,
          language: data.language ?? 'Unknown',
          topics: data.topics ?? [],
        }
      })
    )

    // Filter out failed requests and return successful ones
    return projects
      .filter(
        (result): result is PromiseFulfilledResult<GitHubProject> =>
          result.status === 'fulfilled'
      )
      .map((result) => result.value)
  } catch (error) {
    console.error('Error fetching GitHub projects:', error)
    // Return empty array as fallback
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
    // Fetch all pages of repositories
    const repos = await octokit.paginate(octokit.repos.listForUser, {
      username,
      per_page: 100,
      type: 'owner',
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
    // Return fallback data instead of zeros
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

interface ContributionDay {
  date: string
  contributionCount: number
  contributionLevel: string
}

interface ContributionWeek {
  contributionDays: ContributionDay[]
}

const getContributionDataUncached = async (
  username: string
): Promise<
  Array<{ date: string; count: number; level: 0 | 1 | 2 | 3 | 4 }>
> => {
  try {
    const to = new Date().toISOString()
    const from = new Date(Date.now() - 365 * 24 * 60 * 60 * 1000).toISOString()

    const query = `
      query($username: String!, $from: DateTime!, $to: DateTime!) {
        user(login: $username) {
          contributionsCollection(from: $from, to: $to) {
            contributionCalendar {
              totalContributions
              weeks {
                contributionDays {
                  date
                  contributionCount
                  contributionLevel
                }
              }
            }
          }
        }
      }
    `

    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
    }
    if (process.env.GITHUB_TOKEN) {
      headers['Authorization'] = `Bearer ${process.env.GITHUB_TOKEN}`
    }

    const response = await fetch('https://api.github.com/graphql', {
      method: 'POST',
      headers: headers,
      body: JSON.stringify({ query, variables: { username, from, to } }),
    })

    if (!response.ok) {
      throw new Error(`GitHub API error: ${response.status}`)
    }

    const data = await response.json()

    if (data.errors) {
      console.error('GraphQL errors:', data.errors)
      return []
    }

    const weeks =
      data.data?.user?.contributionsCollection?.contributionCalendar?.weeks ??
      []

    const levelMap: Record<string, 0 | 1 | 2 | 3 | 4> = {
      NONE: 0,
      FIRST_QUARTILE: 1,
      SECOND_QUARTILE: 2,
      THIRD_QUARTILE: 3,
      FOURTH_QUARTILE: 4,
    }

    const days: Array<{
      date: string
      count: number
      level: 0 | 1 | 2 | 3 | 4
    }> = []
    for (const week of weeks as ContributionWeek[]) {
      for (const day of week.contributionDays) {
        days.push({
          date: day.date,
          count: day.contributionCount,
          level: levelMap[day.contributionLevel] ?? 0,
        })
      }
    }

    return days
  } catch (error) {
    console.error('Error fetching contribution data:', error)
    return []
  }
}

export const getContributionData = unstable_cache(
  getContributionDataUncached,
  ['contribution-data'],
  { revalidate: 3600 } // Cache for 1 hour
)
