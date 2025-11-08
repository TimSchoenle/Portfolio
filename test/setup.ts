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
