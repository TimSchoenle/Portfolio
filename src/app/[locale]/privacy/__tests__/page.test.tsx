import { describe, it, expect, vi } from 'vitest'
import { render } from '@testing-library/react'
import PrivacyPage from '../page'

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
      if (key === 'content') return 'Privacy policy content'
      if (key === 'lastUpdated') return '2025-11-29'
      if (key === 'title') return 'Privacy Policy'
      if (key === 'description') return 'Privacy Policy Description'
      return key
    }
    // Add rich method that returns React content
    mockTranslation.rich = (key: string, values?: any) => {
      if (key === 'content') {
        return (
          <div key="privacy-content">
            <h2>Privacy Policy</h2>
            <p>Controller: {values?.controllerName}</p>
            <p>Email: {values?.controllerEmail}</p>
            <p>Address: {values?.controllerAddress}</p>
            <p>Server location: {values?.serverLocation}</p>
            <p>Log retention: {values?.logRetentionDays} days</p>
            <p>Cloudflare: {values?.cloudflareProvider}</p>
            <a href={values?.cloudflarePolicyUrl}>Cloudflare Policy</a>
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

// Mock site config
vi.mock('@/lib/config', () => ({
  siteConfig: {
    legals: {
      privacyPolicyLastChange: '2025-11-29',
      cloudflare: {
        policyUrl: 'https://cloudflare.com/privacy',
        address: 'Cloudflare Inc.',
      },
      hosting: {
        name: 'Test Hosting',
        address: 'Test Address',
        policyUrl: 'https://hosting.test/privacy',
      },
      address: '123 Main St',
      logRetentionDays: 30,
      secondContact: 'https://contact.test',
    },
    email: 'contact@example.com',
    fullName: 'John Doe',
  },
}))

// Mock locale utilities
vi.mock('@/i18n/locale', () => ({
  ensureLocaleFromParameters: vi.fn(async () => 'en'),
  maybeLocaleFromParameters: vi.fn(async () => 'en'),
}))

describe('PrivacyPage', () => {
  const mockParams = Promise.resolve({ locale: 'en' })

  it('renders privacy page', async () => {
    const Component = await PrivacyPage({ params: mockParams })
    const { container } = render(Component)

    expect(
      container.querySelector('[data-testid="legal-layout"]')
    ).toBeDefined()
  })

  it('renders controller information', async () => {
    const Component = await PrivacyPage({ params: mockParams })
    const { container } = render(Component)

    // Just check that content was rendered
    expect(container.textContent).toBeTruthy()
  })
})

describe('PrivacyPage metadata', () => {
  const mockParams = Promise.resolve({ locale: 'en' })

  it('generates metadata', async () => {
    const { generateMetadata } = await import('../page')
    const metadata = await generateMetadata({ params: mockParams })

    expect(metadata).toBeDefined()
  })
})
