export interface GitHubProject {
  readonly description: string
  readonly forks_count: number
  readonly homepage: string | undefined
  readonly html_url: string
  readonly language: string
  readonly name: string
  readonly stargazers_count: number
  readonly topics: readonly string[]
}

export interface UserStats {
  readonly forks: number
  readonly repositories: number
  readonly stars: number
}

export const CONTRIBUTION_LEVELS: readonly [0, 1, 2, 3, 4] = [
  0, 1, 2, 3, 4,
] as const

export type ContributionLevel = (typeof CONTRIBUTION_LEVELS)[number]

export interface ContributionPoint {
  readonly count: number
  readonly date: string
  readonly level: ContributionLevel
}
