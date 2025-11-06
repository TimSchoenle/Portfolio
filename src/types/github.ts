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

export interface ContributionPoint {
  readonly date: string
  readonly count: number
  readonly level: 0 | 1 | 2 | 3 | 4
}
