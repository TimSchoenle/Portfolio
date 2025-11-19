import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import { BackToHome } from '../back-to-home'

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

// Mock lucide icons
vi.mock('lucide-react', () => ({
  ArrowLeft: () => <div data-testid="arrow-icon">Arrow</div>,
}))

describe('BackToHome', () => {
  it('renders back button', async () => {
    const Component = await BackToHome({ locale: 'en' })
    render(Component)

    expect(screen.getByText('backHome')).toBeDefined()
  })

  it('renders arrow icon', async () => {
    const Component = await BackToHome({ locale: 'en' })
    render(Component)

    expect(screen.getByTestId('arrow-icon')).toBeDefined()
  })

  it('links to home page', async () => {
    const Component = await BackToHome({ locale: 'en' })
    const { container } = render(Component)

    const link = container.querySelector('a[href="/"]')
    expect(link).toBeDefined()
  })
})
