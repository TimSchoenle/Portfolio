import { describe, expect, it } from 'vitest'

import { render, screen } from '@testing-library/react'

import { SectionContainer } from '../section-container'

describe('SectionContainer', () => {
  it('renders children', () => {
    render(<SectionContainer>Test Content</SectionContainer>)
    expect(screen.getByText('Test Content')).toBeDefined()
  })

  it('renders as div element', () => {
    const { container } = render(<SectionContainer>Content</SectionContainer>)
    const div = container.firstChild as HTMLElement
    expect(div.tagName).toBe('DIV')
  })

  it('applies base classes', () => {
    const { container } = render(<SectionContainer>Content</SectionContainer>)
    const div = container.firstChild as HTMLElement
    expect(div.className).toContain('mx-auto')
    expect(div.className).toContain('w-full')
  })

  it('applies sm size by default', () => {
    const { container } = render(<SectionContainer>Content</SectionContainer>)
    const div = container.firstChild as HTMLElement
    expect(div.className).toContain('max-w-4xl')
  })

  it('applies sm size when specified', () => {
    const { container } = render(
      <SectionContainer size="sm">Content</SectionContainer>
    )
    const div = container.firstChild as HTMLElement
    expect(div.className).toContain('max-w-4xl')
  })

  it('applies md size when specified', () => {
    const { container } = render(
      <SectionContainer size="md">Content</SectionContainer>
    )
    const div = container.firstChild as HTMLElement
    expect(div.className).toContain('max-w-5xl')
  })

  it('applies lg size when specified', () => {
    const { container } = render(
      <SectionContainer size="lg">Content</SectionContainer>
    )
    const div = container.firstChild as HTMLElement
    expect(div.className).toContain('max-w-6xl')
  })

  it('applies xl size when specified', () => {
    const { container } = render(
      <SectionContainer size="xl">Content</SectionContainer>
    )
    const div = container.firstChild as HTMLElement
    expect(div.className).toContain('max-w-7xl')
  })

  it('applies custom className when provided', () => {
    const { container } = render(
      <SectionContainer className="relative">Content</SectionContainer>
    )
    const div = container.firstChild as HTMLElement
    expect(div.className).toContain('relative')
  })

  it('merges className with size variant', () => {
    const { container } = render(
      <SectionContainer className="relative" size="xl">
        Content
      </SectionContainer>
    )
    const div = container.firstChild as HTMLElement
    expect(div.className).toContain('max-w-7xl')
    expect(div.className).toContain('relative')
  })

  it('passes through HTML div attributes', () => {
    const { container } = render(
      <SectionContainer data-testid="container" id="test-container">
        Content
      </SectionContainer>
    )
    const div = container.firstChild as HTMLElement
    expect(div.id).toBe('test-container')
    expect(div.getAttribute('data-testid')).toBe('container')
  })
})
