import { describe, it, expect, vi } from 'vitest'
import robots from '../robots'

// Mock config
vi.mock('@/lib/config', () => ({
  siteConfig: {
    url: 'https://example.com',
  },
}))

describe('robots.ts', () => {
  it('returns robots configuration', () => {
    const result = robots()

    expect(result).toBeDefined()
    expect(result.host).toBe('https://example.com')
    expect(result.sitemap).toBe('https://example.com/sitemap.xml')
  })

  it('includes correct rules', () => {
    const result = robots()

    expect(result.rules).toHaveLength(1)
    expect((result.rules as any)[0].userAgent).toBe('*')
    expect((result.rules as any)[0].allow).toBe('/')
    expect((result.rules as any)[0].disallow).toContain('/api/')
  })
})
