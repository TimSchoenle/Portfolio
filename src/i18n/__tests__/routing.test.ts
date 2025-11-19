import { describe, it, expect, vi } from 'vitest'

// Mock next/navigation before importing routing
vi.mock('next/navigation', () => ({
  notFound: vi.fn(),
  redirect: vi.fn(),
  useRouter: vi.fn(),
  usePathname: vi.fn(),
}))

// Mock next-intl functions
vi.mock('next-intl/navigation', () => ({
  createNavigation: () => ({
    Link: 'div',
    redirect: vi.fn(),
    useRouter: vi.fn(),
    usePathname: vi.fn(),
    getPathname: vi.fn(),
  }),
}))

vi.mock('next-intl/routing', () => ({
  defineRouting: (config: any) => config,
}))

describe('routing configuration', () => {
  it('can be imported', async () => {
    const { routing } = await import('../routing')

    expect(routing).toBeDefined()
  })

  it('has correct default locale', async () => {
    const { routing } = await import('../routing')

    expect(routing.defaultLocale).toBe('en')
  })

  it('has correct supported locales', async () => {
    const { routing } = await import('../routing')

    expect(routing.locales).toEqual(['en', 'de'])
  })
})
