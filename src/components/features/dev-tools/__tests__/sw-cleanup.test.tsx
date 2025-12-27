import { render, waitFor } from '@testing-library/react'
import { vi, describe, it, expect, beforeEach, afterEach } from 'vitest'
import { DevelopmentServiceWorkerGuard } from '../sw-cleanup'

// Mock environment at top level
vi.mock('@/environment', () => ({
  environment: { NODE_ENV: 'development' },
}))

describe('sw-cleanup', () => {
  const mockUnregister = vi.fn()
  const mockDelete = vi.fn()
  const mockReload = vi.fn()

  // Save original location
  const originalLocation = window.location

  beforeEach(() => {
    vi.clearAllMocks()

    // Mock Navigator
    const mockServiceWorker: any = {
      getRegistrations: vi
        .fn()
        .mockResolvedValue([{ unregister: mockUnregister }]),
    }

    Object.defineProperty(window, 'navigator', {
      value: { ...window.navigator, serviceWorker: mockServiceWorker },
      writable: true,
      configurable: true,
    })

    // Mock Caches
    Object.defineProperty(global, 'caches', {
      value: {
        keys: vi.fn().mockResolvedValue(['cache-1']),
        delete: mockDelete,
      },
      writable: true,
      configurable: true,
    })

    // Mock Location
    Object.defineProperty(window, 'location', {
      value: { ...originalLocation, reload: mockReload },
      writable: true,
      configurable: true,
    })
  })

  afterEach(() => {
    vi.restoreAllMocks()
    Object.defineProperty(window, 'location', {
      value: originalLocation,
      writable: true,
    })
  })

  it('should cleanup service workers in development', async () => {
    render(<DevelopmentServiceWorkerGuard />)

    // Wait for effect to run
    await new Promise((resolve) => setTimeout(resolve, 100))

    await waitFor(
      () => {
        expect(mockUnregister).toHaveBeenCalled()
      },
      { timeout: 2000 }
    )

    expect(mockDelete).toHaveBeenCalledWith('cache-1')
    expect(mockReload).toHaveBeenCalled()
  })

  it('should not cleanup if no registrations', async () => {
    // @ts-ignore
    navigator.serviceWorker.getRegistrations.mockResolvedValue([])

    render(<DevelopmentServiceWorkerGuard />)

    // Wait a tick
    await new Promise((resolve) => setTimeout(resolve, 0))

    expect(mockUnregister).not.toHaveBeenCalled()
    expect(mockReload).not.toHaveBeenCalled()
  })

  // Additional tests for production env or missing SW support can be added
})
