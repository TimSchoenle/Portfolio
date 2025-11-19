import { describe, it, expect, vi } from 'vitest'

// Mock next/dynamic
vi.mock('next/dynamic', () => ({
  default: vi.fn((loader, options) => {
    // Execute loader to verify it works
    loader()
    return {
      loader,
      options,
      __mockType: 'dynamic',
    }
  }),
}))

describe('lazyClient', () => {
  it('creates a dynamic component with selector', async () => {
    const { lazyClient } = await import('../lazy-client')

    const mockImporter = vi.fn().mockResolvedValue({
      MyComponent: 'MyComponent',
    })
    const mockSelect = vi.fn((mod: any) => mod.MyComponent)

    const result = lazyClient(mockImporter, mockSelect as any)

    expect(result).toBeDefined()
    // @ts-ignore
    expect(result.__mockType).toBe('dynamic')
  })

  it('handles loading option', async () => {
    const { lazyClient } = await import('../lazy-client')

    const mockImporter = vi.fn().mockResolvedValue({})
    const mockSelect = vi.fn(() => 'Component')
    const Loading = () => <div>Loading </div>

    const result = lazyClient(mockImporter, mockSelect as any, {
      loading: Loading,
    })

    // @ts-ignore
    expect(result.options.loading).toBe(Loading)
  })

  it('uses default loading if not provided', async () => {
    const { lazyClient } = await import('../lazy-client')

    const mockImporter = vi.fn().mockResolvedValue({})
    const mockSelect = vi.fn(() => 'Component')

    const result = lazyClient(mockImporter, mockSelect as any)

    // @ts-ignore
    expect(result.options.loading()).toBeNull()
  })
})

describe('lazyClientDefault', () => {
  it('creates a dynamic component from default export', async () => {
    const { lazyClientDefault } = await import('../lazy-client')

    const mockImporter = vi.fn().mockResolvedValue({
      default: 'DefaultComponent',
    })

    const result = lazyClientDefault(mockImporter)

    expect(result).toBeDefined()
  })

  it('handles loading option', async () => {
    const { lazyClientDefault } = await import('../lazy-client')

    const mockImporter = vi.fn().mockResolvedValue({ default: 'Comp' })
    const Loading = () => <div>Loading </div>

    const result = lazyClientDefault(mockImporter, { loading: Loading })

    // @ts-ignore
    expect(result.options.loading).toBe(Loading)
  })
})
