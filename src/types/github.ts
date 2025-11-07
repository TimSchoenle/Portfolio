export interface GitHubProject {
  readonly name: string
  readonly description: string
  readonly html_url: string
  readonly homepage: string | undefined
  readonly stargazers_count: number
  readonly forks_count: number
  readonly language: string
  readonly topics: readonly string[]
}

export interface UserStats {
  readonly repositories: number
  readonly stars: number
  readonly forks: number
}

export const CONTRIBUTION_LEVELS: readonly [0, 1, 2, 3, 4] = [
  0, 1, 2, 3, 4,
] as const

export type ContributionLevel = (typeof CONTRIBUTION_LEVELS)[number]

export interface ContributionPoint {
  readonly date: string
  readonly count: number
  readonly level: ContributionLevel
}
