import 'server-only'
import { Octokit } from '@octokit/rest'
import { unstable_cache } from 'next/cache'

import { siteConfig } from '@/lib/config'
import type {
  ContributionLevel,
  ContributionPoint,
  GitHubProject,
  UserStats,
} from '@/types/github'

/* ----------------------------- shared interfaces ---------------------------- */
interface GraphQLCalendarDay {
  readonly contributionCount: number
  readonly contributionLevel:
    | 'FIRST_QUARTILE'
    | 'FOURTH_QUARTILE'
    | 'NONE'
    | 'SECOND_QUARTILE'
    | 'THIRD_QUARTILE'
  readonly date: string
}

interface GraphQLCalendarWeek {
  readonly contributionDays: readonly GraphQLCalendarDay[]
}

interface GraphQLCalendar {
  readonly weeks: readonly GraphQLCalendarWeek[]
}

interface GraphQLUser {
  readonly contributionsCollection: {
    readonly contributionCalendar: GraphQLCalendar
  }
}

interface GraphQLResponse {
  readonly data: {
    readonly user: GraphQLUser | null | undefined
  }
  readonly errors?: readonly unknown[]
}

/* --------------------------------- octokit --------------------------------- */

const octokit: Octokit = new Octokit({
  auth: process.env['GITHUB_TOKEN'],
})

/* ---------------------------- featured repositories --------------------------- */
const getFeaturedProjectsUncached: () => Promise<
  GitHubProject[]
> = async (): Promise<GitHubProject[]> => {
  try {
    const projects: PromiseSettledResult<GitHubProject>[] =
      await Promise.allSettled(
        siteConfig.featuredRepos.map(
          async (repo: string): Promise<GitHubProject> => {
            const resp: Awaited<ReturnType<typeof octokit.repos.get>> =
              await octokit.repos.get({
                owner: siteConfig.githubUsername,
                repo,
              })

            return {
              description: resp.data.description ?? '',
              forks_count: resp.data.forks_count,
              homepage: resp.data.homepage ?? undefined,
              html_url: resp.data.html_url,
              language: resp.data.language ?? 'Unknown',
              name: resp.data.name,
              stargazers_count: resp.data.stargazers_count,
              topics: Array.isArray(resp.data.topics) ? resp.data.topics : [],
            }
          }
        )
      )

    // Keep only fulfilled results
    return projects
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

export const getFeaturedProjects: () => Promise<GitHubProject[]> =
  unstable_cache(getFeaturedProjectsUncached, ['featured-projects'], {
    revalidate: 86_400, // 24h
  })

/* -------------------------------- user stats -------------------------------- */

const getUserStatsUncached: () => Promise<UserStats> =
  async (): Promise<UserStats> => {
    try {
      const repos: readonly {
        readonly stargazers_count?: number | null
        readonly forks_count?: number | null
      }[] = await octokit.paginate(octokit.repos.listForUser, {
        per_page: 100,
        type: 'owner',
        username: siteConfig.githubUsername,
      })

      const totalStars: number = repos.reduce(
        (
          accumulator: number,
          repo: { readonly stargazers_count?: number | null }
        ): number => accumulator + (repo.stargazers_count ?? 0),
        0
      )

      const totalForks: number = repos.reduce(
        (
          accumulator: number,
          repo: { readonly forks_count?: number | null }
        ): number => accumulator + (repo.forks_count ?? 0),
        0
      )

      return {
        forks: totalForks,
        repositories: repos.length,
        stars: totalStars,
      }
    } catch {
      return { forks: 0, repositories: 0, stars: 0 }
    }
  }

export const getUserStats: () => Promise<UserStats> = unstable_cache(
  getUserStatsUncached,
  ['user-stats'],
  { revalidate: 86_400 }
)

/* ------------------------ contributions (GraphQL API) ----------------------- */

const buildContributionQuery: () => string = (): string => `
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

const buildHeaders: () => Record<string, string> = (): Record<
  string,
  string
> => {
  const headers: Record<string, string> = { 'Content-Type': 'application/json' }
  const token: string | undefined = process.env['GITHUB_TOKEN']
  if (token !== undefined && token !== '') {
    headers['Authorization'] = `Bearer ${token}`
  }
  return headers
}

const isGraphQLResponse: (value: unknown) => value is GraphQLResponse = (
  value: unknown
): value is GraphQLResponse => {
  if (typeof value !== 'object' || value === null) {
    return false
  }
  const object: Record<string, unknown> = value as Record<string, unknown>
  return 'data' in object
}

const levelToInt: (
  level: GraphQLCalendarDay['contributionLevel']
) => ContributionLevel = (
  level: GraphQLCalendarDay['contributionLevel']
): ContributionLevel => {
  switch (level) {
    case 'NONE': {
      return 0
    }
    case 'FIRST_QUARTILE': {
      return 1
    }
    case 'SECOND_QUARTILE': {
      return 2
    }
    case 'THIRD_QUARTILE': {
      return 3
    }
    case 'FOURTH_QUARTILE': {
      return 4
    }
    default: {
      return 0
    }
  }
}

const flattenWeeks: (
  weeks: readonly GraphQLCalendarWeek[]
) => ContributionPoint[] = (
  weeks: readonly GraphQLCalendarWeek[]
): ContributionPoint[] => {
  const out: ContributionPoint[] = []
  for (const week of weeks) {
    for (const day of week.contributionDays) {
      out.push({
        count: day.contributionCount,
        date: day.date,
        level: levelToInt(day.contributionLevel),
      })
    }
  }
  return out
}

const getContributionDataUncached: () => Promise<
  ContributionPoint[]
> = async (): Promise<ContributionPoint[]> => {
  try {
    const toDate: string = new Date().toISOString()
    const fromDate: string = new Date(
      Date.now() - 365 * 24 * 60 * 60 * 1000
    ).toISOString()
    const query: string = buildContributionQuery()
    const headers: Record<string, string> = buildHeaders()

    const response: Response = await fetch('https://api.github.com/graphql', {
      body: JSON.stringify({
        query,
        variables: {
          from: fromDate,
          to: toDate,
          username: siteConfig.githubUsername,
        },
      }),
      headers,
      method: 'POST',
    })

    if (!response.ok) {
      // avoid template-literal on number (restrict-template-expressions)
      throw new Error('GitHub API error: ' + String(response.status))
    }

    const json: unknown = await response.json()

    // Validate & extract weeks
    if (!isGraphQLResponse(json)) {
      return []
    }
    const hasErrors: boolean =
      Array.isArray(json.errors) && json.errors.length > 0
    if (hasErrors) {
      // silently drop per no-console policy
      return []
    }

    const weeks: readonly GraphQLCalendarWeek[] =
      json.data.user?.contributionsCollection.contributionCalendar.weeks ?? []

    return flattenWeeks(weeks)
  } catch {
    return []
  }
}

export const getContributionData: () => Promise<ContributionPoint[]> =
  unstable_cache(getContributionDataUncached, ['contribution-data'], {
    revalidate: 3600,
  })

/* -------------------------------- aggregate -------------------------------- */

export interface GitHubData {
  contributionData: ContributionPoint[]
  projects: GitHubProject[]
  stats: UserStats
}

export const fetchGitHubData: () => Promise<
  Readonly<GitHubData>
> = async (): Promise<GitHubData> => {
  const projects: GitHubProject[] = await getFeaturedProjects()
  const stats: UserStats = await getUserStats()
  const contributionData: ContributionPoint[] = await getContributionData()
  return { contributionData, projects, stats }
}
