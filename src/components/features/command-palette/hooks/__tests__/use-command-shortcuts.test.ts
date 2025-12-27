import { renderHook } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

import { useCommandShortcuts } from '../use-command-shortcuts'

describe('useCommandShortcuts', () => {
  it('toggles open state when Cmd+K is pressed', () => {
    const setOpen = vi.fn()
    renderHook(() => useCommandShortcuts({ setOpen }))

    const event = new KeyboardEvent('keydown', {
      key: 'k',
      metaKey: true,
      bubbles: true,
    })
    document.dispatchEvent(event)

    expect(setOpen).toHaveBeenCalled()
    // Check if the callback function passed to setOpen toggles the value
    const updateFn = setOpen.mock.calls[0]?.[0]
    expect(updateFn).toBeDefined()
    expect(updateFn(false)).toBe(true)
    expect(updateFn(true)).toBe(false)
  })

  it('toggles open state when Ctrl+K is pressed', () => {
    const setOpen = vi.fn()
    renderHook(() => useCommandShortcuts({ setOpen }))

    const event = new KeyboardEvent('keydown', {
      key: 'k',
      ctrlKey: true,
      bubbles: true,
    })
    document.dispatchEvent(event)

    expect(setOpen).toHaveBeenCalled()
  })

  it('does not toggle when only k is pressed', () => {
    const setOpen = vi.fn()
    renderHook(() => useCommandShortcuts({ setOpen }))

    const event = new KeyboardEvent('keydown', {
      key: 'k',
      metaKey: false,
      ctrlKey: false,
      bubbles: true,
    })
    document.dispatchEvent(event)

    expect(setOpen).not.toHaveBeenCalled()
  })

  it('removes event listener on unmount', () => {
    const setOpen = vi.fn()
    const { unmount } = renderHook(() => useCommandShortcuts({ setOpen }))

    unmount()

    const event = new KeyboardEvent('keydown', {
      key: 'k',
      metaKey: true,
      bubbles: true,
    })
    document.dispatchEvent(event)

    expect(setOpen).not.toHaveBeenCalled()
  })
})
