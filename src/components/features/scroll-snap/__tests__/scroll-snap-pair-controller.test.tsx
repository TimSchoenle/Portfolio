import { render, fireEvent } from '@testing-library/react'
import { vi, describe, it, expect, beforeEach } from 'vitest'
import { ScrollSnapPairController } from '../scroll-snap-pair-controller'

describe('ScrollSnapPairController', () => {
  const mockScrollTo = vi.fn()

  beforeEach(() => {
    vi.clearAllMocks()
    window.scrollTo = mockScrollTo
    document.body.innerHTML = `
            <div id="top-section" style="height: 100vh;"></div>
            <div id="bottom-section" style="height: 100vh;"></div>
        `
    // Mock getBoundingClientRect
    Element.prototype.getBoundingClientRect = vi.fn(() => ({
      top: 0,
      bottom: 0,
      left: 0,
      right: 0,
      width: 0,
      height: 0,
      x: 0,
      y: 0,
      toJSON: () => {},
    }))
  })

  const renderController = () => {
    render(
      <ScrollSnapPairController
        topSectionId="top-section"
        bottomSectionId="bottom-section"
      />
    )
  }

  it('should ignore small deltas', () => {
    renderController()

    fireEvent.wheel(window, { deltaY: 5 })
    expect(mockScrollTo).not.toHaveBeenCalled()
  })

  it('should snap down when conditions met', () => {
    // Setup geometry for snap down
    // Top section is near top (0)
    // Bottom section is below viewport (> 64)
    document.getElementById('top-section')!.getBoundingClientRect = vi
      .fn()
      .mockReturnValue({
        top: 0,
        bottom: 800,
      } as DOMRect)
    document.getElementById('bottom-section')!.getBoundingClientRect = vi
      .fn()
      .mockReturnValue({
        top: 800,
        bottom: 1600,
      } as DOMRect)

    renderController()

    fireEvent.wheel(window, { deltaY: 100 })

    expect(mockScrollTo).toHaveBeenCalledWith(
      expect.objectContaining({
        top: 800,
      })
    )
  })

  it('should snap up when conditions met', () => {
    // Setup geometry for snap up
    // Bottom section is near top (0)
    // Top section is above viewport (< 0)
    document.getElementById('top-section')!.getBoundingClientRect = vi
      .fn()
      .mockReturnValue({
        top: -800,
        bottom: 0,
      } as DOMRect)
    document.getElementById('bottom-section')!.getBoundingClientRect = vi
      .fn()
      .mockReturnValue({
        top: 0,
        bottom: 800,
      } as DOMRect)

    renderController()

    // Negative delta for scroll up
    fireEvent.wheel(window, { deltaY: -100 })

    expect(mockScrollTo).toHaveBeenCalledWith(
      expect.objectContaining({
        top: -800,
      })
    )
  })

  it('should not snap if body is locked', () => {
    document.body.style.overflow = 'hidden'
    renderController()
    fireEvent.wheel(window, { deltaY: 100 })
    expect(mockScrollTo).not.toHaveBeenCalled()
  })
})
