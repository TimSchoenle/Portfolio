import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import { ContributionGraph } from '../contribution-graph'
import type { ContributionPoint } from '@/types/github'

// Mock next-intl
vi.mock('next-intl/server', () => ({
  getTranslations: vi.fn(() => (key: string) => key),
}))

describe('ContributionGraph', () => {
  it('renders correctly with data', async () => {
    const data: ContributionPoint[] = [
      { date: '2023-01-01', count: 5, level: 1 },
      { date: '2023-01-02', count: 10, level: 2 },
    ]

    const Component = await ContributionGraph({ data, locale: 'en-US' })
    render(Component)

    // Check for title (mocked translation returns key)
    expect(screen.getByText('title')).toBeDefined()

    // Check for total count (mocked translation returns key, but we can check if it was called)
    // Since the mock returns the key, we might see 'totalAmount'
    expect(screen.getByText('totalAmount')).toBeDefined()
  })

  it('renders empty state correctly', async () => {
    const Component = await ContributionGraph({ data: [], locale: 'en-US' })
    render(Component)
    expect(screen.getByText('title')).toBeDefined()
  })
})
