import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import AboutSection from '../about-section'

// Mock next-intl
vi.mock('next-intl/server', () => ({
  getTranslations: vi.fn(async () => {
    return Object.assign((key: string) => key, {
      rich: (key: string, options?: any) => {
        if (options?.highlight) {
          return options.highlight(`highlighted-${key}`)
        }
        return key
      },
    })
  }),
}))

// Mock Lucide icons
vi.mock('lucide-react', () => ({
  BookOpen: () => <div data-testid="book-open-icon">BookOpen</div>,
  Code2: () => <div data-testid="code2-icon">Code2</div>,
}))

describe('AboutSection', () => {
  it('renders section with title', async () => {
    const Component = await AboutSection({ locale: 'en' })
    render(Component)

    expect(screen.getByText('title')).toBeDefined()
  })

  it('renders learning card', async () => {
    const Component = await AboutSection({ locale: 'en' })
    render(Component)

    expect(screen.getByText('learning.title')).toBeDefined()
    expect(screen.getByTestId('book-open-icon')).toBeDefined()
  })

  it('renders expertise card', async () => {
    const Component = await AboutSection({ locale: 'en' })
    render(Component)

    expect(screen.getByText('expertise.title')).toBeDefined()
    expect(screen.getByTestId('code2-icon')).toBeDefined()
  })
})
