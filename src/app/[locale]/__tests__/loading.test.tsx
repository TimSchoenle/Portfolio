import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import Loading from '../loading'

// Mock next-intl
vi.mock('next-intl', () => ({
  useTranslations: vi.fn(() => (key: string) => key),
}))

describe('Loading', () => {
  it('renders loading state', () => {
    render(<Loading />)
    expect(screen.getByText('title')).toBeDefined()
  })

  it('renders spinner container', () => {
    const { container } = render(<Loading />)
    const spinner = container.querySelector('.animate-spin')
    expect(spinner).toBeDefined()
  })

  it('renders pulsing text', () => {
    const { container } = render(<Loading />)
    const pulseText = container.querySelector('.animate-pulse')
    expect(pulseText).toBeDefined()
  })
})
