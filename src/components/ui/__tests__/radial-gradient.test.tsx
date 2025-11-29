import { describe, expect, it } from 'vitest'

import { render } from '@testing-library/react'

import { RadialGradient } from '../radial-gradient'

describe('RadialGradient', () => {
  it('renders a div element', () => {
    const { container } = render(<RadialGradient />)
    const gradient = container.firstChild as HTMLElement
    expect(gradient.tagName).toBe('DIV')
  })

  it('applies base classes', () => {
    const { container } = render(<RadialGradient />)
    const gradient = container.firstChild as HTMLElement
    expect(gradient.className).toContain('absolute')
    expect(gradient.className).toContain('-z-10')
  })

  it('applies top-right position by default', () => {
    const { container } = render(<RadialGradient />)
    const gradient = container.firstChild as HTMLElement
    expect(gradient.className).toContain('top-0')
    expect(gradient.className).toContain('right-0')
  })

  it('applies top-left position when specified', () => {
    const { container } = render(<RadialGradient position="top-left" />)
    const gradient = container.firstChild as HTMLElement
    expect(gradient.className).toContain('top-0')
    expect(gradient.className).toContain('left-0')
  })

  it('applies bottom-left position when specified', () => {
    const { container } = render(<RadialGradient position="bottom-left" />)
    const gradient = container.firstChild as HTMLElement
    expect(gradient.className).toContain('bottom-0')
    expect(gradient.className).toContain('left-0')
  })

  it('applies bottom-right position when specified', () => {
    const { container } = render(<RadialGradient position="bottom-right" />)
    const gradient = container.firstChild as HTMLElement
    expect(gradient.className).toContain('bottom-0')
    expect(gradient.className).toContain('right-0')
  })

  it('uses default size of 600px when not specified', () => {
    const { container } = render(<RadialGradient />)
    const gradient = container.firstChild as HTMLElement
    expect(gradient.style.width).toBe('600px')
    expect(gradient.style.height).toBe('600px')
  })

  it('uses custom size when provided', () => {
    const { container } = render(<RadialGradient size={800} />)
    const gradient = container.firstChild as HTMLElement
    expect(gradient.style.width).toBe('800px')
    expect(gradient.style.height).toBe('800px')
  })

  it('applies radial gradient background image with default size', () => {
    const { container } = render(<RadialGradient />)
    const gradient = container.firstChild as HTMLElement
    expect(gradient.style.backgroundImage).toContain('radial-gradient')
    expect(gradient.style.backgroundImage).toContain('circle 600px')
  })

  it('applies radial gradient background image with custom size', () => {
    const { container } = render(<RadialGradient size={800} />)
    const gradient = container.firstChild as HTMLElement
    expect(gradient.style.backgroundImage).toContain('circle 800px')
  })

  it('includes primary color in gradient', () => {
    const { container } = render(<RadialGradient />)
    const gradient = container.firstChild as HTMLElement
    expect(gradient.style.backgroundImage).toContain('rgba(var(--primary-rgb')
  })

  it('applies custom className when provided', () => {
    const { container } = render(<RadialGradient className="opacity-50" />)
    const gradient = container.firstChild as HTMLElement
    expect(gradient.className).toContain('opacity-50')
  })

  it('merges custom className with position classes', () => {
    const { container } = render(
      <RadialGradient className="blur-3xl" position="bottom-left" />
    )
    const gradient = container.firstChild as HTMLElement
    expect(gradient.className).toContain('bottom-0')
    expect(gradient.className).toContain('left-0')
    expect(gradient.className).toContain('blur-3xl')
  })
})
