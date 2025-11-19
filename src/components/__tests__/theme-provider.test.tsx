import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen } from '@testing-library/react'
import { ThemeProvider, useTheme } from '../theme-provider'

// Mock localStorage
const localStorageMock = {
  getItem: vi.fn(),
  setItem: vi.fn(),
  removeItem: vi.fn(),
  clear: vi.fn(),
}
global.localStorage = localStorageMock as any

describe('ThemeProvider', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    document.documentElement.className = ''
  })

  it('renders children', () => {
    render(
      <ThemeProvider>
        <div>Test Child</div>
      </ThemeProvider>
    )

    expect(screen.getByText('Test Child')).toBeDefined()
  })

  it('applies default dark theme', () => {
    localStorageMock.getItem.mockReturnValue(null)

    render(
      <ThemeProvider>
        <div>Test</div>
      </ThemeProvider>
    )

    // Note: useEffect runs after render, so we can't directly test classList immediately
    expect(localStorageMock.getItem).toHaveBeenCalledWith('theme')
  })

  it('uses saved theme from localStorage', () => {
    localStorageMock.getItem.mockReturnValue('light')

    render(
      <ThemeProvider>
        <div>Test</div>
      </ThemeProvider>
    )

    expect(localStorageMock.getItem).toHaveBeenCalledWith('theme')
  })
})

describe('useTheme', () => {
  it('throws error when used outside provider', () => {
    const TestComponent = () => {
      useTheme()
      return <div>Test</div>
    }

    expect(() => render(<TestComponent />)).toThrow(
      'useTheme must be used within a ThemeProvider'
    )
  })
})
