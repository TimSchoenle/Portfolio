import { describe, expect, it, vi } from 'vitest'

import { render, screen } from '@testing-library/react'

import { HeroSection } from '../hero-section'

// Mock next-intl
vi.mock('next-intl/server', () => ({
  getTranslations: vi.fn(() => (key: string) => {
    const translations: Record<string, string> = {
      'common.socials.github': 'GitHub',
      contact: 'Contact Me',
      greeting: 'Hi, I am',
      location: 'Based in {country}',
      'personalInfo.country': 'Germany',
      'personalInfo.jobTitle': 'Software Developer',
      tagline: 'Passionate about coding',
    }
    return translations[key] ?? key
  }),
}))

// Mock config
vi.mock('@/lib/config', () => ({
  siteConfig: {
    email: 'test@example.com',
    fullName: 'John Doe',
    name: 'John',
    socials: {
      github: 'https://github.com/test',
    },
  },
}))

describe('HeroSection', () => {
  it('renders hero content', async () => {
    const Component = await HeroSection({ locale: 'en-US' })
    render(Component)

    // Check for actual rendered content
    expect(screen.getByText('Hi, I am')).toBeDefined()
    expect(screen.getByText('John')).toBeDefined()
    expect(screen.getByText('Software Developer')).toBeDefined()

    // Check for links
    const githubLink = screen.getByRole('link', { name: /github/i })
    expect(githubLink.getAttribute('href')).toBe('https://github.com/test')

    const emailLink = screen.getByRole('link', { name: /contact/i })
    expect(emailLink.getAttribute('href')).toBe('mailto:test@example.com')
  })
})
