import { render, screen, fireEvent } from '@testing-library/react'
import { describe, it, expect, beforeEach } from 'vitest'
import { ThemeProvider, useTheme } from '../theme-provider'

const TestComponent = () => {
  const { theme, setTheme } = useTheme()
  return (
    <div>
      <span data-testid="theme-value">{theme}</span>
      <button onClick={() => setTheme('dark')}>Set Dark</button>
      <button onClick={() => setTheme('light')}>Set Light</button>
    </div>
  )
}

describe('ThemeProvider', () => {
  beforeEach(() => {
    localStorage.clear()
    document.documentElement.className = ''
  })

  it('should use default theme if no storage', () => {
    render(
      <ThemeProvider defaultTheme="light">
        <TestComponent />
      </ThemeProvider>
    )
    expect(screen.getByTestId('theme-value').textContent).toBe('light')
  })

  it('should load theme from local storage', () => {
    localStorage.setItem('theme', 'light')
    render(
      <ThemeProvider defaultTheme="dark">
        <TestComponent />
      </ThemeProvider>
    )
    expect(screen.getByTestId('theme-value').textContent).toBe('light')
    expect(document.documentElement.classList.contains('light')).toBe(true)
  })

  it('should change theme', () => {
    render(
      <ThemeProvider defaultTheme="light">
        <TestComponent />
      </ThemeProvider>
    )

    const button = screen.getByText('Set Dark')
    fireEvent.click(button)

    expect(screen.getByTestId('theme-value').textContent).toBe('dark')
    expect(localStorage.getItem('theme')).toBe('dark')
    expect(document.documentElement.classList.contains('dark')).toBe(true)
  })

  it('should throw error if useTheme is used outside provider', () => {
    expect(() => {
      render(<TestComponent />)
    }).toThrow('useTheme must be used within a ThemeProvider')
  })

  // Add missing check for other branches if any
})
