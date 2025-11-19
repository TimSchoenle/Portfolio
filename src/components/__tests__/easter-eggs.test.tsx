import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, fireEvent, act } from '@testing-library/react'

// Mock sonner
const mockToast = vi.fn()
const mockSuccess = vi.fn()
vi.mock('sonner', () => ({
  toast: Object.assign((msg: string, opts: any) => mockToast(msg, opts), {
    success: (msg: string, opts: any) => mockSuccess(msg, opts),
  }),
}))

describe('EasterEggs', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('triggers Konami code', async () => {
    const { EasterEggs } = await import('../easter-eggs')
    render(<EasterEggs />)

    const sequence = [
      'ArrowUp',
      'ArrowUp',
      'ArrowDown',
      'ArrowDown',
      'ArrowLeft',
      'ArrowRight',
      'ArrowLeft',
      'ArrowRight',
      'b',
      'a',
    ]

    await act(async () => {
      sequence.forEach((key) => {
        fireEvent.keyDown(document, { key })
      })
    })

    expect(mockSuccess).toHaveBeenCalledWith(
      expect.stringContaining('Konami Code Activated'),
      expect.any(Object)
    )
  })

  it('triggers triple click', async () => {
    const { EasterEggs } = await import('../easter-eggs')
    render(<EasterEggs />)

    await act(async () => {
      fireEvent.click(document)
      fireEvent.click(document)
      fireEvent.click(document)
    })

    expect(mockToast).toHaveBeenCalled()
  })

  it('resets click count after timeout', async () => {
    const { EasterEggs } = await import('../easter-eggs')
    render(<EasterEggs />)

    await act(async () => {
      fireEvent.click(document)
      fireEvent.click(document)
    })

    // Wait for timeout
    await act(async () => {
      vi.advanceTimersByTime(600)
    })

    // Click again (should be 1st click now)
    await act(async () => {
      fireEvent.click(document)
    })

    expect(mockToast).not.toHaveBeenCalled()
  })
})
