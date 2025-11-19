import { describe, it, expect, vi } from 'vitest'
import sitemap from '../sitemap'

// Mock config
vi.mock('@/lib/config', () => ({
  siteConfig: {
    url: 'https://example.com/',
  },
}))

// Mock i18n routing
vi.mock('@/i18n/routing', () => ({
  routing: {
    locales: ['en', 'de'],
  },
  getPathname: ({ href, locale }: any) => `/${locale}${href}`,
}))

describe('sitemap.ts', () => {
  it('returns sitemap array', () => {
    const result = sitemap()

    expect(Array.isArray(result)).toBe(true)
    expect(result.length).toBeGreaterThan(0)
  })

  it('includes root page', () => {
    const result = sitemap()

    const rootPage = result.find(
      (entry) => entry.url === 'https://example.com/'
    )
    expect(rootPage).toBeDefined()
    expect(rootPage?.priority).toBe(1)
  })

  it('includes static pages', () => {
    const result = sitemap()

    expect(result.length).toBeGreaterThan(1)
  })

  it('includes alternates for locales', () => {
    const result = sitemap()

    const firstEntry = result[0]
    expect(firstEntry?.alternates?.languages).toBeDefined()
  })
})
