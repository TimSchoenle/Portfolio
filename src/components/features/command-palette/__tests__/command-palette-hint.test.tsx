import { render, screen, waitFor } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

import { CommandPaletteHint } from '../command-palette-hint'

describe('CommandPaletteHint', () => {
  const originalNavigator = global.navigator

  beforeEach(() => {
    // Mock navigator.userAgent
    Object.defineProperty(global, 'navigator', {
      value: {
        userAgent: 'test-agent',
        keyboard: {
          getLayoutMap: vi.fn(),
        },
      },
      writable: true,
      configurable: true,
    })
  })

  afterEach(() => {
    Object.defineProperty(global, 'navigator', {
      value: originalNavigator,
      writable: true,
      configurable: true,
    })
  })

  it('renders nothing if navigator.keyboard is not available', async () => {
    Object.defineProperty(global, 'navigator', {
      value: { userAgent: 'test-agent' }, // No keyboard property
      writable: true,
      configurable: true,
    })

    const { container } = render(<CommandPaletteHint />)

    // Wait for effect to run
    await waitFor(() => {
      expect(container).toBeEmptyDOMElement()
    })
  })

  it('renders nothing if layout map fails', async () => {
    const mockGetLayoutMap = vi.fn().mockRejectedValue(new Error('Failed'))
    Object.defineProperty(global, 'navigator', {
      value: {
        userAgent: 'test-agent',
        keyboard: {
          getLayoutMap: mockGetLayoutMap,
        },
      },
      writable: true,
      configurable: true,
    })

    const { container } = render(<CommandPaletteHint />)
    await waitFor(() => {
      expect(container).toBeEmptyDOMElement()
    })
  })

  it('renders correct shortcut for default OS (non-Mac)', async () => {
    const mockGetLayoutMap = vi
      .fn()
      .mockResolvedValue(new Map([['ControlLeft', 'Ctrl']]))
    Object.defineProperty(global, 'navigator', {
      value: {
        userAgent: 'Windows NT 10.0',
        keyboard: {
          getLayoutMap: mockGetLayoutMap,
        },
      },
      writable: true,
      configurable: true,
    })

    render(<CommandPaletteHint />)

    await waitFor(() => {
      expect(screen.getByText('Ctrl+K')).toBeInTheDocument()
    })
  })

  it('renders correct shortcut for Mac', async () => {
    const mockGetLayoutMap = vi
      .fn()
      .mockResolvedValue(new Map([['MetaLeft', 'Cmd']]))
    Object.defineProperty(global, 'navigator', {
      value: {
        userAgent: 'Macintosh',
        keyboard: {
          getLayoutMap: mockGetLayoutMap,
        },
      },
      writable: true,
      configurable: true,
    })

    render(<CommandPaletteHint />)

    await waitFor(() => {
      expect(screen.getByText('Cmd+K')).toBeInTheDocument()
    })
  })
})
