import { describe, expect, it, vi } from 'vitest'

import { ContactSection } from '@/components/resume/templates/modern/contact-section'
import { siteConfig } from '@/lib/config'
import type { ResumeTranslations } from '@/types/resume'

// Mock site config
vi.mock('@/lib/config', () => ({
  siteConfig: {
    email: 'test@example.com',
    github: 'https://github.com/testuser',
    linkedin: 'https://linkedin.com/in/testuser',
  },
}))

describe('ContactSection', () => {
  const mockTranslations: ResumeTranslations = vi.fn((key: string) => {
    const translations: Record<string, string> = {
      'contact.title': 'Contact',
      'contact.email': 'Email',
      'contact.location': 'Location',
      'personalInfo.country': 'Germany',
      'common.socials.github': 'GitHub',
      'common.socials.linkedin': 'LinkedIn',
    }
    return translations[key] ?? key
  }) as unknown as ResumeTranslations

  it('can be imported', async () => {
    const module = await import(
      '@/components/resume/templates/modern/contact-section'
    )
    expect(module.ContactSection).toBeDefined()
  })

  it('renders without errors', () => {
    expect(() => {
      ContactSection({ translations: mockTranslations })
    }).not.toThrow()
  })

  it('displays section title', () => {
    const result = ContactSection({
      translations: mockTranslations,
    })

    const resultString = JSON.stringify(result)
    expect(resultString.includes('Contact')).toBe(true)
  })

  it('displays email from siteConfig', () => {
    const result = ContactSection({
      translations: mockTranslations,
    })

    const resultString = JSON.stringify(result)
    expect(resultString.includes(siteConfig.email)).toBe(true)
  })

  it('displays location from translations', () => {
    const result = ContactSection({
      translations: mockTranslations,
    })

    const resultString = JSON.stringify(result)
    expect(resultString.includes('Germany')).toBe(true)
  })

  it('displays GitHub link', () => {
    const result = ContactSection({
      translations: mockTranslations,
    })

    const resultString = JSON.stringify(result)
    expect(resultString.includes(siteConfig.github ?? '')).toBe(true)
  })

  it('displays LinkedIn link when available', () => {
    const result = ContactSection({
      translations: mockTranslations,
    })

    const resultString = JSON.stringify(result)
    expect(resultString.includes(siteConfig.linkedin ?? '')).toBe(true)
  })

  it('renders section divider', () => {
    const result = ContactSection({
      translations: mockTranslations,
    })

    const resultString = JSON.stringify(result)
    // Check for style object which implies View components are present
    expect(resultString.includes('"style":')).toBe(true)
  })
})
