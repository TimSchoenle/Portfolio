import { describe, it, expect, vi } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import Page from '../page'

// Mock next/navigation
vi.mock('next/navigation', () => ({
  useRouter: vi.fn(),
  usePathname: vi.fn(),
  useSearchParams: vi.fn(),
  redirect: vi.fn(),
  notFound: vi.fn(),
}))

// Mock i18n routing
vi.mock('@/i18n/routing', () => ({
  Link: ({ children, ...props }: any) => <a {...props}>{children}</a>,
  getPathname: vi.fn(),
  routing: {
    locales: ['en', 'de'],
    defaultLocale: 'en',
  },
}))

// Mock next-intl
vi.mock('next-intl/server', () => ({
  setRequestLocale: vi.fn(),
  getTranslations: vi.fn(() => (key: string) => key),
}))

// Mock github data fetching
vi.mock('@/lib/github', () => ({
  fetchGitHubData: vi.fn().mockResolvedValue({
    contributionData: [],
    projects: [],
    stats: { stars: 0, forks: 0, repositories: 0 },
  }),
}))

// Mock config
vi.mock('@/lib/config', () => ({
  siteConfig: {
    email: 'test@example.com',
    github: 'https://github.com/test',
    githubUsername: 'testuser',
  },
}))

// Mock components to avoid deep rendering complexity in integration test
vi.mock('@/components/hero-section', () => ({
  HeroSection: () => <div data-testid="hero-section">Hero</div>,
}))

vi.mock('@/components/scroll-snap-pair-controller', () => ({
  ScrollSnapPairController: () => null,
}))
vi.mock('@/components/deferred-sections', () => ({
  DeferredSections: () => (
    <div data-testid="deferred-sections">Deferred Sections</div>
  ),
}))

describe('Page', () => {
  it('renders all sections', async () => {
    const Component = await Page({
      params: Promise.resolve({ locale: 'en-US' }),
    })
    render(Component)

    expect(screen.getByTestId('hero-section')).toBeDefined()

    // The other sections are deferred/suspended, so we might need to wait or check if they are rendered
    // In the actual code, they are inside <Suspense> and <DeferredSections>.
    // Since we are rendering the async component directly, React's `use` hook inside `DeferredSections` will resolve.
    // However, `render` from RTL might not handle async components fully without a wrapper or `await`.
    // But `Page` itself is async.

    // We can check if the main container is there
    expect(screen.getByRole('main')).toBeDefined()

    // Verify mocks were called
    const { fetchGitHubData } = await import('@/lib/github')
    expect(fetchGitHubData).toHaveBeenCalled()

    // Wait for deferred sections
    await waitFor(() => {
      expect(screen.getByTestId('deferred-sections')).toBeDefined()
    })
  })
})
