import { describe, it, expect, vi } from 'vitest'
import { render } from '@testing-library/react'
import RootLayout from '../layout'

// Mock all dependencies
vi.mock('next/font/google', () => ({
  Inter: () => ({ variable: '--font-inter' }),
}))

vi.mock('next-intl/server', () => ({
  getMessages: vi.fn(async () => ({ test: 'message' })),
  setRequestLocale: vi.fn(),
}))

vi.mock('next-intl', () => ({
  NextIntlClientProvider: ({ children }: any) => (
    <div data-testid="intl-provider">{children}</div>
  ),
}))

vi.mock('@/i18n/locale', () => ({
  ensureLocaleFromParameters: vi.fn(async () => 'en'),
  maybeLocaleFromParameters: vi.fn(async () => 'en'),
}))

vi.mock('@/i18n/routing', () => ({
  routing: {
    locales: ['en', 'de'],
    defaultLocale: 'en',
  },
  getPathname: vi.fn(() => '/'),
}))

vi.mock('@/lib/config', () => ({
  siteConfig: {
    title: 'Test Site',
    description: 'Test Description',
    url: 'https://test.com',
    fullName: 'Test User',
    email: 'test@test.com',
    socials: {
      githubUsername: 'testuser',
    },
    seo: { keywords: ['test', 'keywords'] },
  },
}))

vi.mock('@/components/common/theme-provider', () => ({
  ThemeProvider: ({ children }: any) => (
    <div data-testid="theme-provider">{children}</div>
  ),
}))

vi.mock('@/app/[locale]/deferred-client-ui', () => ({
  __esModule: true,
  default: () => <div data-testid="deferred-ui" />,
}))

vi.mock('@/components/layout/legal-footer', () => ({
  LegalFooter: () => <div data-testid="legal-footer" />,
}))

vi.mock('@/components/common/theme-toggle', () => ({
  ThemeToggle: () => <div data-testid="theme-toggle" />,
}))

vi.mock('@/components/common/language-switcher', () => ({
  LanguageSwitcher: () => <div data-testid="language-switcher" />,
}))

describe('RootLayout', () => {
  const mockParams = Promise.resolve({ locale: 'en' })

  it('renders layout with children', async () => {
    const Component = await RootLayout({
      children: <div>Test Content</div>,
      params: mockParams,
    })

    const { container } = render(Component)
    expect(container.textContent).toContain('Test Content')
  })

  it('renders all providers', async () => {
    const Component = await RootLayout({
      children: <div>Content</div>,
      params: mockParams,
    })

    const { getByTestId } = render(Component)
    expect(getByTestId('intl-provider')).toBeDefined()
    expect(getByTestId('theme-provider')).toBeDefined()
  })

  it('renders deferred client UI', async () => {
    const Component = await RootLayout({
      children: <div>Content</div>,
      params: mockParams,
    })

    const { getByTestId } = render(Component)
    expect(getByTestId('deferred-ui')).toBeDefined()
  })

  it('renders legal footer', async () => {
    const Component = await RootLayout({
      children: <div>Content</div>,
      params: mockParams,
    })

    const { getByTestId } = render(Component)
    expect(getByTestId('legal-footer')).toBeDefined()
    expect(getByTestId('theme-toggle')).toBeDefined()
    expect(getByTestId('language-switcher')).toBeDefined()
  })
})

describe('RootLayout metadata', () => {
  it('exports generateMetadata', async () => {
    const { generateMetadata } = await import('../layout')
    expect(generateMetadata).toBeDefined()
  })

  it('generates metadata for supported locale', async () => {
    const { generateMetadata } = await import('../layout')
    const metadata = await generateMetadata({
      params: Promise.resolve({ locale: 'en' }),
    })

    expect(metadata.title).toBeDefined()
    expect(metadata.description).toBe('Test Description')
  })
})

describe('RootLayout viewport', () => {
  it('exports viewport configuration', async () => {
    const { viewport } = await import('../layout')
    expect(viewport).toBeDefined()
    expect(viewport.width).toBe('device-width')
  })
})

describe('RootLayout generateStaticParams', () => {
  it('exports generateStaticParams', async () => {
    const { generateStaticParams } = await import('../layout')
    expect(generateStaticParams).toBeDefined()
  })

  it('generates params for all locales', async () => {
    const { generateStaticParams } = await import('../layout')
    const params = generateStaticParams()

    expect(params).toHaveLength(2)
    expect((params as any)[0].locale).toBe('en')
    expect((params as any)[1].locale).toBe('de')
  })
})
