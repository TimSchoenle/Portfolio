import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import { ProjectsSection } from '../projects-section'

// Mock next-intl
vi.mock('next-intl/server', () => ({
  getTranslations: vi.fn(async () => (key: string) => key),
}))

// Mock ContributionGraph
vi.mock('../contribution-graph', () => ({
  ContributionGraph: () => <div data-testid="contribution-graph">Graph</div>,
}))

// Mock Lucide icons
vi.mock('lucide-react', () => ({
  Code2: () => <div data-testid="code-icon">Code</div>,
  Star: () => <div data-testid="star-icon">Star</div>,
  GitFork: () => <div data-testid="fork-icon">Fork</div>,
  Link: () => <div data-testid="link-icon">Link</div>,
  Globe: () => <div data-testid="globe-icon">Globe</div>,
  ExternalLink: () => <div data-testid="external-icon">External</div>,
}))

describe('ProjectsSection', () => {
  const mockProjects = [
    {
      name: 'Test Project',
      description: 'A test project',
      html_url: 'https://github.com/test/project',
      stargazers_count: 10,
      forks_count: 5,
      topics: ['react', 'typescript'],
      language: 'TypeScript',
      homepage: 'https://test.com',
    },
  ]

  const mockStats = {
    repositories: 20,
    stars: 100,
    forks: 50,
  }

  const mockContributions = [
    { date: '2024-01-01', count: 5, level: 2 as const },
  ]

  it('renders section title', async () => {
    const Component = await ProjectsSection({
      locale: 'en',
      projects: mockProjects,
      stats: mockStats,
      contributionData: mockContributions,
      githubUsername: 'testuser',
    })
    render(Component)

    expect(screen.getByText('title')).toBeDefined()
  })

  it('renders stats cards', async () => {
    const Component = await ProjectsSection({
      locale: 'en',
      projects: mockProjects,
      stats: mockStats,
      contributionData: mockContributions,
      githubUsername: 'testuser',
    })
    render(Component)

    expect(screen.getByText('20')).toBeDefined()
  })
})
