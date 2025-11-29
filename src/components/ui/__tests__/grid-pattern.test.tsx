import { describe, expect, it } from 'vitest'

import { render } from '@testing-library/react'

import { GridPattern } from '../grid-pattern'

describe('GridPattern', () => {
  it('renders a div element', () => {
    const { container } = render(<GridPattern />)
    const pattern = container.firstChild as HTMLElement
    expect(pattern.tagName).toBe('DIV')
  })

  it('applies base classes', () => {
    const { container } = render(<GridPattern />)
    const pattern = container.firstChild as HTMLElement
    expect(pattern.className).toContain('absolute')
    expect(pattern.className).toContain('inset-0')
    expect(pattern.className).toContain('-z-10')
  })

  it('applies grid gradient background', () => {
    const { container } = render(<GridPattern />)
    const pattern = container.firstChild as HTMLElement
    expect(pattern.className).toContain('bg-[linear-gradient')
  })

  it('uses default size of 32px when not specified', () => {
    const { container } = render(<GridPattern />)
    const pattern = container.firstChild as HTMLElement
    expect(pattern.style.backgroundSize).toBe('32px 32px')
  })

  it('uses custom size when provided', () => {
    const { container } = render(<GridPattern size={24} />)
    const pattern = container.firstChild as HTMLElement
    expect(pattern.style.backgroundSize).toBe('24px 24px')
  })

  it('uses default offset of 0 when not specified', () => {
    const { container } = render(<GridPattern />)
    const pattern = container.firstChild as HTMLElement
    expect(pattern.style.backgroundPosition).toBe('0px 0px')
  })

  it('uses custom offsetX when provided', () => {
    const { container } = render(<GridPattern offsetX={10} />)
    const pattern = container.firstChild as HTMLElement
    expect(pattern.style.backgroundPosition).toBe('10px 0px')
  })

  it('uses custom offsetY when provided', () => {
    const { container } = render(<GridPattern offsetY={20} />)
    const pattern = container.firstChild as HTMLElement
    expect(pattern.style.backgroundPosition).toBe('0px 20px')
  })

  it('uses both custom offset values when provided', () => {
    const { container } = render(<GridPattern offsetX={10} offsetY={20} />)
    const pattern = container.firstChild as HTMLElement
    expect(pattern.style.backgroundPosition).toBe('10px 20px')
  })

  it('applies custom className when provided', () => {
    const { container } = render(<GridPattern className="custom-pattern" />)
    const pattern = container.firstChild as HTMLElement
    expect(pattern.className).toContain('custom-pattern')
  })

  it('merges custom className with base classes', () => {
    const { container } = render(<GridPattern className="opacity-50" />)
    const pattern = container.firstChild as HTMLElement
    expect(pattern.className).toContain('absolute')
    expect(pattern.className).toContain('opacity-50')
  })
})
