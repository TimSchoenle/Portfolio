import { render, screen, act } from '@testing-library/react'
import { vi, describe, it, expect, beforeEach, afterEach } from 'vitest'
import { LazyLoad } from '../lazy-load'

describe('LazyLoad', () => {
  let observeMock = vi.fn()
  let disconnectMock = vi.fn()

  beforeEach(() => {
    observeMock = vi.fn()
    disconnectMock = vi.fn()

    // Mock IntersectionObserver
    global.IntersectionObserver = class IntersectionObserver {
      constructor(callback: IntersectionObserverCallback) {
        // @ts-ignore
        this.callback = callback
      }
      callback(_entries: any) {}
      observe = observeMock
      disconnect = disconnectMock
      unobserve = vi.fn()
      takeRecords = vi.fn()
      root = null
      rootMargin = ''
      thresholds = []
    } as any
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  it('should render nothing initially', () => {
    render(
      <LazyLoad>
        <div data-testid="child">Child</div>
      </LazyLoad>
    )
    expect(screen.queryByTestId('child')).not.toBeInTheDocument()
    expect(observeMock).toHaveBeenCalled()
  })

  it('should render children when intersecting', () => {
    let callback: any
    global.IntersectionObserver = class IntersectionObserver {
      constructor(cb: any) {
        callback = cb
      }
      observe = observeMock
      disconnect = disconnectMock
    } as any

    render(
      <LazyLoad>
        <div data-testid="child">Child</div>
      </LazyLoad>
    )

    act(() => {
      callback([{ isIntersecting: true }])
    })

    expect(screen.getByTestId('child')).toBeInTheDocument()
    expect(disconnectMock).toHaveBeenCalled()
  })

  it('should load immediately if IntersectionObserver is undefined', () => {
    const originalObserver = global.IntersectionObserver
    // @ts-ignore
    global.IntersectionObserver = undefined

    render(
      <LazyLoad>
        <div data-testid="child">Child</div>
      </LazyLoad>
    )
    expect(screen.getByTestId('child')).toBeInTheDocument()

    // Restore
    global.IntersectionObserver = originalObserver
  })
})
