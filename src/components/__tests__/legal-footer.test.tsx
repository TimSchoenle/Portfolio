import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import { LegalFooter } from '../legal-footer'

// Mock next-intl
vi.mock('next-intl/server', () => ({
  getTranslations: vi.fn(async () => (key: string) => key),
}))

// Mock routing Link
vi.mock('@/i18n/routing', () => ({
  Link: ({ children, href, ...props }: any) => (
    <a href={href} {...props}>
      {children}
    </a>
  ),
}))

describe('LegalFooter', () => {
  it('renders footer', async () => {
    const Component = await LegalFooter({ locale: 'en' })
    render(Component)

    const footer = screen.getByRole('contentinfo')
    expect(footer).toBeDefined()
  })

  it('renders imprint link', async () => {
    const Component = await LegalFooter({ locale: 'en' })
    render(Component)

    expect(screen.getByText('imprint.title')).toBeDefined()
  })

  it('renders privacy link', async () => {
    const Component = await LegalFooter({ locale: 'en' })
    render(Component)

    expect(screen.getByText('privacy.title')).toBeDefined()
  })
})
