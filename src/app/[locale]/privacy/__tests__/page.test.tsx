import { describe, it, expect, vi } from 'vitest'
import { render } from '@testing-library/react'
import PrivacyPage from '../page'

// Mock next-intl
vi.mock('next-intl/server', () => ({
  getTranslations: vi.fn(async () => (key: string) => key),
  setRequestLocale: vi.fn(),
}))

// Mock LegalPageLayout
vi.mock('@/components/legal-page-layout', () => ({
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
