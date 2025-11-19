import '@testing-library/jest-dom'
import '@testing-library/jest-dom/vitest'
import React from 'react'

import { vi } from 'vitest'

/**
 * Light mocks for Next modules commonly imported by components.
 * Extend as needed.
 */
vi.mock('next/navigation', () => {
  const push = vi.fn()
  const replace = vi.fn()
  const prefetch = vi.fn()
  const back = vi.fn()
  const refresh = vi.fn()

  return {
    usePathname: () => '/',
    useRouter: () => ({ back, prefetch, push, refresh, replace }),
    useSearchParams: () => new URLSearchParams(),
  }
})

// Render <Image> as a plain <img>
vi.mock('next/image', () => {
  return {
    default: (properties: any) => {
      const { alt, src, ...rest } = properties ?? {}
      const resolved = typeof src === 'string' ? src : src?.src
      return React.createElement('img', { alt, src: resolved, ...rest })
    },
  }
})

// Render <Link> as a plain <a>
vi.mock('next/link', () => {
  return {
    default: (properties: any) =>
      React.createElement('a', { ...properties }, properties.children),
  }
})

// Mock ResizeObserver
global.ResizeObserver = class ResizeObserver {
  observe() {}
  unobserve() {}
  disconnect() {}
}

// Mock PointerEvent
if (!global.PointerEvent) {
  class PointerEvent extends Event {
    button: number
    ctrlKey: boolean
    metaKey: boolean
    shiftKey: boolean
    altKey: boolean
    constructor(type: string, params: PointerEventInit = {}) {
      super(type, params)
      this.button = params.button ?? 0
      this.ctrlKey = params.ctrlKey ?? false
      this.metaKey = params.metaKey ?? false
      this.shiftKey = params.shiftKey ?? false
      this.altKey = params.altKey ?? false
    }
  }
  global.PointerEvent = PointerEvent as any
}

// Mock HTMLElement methods
Object.assign(window.HTMLElement.prototype, {
  scrollIntoView: vi.fn(),
  releasePointerCapture: vi.fn(),
  hasPointerCapture: vi.fn(),
})
