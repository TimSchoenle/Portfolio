import 'server-only'
import { Octokit } from '@octokit/rest'
import { unstable_cache } from 'next/cache'

import { environment } from '@/environment'
import { siteConfig } from '@/lib/config'
import { logger } from '@/lib/logger'
import {
  type ContributionCollection,
  type ContributionLevel,
  type ContributionPoint,
  contributionPointSchema,
  type GitHubProject,
  gitHubProjectSchema,
  type UserStats,
  userStatsSchema,
} from '@/models/github'

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
  auth: environment.GITHUB_TOKEN,
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
                owner: siteConfig.socials.githubUsername,
                repo,
              })

            return gitHubProjectSchema.parse({
              description: resp.data.description ?? '',
              forks_count: resp.data.forks_count,
              homepage: resp.data.homepage ?? undefined,
              html_url: resp.data.html_url,
              language: resp.data.language ?? 'Unknown',
              name: resp.data.name,
              stargazers_count: resp.data.stargazers_count,
              topics: Array.isArray(resp.data.topics) ? resp.data.topics : [],
            })
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
        username: siteConfig.socials.githubUsername,
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

      return userStatsSchema.parse({
        forks: totalForks,
        repositories: repos.length,
        stars: totalStars,
      })
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
  const token: string | undefined = environment.GITHUB_TOKEN
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
      out.push(
        contributionPointSchema.parse({
          count: day.contributionCount,
          date: day.date,
          level: levelToInt(day.contributionLevel),
        })
      )
    }
  }
  return out
}

const fetchYearlyData: (year: number) => Promise<ContributionPoint[]> = async (
  year: number
): Promise<ContributionPoint[]> => {
  const yearString: string = String(year)
  const fromDate: string = `${yearString}-01-01T00:00:00Z`
  const toDate: string = `${yearString}-12-31T23:59:59Z`

  try {
    const response: Response = await fetch('https://api.github.com/graphql', {
      body: JSON.stringify({
        query: buildContributionQuery(),
        variables: {
          from: fromDate,
          to: toDate,
          username: siteConfig.socials.githubUsername,
        },
      }),
      headers: buildHeaders(),
      method: 'POST',
    })

    if (!response.ok) {
      logger.error(
        { status: response.status, year: yearString },
        `GitHub API error`
      )
      return []
    }

    const json: unknown = await response.json()
    if (!isGraphQLResponse(json)) {
      return []
    }

    if (Array.isArray(json.errors) && json.errors.length > 0) {
      logger.error(
        { errors: json.errors, year: yearString },
        `GitHub GraphQL errors`
      )
      return []
    }

    return flattenWeeks(
      json.data.user?.contributionsCollection.contributionCalendar.weeks ?? []
    )
  } catch (error) {
    logger.error(
      { err: error, year: yearString },
      `Failed to fetch contribution data`
    )
    return []
  }
}

const getContributionDataUncached: () => Promise<ContributionCollection> =
  async (): Promise<ContributionCollection> => {
    const currentYear: number = new Date().getFullYear()
    const years: number[] = Array.from(
      { length: siteConfig.contribution.yearsToShow },
      (_unused: unknown, index: number): number => currentYear - index
    )

    const results: { data: ContributionPoint[]; year: number }[] =
      await Promise.all(
        years.map(
          async (
            year: number
          ): Promise<{
            data: ContributionPoint[]
            year: number
          }> => {
            const data: ContributionPoint[] = await fetchYearlyData(year)
            return { data, year }
          }
        )
      )

    const collection: ContributionCollection = {}

    for (const { data, year } of results) {
      if (data.length > 0) {
        Object.assign(collection, { [year]: data })
      }
    }

    return collection
  }

export const getContributionData: () => Promise<ContributionCollection> =
  unstable_cache(getContributionDataUncached, ['contribution-data'], {
    revalidate: 3600,
  })

/* -------------------------------- aggregate -------------------------------- */

export interface GitHubData {
  contributionData: ContributionCollection
  projects: GitHubProject[]
  stats: UserStats
}

export const getGithubUser: () => Promise<
  Readonly<GitHubData>
> = async (): Promise<GitHubData> => {
  const projects: GitHubProject[] = await getFeaturedProjects()
  const stats: UserStats = await getUserStats()
  const contributionData: ContributionCollection = await getContributionData()
  return { contributionData, projects, stats }
}
