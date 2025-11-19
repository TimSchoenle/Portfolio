import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import { HeroSection } from '../hero-section'

// Mock next-intl
vi.mock('next-intl/server', () => ({
  getTranslations: vi.fn(() => (key: string) => key),
}))

// Mock config
vi.mock('@/lib/config', () => ({
  siteConfig: {
    email: 'test@example.com',
    github: 'https://github.com/test',
  },
}))

describe('HeroSection', () => {
  it('renders hero content', async () => {
    const Component = await HeroSection({ locale: 'en-US' })
    render(Component)

    // Check for translation keys
    expect(screen.getByText('greeting')).toBeDefined()
    expect(screen.getByText('name')).toBeDefined()
    expect(screen.getByText('title')).toBeDefined()

    // Check for links
    const githubLink = screen.getByRole('link', { name: /github/i })
    expect(githubLink.getAttribute('href')).toBe('https://github.com/test')

    const emailLink = screen.getByRole('link', { name: /contact/i })
    expect(emailLink.getAttribute('href')).toBe('mailto:test@example.com')
  })
})
