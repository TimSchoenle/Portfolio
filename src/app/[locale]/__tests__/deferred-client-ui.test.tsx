import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, act, screen } from '@testing-library/react'

// Mock lazy-client to return simple components immediately
vi.mock('@/lib/lazy-client', () => ({
  lazyClient: () => () => (
    <div data-testid="lazy-component">Lazy Component</div>
  ),
}))

describe('DeferredClientUi', () => {
  beforeEach(() => {
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
    vi.restoreAllMocks()
  })

  it('renders nothing initially', async () => {
    const { default: DeferredClientUi } = await import('../deferred-client-ui')
    const { container } = render(<DeferredClientUi />)
    expect(container.firstChild).toBeNull()
  })

  it('renders children after idle callback', async () => {
    // Mock requestIdleCallback
    const mockRequestIdleCallback = vi.fn((cb) => {
      cb()
      return 1
    })
    window.requestIdleCallback = mockRequestIdleCallback as any

    const { default: DeferredClientUi } = await import('../deferred-client-ui')

    await act(async () => {
      render(<DeferredClientUi />)
    })

    expect(screen.getAllByTestId('lazy-component').length).toBeGreaterThan(0)
  })

  it('falls back to setTimeout if requestIdleCallback is missing', async () => {
    // Remove requestIdleCallback
    // @ts-ignore
    delete window.requestIdleCallback

    const { default: DeferredClientUi } = await import('../deferred-client-ui')

    render(<DeferredClientUi />)

    // Should be null initially
    expect(screen.queryByTestId('lazy-component')).toBeNull()

    // Advance timers
    await act(async () => {
      vi.runAllTimers()
    })

    expect(screen.getAllByTestId('lazy-component').length).toBeGreaterThan(0)
  })
})
