import { describe, expect, it } from 'vitest'

import { render, screen } from '@testing-library/react'

import { Section, SECTION_BACKGROUNDS } from '../section'

describe('Section', () => {
  it('renders children', () => {
    render(<Section>Test Content</Section>)
    expect(screen.getByText('Test Content')).toBeDefined()
  })

  it('renders as section element', () => {
    const { container } = render(<Section>Content</Section>)
    const section = container.querySelector('section')
    expect(section).toBeDefined()
  })

  it('applies default background classes', () => {
    const { container } = render(<Section>Content</Section>)
    const section = container.querySelector('section')
    expect(section?.className).toContain('bg-background')
  })

  it('applies muted background when specified', () => {
    const { container } = render(
      <Section background={SECTION_BACKGROUNDS.MUTED}>Content</Section>
    )
    const section = container.querySelector('section')
    expect(section?.className).toContain('bg-muted/30')
  })

  it('applies gradient background when specified', () => {
    const { container } = render(
      <Section background={SECTION_BACKGROUNDS.GRADIENT}>Content</Section>
    )
    const section = container.querySelector('section')
    expect(section?.className).toContain('bg-gradient-to-b')
    expect(section?.className).toContain('from-background')
  })

  it('applies base layout classes', () => {
    const { container } = render(<Section>Content</Section>)
    const section = container.querySelector('section')
    expect(section?.className).toContain('relative')
    expect(section?.className).toContain('overflow-hidden')
    expect(section?.className).toContain('px-4')
    expect(section?.className).toContain('py-20')
  })

  it('applies custom className when provided', () => {
    const { container } = render(
      <Section className="custom-class">Content</Section>
    )
    const section = container.querySelector('section')
    expect(section?.className).toContain('custom-class')
  })

  it('passes through HTML section attributes', () => {
    const { container } = render(
      <Section data-testid="test-section" id="test-id">
        Content
      </Section>
    )
    const section = container.querySelector('section')
    expect(section?.id).toBe('test-id')
    expect(section?.getAttribute('data-testid')).toBe('test-section')
  })

  it('merges className with background variant', () => {
    const { container } = render(
      <Section
        background={SECTION_BACKGROUNDS.GRADIENT}
        className="min-h-screen"
      >
        Content
      </Section>
    )
    const section = container.querySelector('section')
    expect(section?.className).toContain('bg-gradient-to-b')
    expect(section?.className).toContain('min-h-screen')
  })
})
