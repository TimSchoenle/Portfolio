import { describe, it, expect, vi } from 'vitest'

// Mock next/navigation
vi.mock('next/navigation', () => ({
  notFound: vi.fn(),
  redirect: vi.fn(),
}))

// Mock next-intl/navigation
vi.mock('next-intl/navigation', () => ({
  createNavigation: vi.fn(() => ({
    Link: () => null,
    redirect: vi.fn(),
    usePathname: vi.fn(),
    useRouter: vi.fn(),
  })),
}))

// Mock next-intl/server
vi.mock('next-intl/server', () => ({
  getRequestConfig: (fn: any) => fn,
}))

// Mock routing
vi.mock('./routing', () => ({
  routing: {
    defaultLocale: 'en',
    locales: ['en', 'de'],
  },
}))

describe('request configuration', () => {
  it('loads messages for valid locale', async () => {
    const config = await import('../request')
    const getConfig = config.default as any

    const result = await getConfig({
      requestLocale: Promise.resolve('en'),
    })

    expect(result.locale).toBe('en')
    expect(result.messages).toHaveProperty('hero')
    expect(result.messages).toHaveProperty('about')
  })

  it('falls back to default locale for invalid locale', async () => {
    const config = await import('../request')
    const getConfig = config.default as any

    const result = await getConfig({
      requestLocale: Promise.resolve('fr'),
    })

    expect(result.locale).toBe('en')
    expect(result.messages).toHaveProperty('hero')
  })
})
