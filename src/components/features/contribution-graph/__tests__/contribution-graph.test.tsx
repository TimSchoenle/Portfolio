import { render, screen, fireEvent } from '@testing-library/react'
import { vi, describe, it, expect } from 'vitest'
import { ContributionGraphView } from '../contribution-graph-view'
import {
  HeaderSection,
  GraphSection,
} from '../contribution-graph-view-components'

// Mock dependencies
vi.mock('next-intl', () => ({
  useTranslations: () => (key: string, values?: any) => {
    if (key === 'totalAmount') return `Total: ${values.count} in ${values.year}`
    if (key === 'less') return 'Less'
    if (key === 'more') return 'More'
    if (key === 'title') return 'Contributions'
    return key
  },
}))

vi.mock('../contribution-graph-svg', () => ({
  ContributionGraphSvg: () => <div data-testid="graph-svg" />,
}))

// Mock Card
vi.mock('@/components/ui/card', () => ({
  Card: ({ children, className }: any) => (
    <div data-testid="card" className={className}>
      {children}
    </div>
  ),
}))

describe('ContributionGraph', () => {
  const mockProps = {
    calendar: {
      weeks: [] as any,
      colorScale: [] as any,
      monthLabels: [] as any,
    },
    labels: {
      dayOne: 'Mon',
      dayThree: 'Wed',
      dayFive: 'Fri',
      legend: {},
      months: {},
      total: {},
      weekdays: {},
    } as any,
    locale: 'en' as const,
    onYearChange: vi.fn(),
    selectedYear: 2023,
    total: 500,
    years: [2023, 2022, 2021],
  }

  describe('ContributionGraphView', () => {
    it('should render default variant as Card', () => {
      render(<ContributionGraphView {...mockProps} />)
      expect(screen.getByTestId('card')).toBeInTheDocument()
      expect(screen.getByText('Contributions')).toBeInTheDocument()
      expect(screen.getByTestId('graph-svg')).toBeInTheDocument()
    })

    it('should render blueprint variant as div', () => {
      render(<ContributionGraphView {...mockProps} variant="blueprint" />)
      expect(screen.queryByTestId('card')).not.toBeInTheDocument()
      // Check class for blueprint
      // expect(screen.getByTestId('wrapper')).toHaveClass('bg-blueprint-bg')
    })
  })

  describe('HeaderSection', () => {
    it('should render title and total', () => {
      render(
        <HeaderSection
          {...mockProps}
          translate={
            ((k: string, v: any) =>
              k === 'totalAmount' ? `Total: ${v.count}` : k) as any
          }
        />
      )
      expect(screen.getByText('title')).toBeInTheDocument()
      expect(screen.getByText('Total: 500')).toBeInTheDocument()
    })

    it('should render year options', () => {
      render(
        <HeaderSection {...mockProps} translate={((k: any) => k) as any} />
      )
      const select = screen.getByRole('combobox')
      expect(select).toBeInTheDocument()
      expect(screen.getAllByRole('option')).toHaveLength(3)
    })

    it('should call onYearChange when changed', () => {
      render(
        <HeaderSection {...mockProps} translate={((k: any) => k) as any} />
      )
      const select = screen.getByRole('combobox')
      fireEvent.change(select, { target: { value: '2022' } })
      expect(mockProps.onYearChange).toHaveBeenCalledWith(2022)
    })

    it('should render legend', () => {
      render(
        <HeaderSection {...mockProps} translate={((k: any) => k) as any} />
      )
      expect(screen.getByText('less')).toBeInTheDocument()
      expect(screen.getByText('more')).toBeInTheDocument()
    })
  })

  describe('GraphSection', () => {
    it('should render svg component', () => {
      render(
        <GraphSection
          dayFive="Fri"
          dayOne="Mon"
          dayThree="Wed"
          locale="en"
          weeks={[]}
        />
      )
      expect(screen.getByTestId('graph-svg')).toBeInTheDocument()
    })
  })
})
