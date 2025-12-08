import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import { ProjectsSection } from '../projects-section'
import type { GitHubProject } from '@/models/github'

// Mock next-intl
vi.mock('next-intl/server', () => ({
  getTranslations: vi.fn(async () => {
    const t = (key: string) => key
    t.rich = (key: string) => key
    t.markup = (key: string) => key
    t.raw = (key: string) => key
    t.has = () => true
    return t
  }),
}))

// Mock GitHub client
const mockGetGithubUser = vi.fn().mockResolvedValue({
  contributionData: [],
  projects: [],
  stats: { repositories: 0, stars: 0, forks: 0 },
})

vi.mock('@/lib/github/client', () => ({
  getGithubUser: () => mockGetGithubUser(),
}))

// Mock ContributionGraph
vi.mock('@/components/features/contribution-graph/contribution-graph', () => ({
  ContributionGraph: () => (
    <div data-testid="contribution-graph">Contribution Graph</div>
  ),
}))

// Mock GridPattern and RadialGradient to avoid rendering issues
vi.mock('@/components/ui/grid-pattern', () => ({
  GridPattern: () => <div data-testid="grid-pattern" />,
}))
vi.mock('@/components/ui/radial-gradient', () => ({
  RadialGradient: () => <div data-testid="radial-gradient" />,
}))

describe('ProjectsSection', () => {
  it('renders section and projects when data is present', async () => {
    const projects: GitHubProject[] = [
      {
        name: 'Project 1',
        description: 'Description 1',
        html_url: 'http://example.com',
        homepage: 'http://example.com',
        topics: ['react'],
        stargazers_count: 10,
        forks_count: 5,
        language: 'TypeScript',
      },
    ]

    mockGetGithubUser.mockResolvedValueOnce({
      contributionData: [{ date: '2023-01-01', count: 1, level: 1 }],
      projects: projects,
      stats: { repositories: 1, stars: 10, forks: 5 },
    })

    const Component = await ProjectsSection({ locale: 'en' })
    const { container } = render(Component)

    // Assert section renders
    expect(container.querySelector('#projects')).toBeDefined()
    expect(screen.getAllByText('title').length).toBeGreaterThan(0)
    expect(screen.getAllByText('Project 1').length).toBeGreaterThan(0)
    expect(screen.getByTestId('contribution-graph')).toBeDefined()
  })

  it('hides contribution graph when contribution data is empty', async () => {
    const projects: GitHubProject[] = [
      {
        name: 'Project 1',
        description: 'Description 1',
        html_url: 'http://example.com',
        homepage: 'http://example.com',
        topics: ['react'],
        stargazers_count: 10,
        forks_count: 5,
        language: 'TypeScript',
      },
    ]

    mockGetGithubUser.mockResolvedValueOnce({
      contributionData: [],
      projects: projects,
      stats: { repositories: 1, stars: 10, forks: 5 },
    })

    const Component = await ProjectsSection({ locale: 'en' })
    render(Component)

    expect(screen.getAllByText('Project 1').length).toBeGreaterThan(0)
    expect(screen.queryByTestId('contribution-graph')).toBeNull()
  })

  it('returns empty section when projects are empty', async () => {
    mockGetGithubUser.mockResolvedValueOnce({
      contributionData: [],
      projects: [],
      stats: { repositories: 0, stars: 0, forks: 0 },
    })

    const Component = await ProjectsSection({ locale: 'en' })
    const { container } = render(Component)

    // Should return just <section id="projects" /> which is empty
    const section = container.querySelector('#projects')
    expect(section).toBeDefined()
    expect(section?.children.length).toBe(0)
  })
})
