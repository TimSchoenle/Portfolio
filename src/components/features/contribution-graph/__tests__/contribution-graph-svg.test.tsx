import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { ContributionGraphSvg } from '../contribution-graph-svg'
import type { WeekModel } from '@/lib/github/contribution-calendar'
import type { ContributionPoint } from '@/models/github'

// Mock next-intl hooks
const mockTranslations = vi.fn((key: string, values?: any) => {
  if (key === 'tooltip') return `${values?.count} contributions`
  return key
})

const mockFormatter = {
  dateTime: vi.fn(() => 'Oct 25, 2023'),
}

vi.mock('next-intl', () => ({
  useTranslations: () => mockTranslations,
  useFormatter: () => mockFormatter,
}))

// Mock createPortal to render directly in body
vi.mock('react-dom', () => ({
  createPortal: (node: React.ReactNode) => node,
}))

describe('ContributionGraphSvg', () => {
  const mockWeeks: WeekModel[] = [
    {
      days: [
        { date: '2023-10-25', count: 5, level: 1 } as ContributionPoint,
        null,
        null,
        null,
        null,
        null,
        null,
      ],
      key: 'week1',
    },
  ]

  const defaultProps = {
    weeks: mockWeeks,
    dayOne: 'Mon',
    dayThree: 'Wed',
    dayFive: 'Fri',
    locale: 'en' as any,
  }

  it('renders the SVG graph', () => {
    const { container } = render(<ContributionGraphSvg {...defaultProps} />)
    const svg = container.querySelector('svg')
    expect(svg).toBeDefined()
    const rectangles = container.querySelectorAll('rect')
    expect(rectangles.length).toBe(1)
  })

  it('displays tooltip on hover', () => {
    const { container } = render(<ContributionGraphSvg {...defaultProps} />)
    const rect = container.querySelector('rect')

    if (!rect) throw new Error('Rect not found')

    // Simulate hover
    fireEvent.mouseEnter(rect)

    // Check if tooltip content appears
    // Since we mocked createPortal to render in place, it might be in the document body or near the component
    // But for testing-library `screen`, it should be visible
    expect(screen.getByText('5 contributions')).toBeDefined()
    expect(screen.getByText('Oct 25, 2023')).toBeDefined()
  })

  it('hides tooltip on mouse leave', () => {
    const { container } = render(<ContributionGraphSvg {...defaultProps} />)
    const rect = container.querySelector('rect')
    const svg = container.querySelector('svg')

    if (!rect || !svg) throw new Error('Elements not found')

    fireEvent.mouseEnter(rect)
    expect(screen.getByText('5 contributions')).toBeDefined()

    fireEvent.mouseLeave(svg)
    expect(screen.queryByText('5 contributions')).toBeNull()
  })
})
