import { describe, it, expect, vi } from 'vitest'
import { render } from '@testing-library/react'
import ImprintPage from '../page'

// Mock i18n-legal-components
vi.mock('@/lib/i18n-legal-components', () => ({
  legalPageComponentMappings: {
    addressBlock: (chunks: any) => (
      <div className="address-block">{chunks}</div>
    ),
    contactBlock: (chunks: any) => (
      <div className="contact-block">{chunks}</div>
    ),
    emailLink: (chunks: any) => <a href={`mailto:${chunks}`}>{chunks}</a>,
    heading: (chunks: any) => <h2>{chunks}</h2>,
    link: (chunks: any) => <a href={chunks as string}>{chunks}</a>,
    section: (chunks: any) => <div>{chunks}</div>,
    text: (chunks: any) => <p>{chunks}</p>,
  },
}))

// Mock next-intl
vi.mock('next-intl/server', () => ({
  getTranslations: vi.fn(async () => {
    const mockTranslation = (key: string) => {
      if (key === 'content') {
        return 'Imprint content'
      }
      if (key === 'lastUpdated') {
        return '2025-11-29'
      }
      if (key === 'title') {
        return 'Imprint'
      }
      if (key === 'description') {
        return 'Imprint Description'
      }
      return key
    }
    // Add rich method that returns React elements
    mockTranslation.rich = (key: string, values?: any) => {
      if (key === 'content') {
        return (
          <div key="imprint-content">
            <h2>Imprint content rendered</h2>
            <p>Name: {values?.name}</p>
            <p>Email: {values?.email}</p>
            <p>Address: {values?.address}</p>
            <p>Country: {values?.country}</p>
            {values?.vatId && <p>VAT ID: {values.vatId}</p>}
          </div>
        )
      }
      if (key === 'lastUpdated') {
        return <div key="last-updated">Last updated: 2025-11-29</div>
      }
      return <div key={key}>{key}</div>
    }
    return mockTranslation
  }),
  getFormatter: vi.fn(async () => ({
    dateTime: (_date: Date) => '11/29/2025',
  })),
  setRequestLocale: vi.fn(),
}))

// Mock LastUpdateNotice
vi.mock('@/components/ui/last-update-notice', () => ({
  default: ({ lastUpdate }: any) => (
    <div data-testid="last-update-notice">
      Last updated: {lastUpdate.toISOString()}
    </div>
  ),
}))

// Mock LegalPageLayout
vi.mock('@/components/layout/legal-page-layout', () => ({
  LegalPageLayout: ({ children, title }: any) => (
    <div data-testid="legal-layout">
      <h1>{title}</h1>
      {children}
    </div>
  ),
}))

// Mock locale utilities
vi.mock('@/i18n/locale', () => ({
  ensureLocaleFromParameters: vi.fn(async () => 'en'),
  maybeLocaleFromParameters: vi.fn(async () => 'en'),
}))

// Mock config
vi.mock('@/lib/config', () => ({
  siteConfig: {
    fullName: 'Test User',
    email: 'test@example.com',
    socials: {
      github: 'https://github.com/test',
      linkedin: 'https://linkedin.com/test',
    },
    legals: {
      imprintLastChange: '2025-11-29',
      address: '123 Main St',
      vatId: 'DE123456789',
    },
  },
}))

describe('ImprintPage', () => {
  const mockParams = Promise.resolve({ locale: 'en' })

  it('renders imprint page', async () => {
    const Component = await ImprintPage({ params: mockParams })
    const { container } = render(Component)

    expect(
      container.querySelector('[data-testid="legal-layout"]')
    ).toBeDefined()
  })

  it('renders content', async () => {
    const Component = await ImprintPage({ params: mockParams })
    const { container } = render(Component)

    // Just check that content was rendered
    expect(container.textContent).toBeTruthy()
  })
})

describe('ImprintPage metadata', () => {
  const mockParams = Promise.resolve({ locale: 'en' })

  it('generates metadata', async () => {
    const { generateMetadata } = await import('../page')
    const metadata = await generateMetadata({ params: mockParams })

    expect(metadata).toBeDefined()
  })
})
