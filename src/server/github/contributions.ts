import { siteConfig } from '@/config/site'
import type { ContributionLevel, ContributionPoint } from '@/types/github'

type ContributionLevelResponse =
  | 'FIRST_QUARTILE'
  | 'FOURTH_QUARTILE'
  | 'NONE'
  | 'SECOND_QUARTILE'
  | 'THIRD_QUARTILE'

interface GraphQLCalendarDay {
  readonly contributionCount: number
  readonly contributionLevel: ContributionLevelResponse
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

const CONTRIBUTION_QUERY: string = `
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

const GITHUB_GRAPHQL_API: string = 'https://api.github.com/graphql'

const buildHeaders: () => Record<string, string> = (): Record<string, string> => {
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

const levelToInt: (level: ContributionLevelResponse) => ContributionLevel = (
  level: ContributionLevelResponse
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
  const contributions: ContributionPoint[] = []

  for (const week of weeks) {
    for (const day of week.contributionDays) {
      contributions.push({
        count: day.contributionCount,
        date: day.date,
        level: levelToInt(day.contributionLevel),
      })
    }
  }

  return contributions
}

export const fetchContributionData: () => Promise<ContributionPoint[]> = async (): Promise<ContributionPoint[]> => {
  try {
    const toDate: string = new Date().toISOString()
    const fromDate: string = new Date(
      Date.now() - 365 * 24 * 60 * 60 * 1000
    ).toISOString()

    const response: Response = await fetch(GITHUB_GRAPHQL_API, {
      body: JSON.stringify({
        query: CONTRIBUTION_QUERY,
        variables: {
          from: fromDate,
          to: toDate,
          username: siteConfig.githubUsername,
        },
      }),
      headers: buildHeaders(),
      method: 'POST',
    })

    if (!response.ok) {
      throw new Error('GitHub API error: ' + String(response.status))
    }

    const json: unknown = await response.json()
    if (!isGraphQLResponse(json)) {
      return []
    }

    const hasErrors: boolean =
      Array.isArray(json.errors) && json.errors.length > 0

    if (hasErrors) {
      return []
    }

    const weeks: readonly GraphQLCalendarWeek[] =
      json.data.user?.contributionsCollection.contributionCalendar.weeks ?? []

    return flattenWeeks(weeks)
  } catch {
    return []
  }
}
