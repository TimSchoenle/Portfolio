import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import { ExperienceSection } from '../experience-section'

// Mock next-intl
vi.mock('next-intl/server', () => ({
  getTranslations: vi.fn(async () => {
    return Object.assign((key: string) => key, {
      raw: (key: string) => {
        if (key === 'items') {
          return [
            {
              company: 'Test Company',
              title: 'Senior Developer',
              dateRange: '2020 - Present',
              description: 'Working on cool stuff',
              logo: null,
            },
          ]
        }
        return key
      },
    })
  }),
}))

// Mock next/image
vi.mock('next/image', () => ({
  default: ({ alt, ...props }: any) => <img alt={alt} {...props} />,
}))

// Mock Lucide icons
vi.mock('lucide-react', () => ({
  Briefcase: () => <div data-testid="briefcase-icon">Briefcase</div>,
  Calendar: () => <div data-testid="calendar-icon">Calendar</div>,
}))

describe('ExperienceSection', () => {
  it('renders section with title', async () => {
    const Component = await ExperienceSection({ locale: 'en' })
    render(Component)

    expect(screen.getByText('title')).toBeDefined()
  })

  it('renders experience items', async () => {
    const Component = await ExperienceSection({ locale: 'en' })
    render(Component)

    expect(screen.getByText('Test Company')).toBeDefined()
    expect(screen.getByText('Senior Developer')).toBeDefined()
    expect(screen.getByText('2020 - Present')).toBeDefined()
    expect(screen.getByText('Working on cool stuff')).toBeDefined()
  })

  it('renders briefcase icon when no logo', async () => {
    const Component = await ExperienceSection({ locale: 'en' })
    render(Component)

    expect(screen.getByTestId('briefcase-icon')).toBeDefined()
  })
})
