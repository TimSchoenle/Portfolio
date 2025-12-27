import { renderHook } from '@testing-library/react'
import { vi, describe, it, expect, beforeEach } from 'vitest'
import { usePaletteNavigation } from '../use-palette-navigation'
import * as KeyHandlers from '../key-handlers'

vi.mock('../key-handlers', () => ({
  handlePaletteKey: vi.fn(),
}))

const mockRouter = {
  push: vi.fn(),
  replace: vi.fn(),
  back: vi.fn(),
  forward: vi.fn(),
  refresh: vi.fn(),
  prefetch: vi.fn(),
}

const mockRun = vi.fn()

describe('usePaletteNavigation', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('should registers event listener when open is true', () => {
    const addSpy = vi.spyOn(document, 'addEventListener')
    const removeSpy = vi.spyOn(document, 'removeEventListener')

    const { unmount } = renderHook(() =>
      usePaletteNavigation({
        locale: 'en',
        open: true,
        pathname: '/',
        router: mockRouter as any,
        run: mockRun,
      })
    )

    expect(addSpy).toHaveBeenCalledWith('keydown', expect.any(Function))

    unmount()
    expect(removeSpy).toHaveBeenCalledWith('keydown', expect.any(Function))
  })

  it('should not register event listener when open is false', () => {
    const addSpy = vi.spyOn(document, 'addEventListener')
    renderHook(() =>
      usePaletteNavigation({
        locale: 'en',
        open: false, // Closed
        pathname: '/',
        router: mockRouter as any,
        run: mockRun,
      })
    )
    expect(addSpy).not.toHaveBeenCalled()
  })

  it('should call handlePaletteKey on keydown', () => {
    renderHook(() =>
      usePaletteNavigation({
        locale: 'en',
        open: true,
        pathname: '/',
        router: mockRouter as any,
        run: mockRun,
      })
    )

    const event = new KeyboardEvent('keydown', { key: 'h' })
    document.dispatchEvent(event)

    expect(KeyHandlers.handlePaletteKey).toHaveBeenCalledWith(
      'H',
      expect.objectContaining({ locale: 'en' }),
      expect.any(Function)
    )
  })

  it('should ignore inputs', () => {
    renderHook(() =>
      usePaletteNavigation({
        locale: 'en',
        open: true,
        pathname: '/',
        router: mockRouter as any,
        run: mockRun,
      })
    )

    const input = document.createElement('input')
    document.body.appendChild(input)

    // Dispatch event on input
    const event = new KeyboardEvent('keydown', { key: 'h', bubbles: true })
    input.dispatchEvent(event)

    expect(KeyHandlers.handlePaletteKey).not.toHaveBeenCalled()

    document.body.removeChild(input)
  })

  it('should prevent default when handleRequest is called', () => {
    // We need to capture the handleRequest callback passed to handlePaletteKey
    // and invoke it, then check if preventDefault was called on the event.

    let capturedHandler: any = null
    vi.mocked(KeyHandlers.handlePaletteKey).mockImplementation(
      (_key, _deps, handler) => {
        capturedHandler = handler
      }
    )

    renderHook(() =>
      usePaletteNavigation({
        locale: 'en',
        open: true,
        pathname: '/',
        router: mockRouter as any,
        run: mockRun,
      })
    )

    const event = new KeyboardEvent('keydown', { key: 'h', cancelable: true })
    const preventSpy = vi.spyOn(event, 'preventDefault')
    document.dispatchEvent(event)

    expect(capturedHandler).toBeDefined()

    // Simulate the action execution
    const mockAction = vi.fn()
    capturedHandler(mockAction)

    expect(preventSpy).toHaveBeenCalled()
    expect(mockAction).toHaveBeenCalledWith('')
  })
})
