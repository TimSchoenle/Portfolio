import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import ErrorPage from '../error'

// Mock next-intl
vi.mock('next-intl', () => ({
  useTranslations: () => (key: string) => key,
}))

// Mock lucide icons
vi.mock('lucide-react', () => ({
  AlertTriangle: () => <div data-testid="alert-icon">Alert</div>,
}))

describe('ErrorPage', () => {
  const mockReset = vi.fn()
  const mockError = {
    message: 'Test error',
    name: 'Error',
    digest: 'abc123',
  }

  beforeEach(() => {
    vi.clearAllMocks()
    delete (window as any).location
    ;(window as any).location = { assign: vi.fn() }
  })

  it('renders error title', () => {
    render(<ErrorPage error={mockError} reset={mockReset} />)
    expect(screen.getByText('title')).toBeDefined()
  })

  it('renders error description', () => {
    render(<ErrorPage error={mockError} reset={mockReset} />)
    expect(screen.getByText('description')).toBeDefined()
  })

  it('renders error digest when provided', () => {
    render(<ErrorPage error={mockError} reset={mockReset} />)
    expect(screen.getByText('abc123')).toBeDefined()
  })

  it('does not render digest when not provided', () => {
    const errorWithoutDigest = { ...mockError, digest: undefined }
    render(<ErrorPage error={errorWithoutDigest as any} reset={mockReset} />)

    expect(screen.queryByText('abc123')).toBeNull()
  })

  it('calls reset when Try Again is clicked', () => {
    render(<ErrorPage error={mockError} reset={mockReset} />)

    const tryAgainButton = screen.getByText('tryAgain')
    fireEvent.click(tryAgainButton)

    expect(mockReset).toHaveBeenCalledTimes(1)
  })

  it('navigates home when Go Home is clicked', () => {
    render(<ErrorPage error={mockError} reset={mockReset} />)

    const goHomeButton = screen.getByText('goHome')
    fireEvent.click(goHomeButton)

    expect(window.location.assign).toHaveBeenCalledWith('/')
  })

  it('renders alert icon', () => {
    render(<ErrorPage error={mockError} reset={mockReset} />)
    expect(screen.getByTestId('alert-icon')).toBeDefined()
  })
})
