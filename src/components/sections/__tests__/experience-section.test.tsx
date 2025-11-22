import { describe, expect, it, vi } from 'vitest'

import { render, screen } from '@testing-library/react'

import { ExperienceSection } from '../experience-section'

// Mock next-intl
vi.mock('next-intl/server', () => ({
  getTranslations: vi.fn(async () => {
    const t = (key: string) => {
      if (key === 'sectionTitles.experience') return 'Experience'
      return key
    }
    t.raw = (key: string) => {
      if (key === 'experience') {
        return [
          {
            achievements: ['Working on cool stuff'],
            company: 'Test Company',
            endDate: 'Present',
            location: 'Remote',
            startDate: '2020',
            title: 'Senior Developer',
          },
        ]
      }
      return key
    }
    return t
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
  MapPin: () => <div data-testid="map-pin-icon">MapPin</div>,
}))

describe('Experience_section', () => {
  it('renders section with title', async () => {
    const Component = await ExperienceSection({ locale: 'en' })
    render(Component)

    expect(screen.getByText('Experience')).toBeDefined()
  })

  it('renders experience items', async () => {
    const Component = await ExperienceSection({ locale: 'en' })
    render(Component)

    expect(screen.getByText('Test Company')).toBeDefined()
    expect(screen.getByText('Senior Developer')).toBeDefined()
    expect(screen.getByText('2020 - Present')).toBeDefined()
    expect(screen.getByText('Working on cool stuff')).toBeDefined()
  })

  it('renders briefcase icon', async () => {
    const Component = await ExperienceSection({ locale: 'en' })
    render(Component)

    expect(screen.getByTestId('briefcase-icon')).toBeDefined()
  })
})
