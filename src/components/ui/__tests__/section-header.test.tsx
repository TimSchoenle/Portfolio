import { describe, expect, it } from 'vitest'

import { render, screen } from '@testing-library/react'

import { SectionHeader } from '../section-header'

describe('SectionHeader', () => {
  it('renders with required title prop', () => {
    render(<SectionHeader title="Test Title" />)
    expect(screen.getByText('Test Title')).toBeDefined()
  })

  it('renders title as h2 heading', () => {
    const { container } = render(<SectionHeader title="Test Title" />)
    const heading = container.querySelector('h2')
    expect(heading).toBeDefined()
    expect(heading?.textContent).toBe('Test Title')
  })

  it('renders subtitle when provided', () => {
    render(<SectionHeader subtitle="Test Subtitle" title="Test Title" />)
    expect(screen.getByText('Test Subtitle')).toBeDefined()
  })

  it('does not render subtitle when not provided', () => {
    const { container } = render(<SectionHeader title="Test Title" />)
    const subtitle = container.querySelector('p')
    expect(subtitle).toBeNull()
  })

  it('applies gradient classes when gradient is true', () => {
    const { container } = render(
      <SectionHeader gradient={true} title="Test Title" />
    )
    const heading = container.querySelector('h2')
    expect(heading?.className).toContain('bg-gradient-to-r')
    expect(heading?.className).toContain('text-transparent')
  })

  it('applies foreground color when gradient is false', () => {
    const { container } = render(
      <SectionHeader gradient={false} title="Test Title" />
    )
    const heading = container.querySelector('h2')
    expect(heading?.className).toContain('text-foreground')
  })

  it('renders underline when underline is true', () => {
    const { container } = render(
      <SectionHeader title="Test Title" underline={true} />
    )
    const underline = container.querySelector('.h-1\\.5')
    expect(underline).toBeDefined()
  })

  it('does not render underline when underline is false', () => {
    const { container } = render(
      <SectionHeader title="Test Title" underline={false} />
    )
    const underline = container.querySelector('.h-1\\.5')
    expect(underline).toBeNull()
  })

  it('applies center alignment by default', () => {
    const { container } = render(<SectionHeader title="Test Title" />)
    const wrapper = container.firstChild as HTMLElement
    expect(wrapper.className).toContain('text-center')
    expect(wrapper.className).toContain('items-center')
  })

  it('applies left alignment when specified', () => {
    const { container } = render(
      <SectionHeader align="left" title="Test Title" />
    )
    const wrapper = container.firstChild as HTMLElement
    expect(wrapper.className).toContain('text-left')
    expect(wrapper.className).toContain('items-start')
  })

  it('applies right alignment when specified', () => {
    const { container } = render(
      <SectionHeader align="right" title="Test Title" />
    )
    const wrapper = container.firstChild as HTMLElement
    expect(wrapper.className).toContain('text-right')
    expect(wrapper.className).toContain('items-end')
  })

  it('applies custom className when provided', () => {
    const { container } = render(
      <SectionHeader className="custom-class" title="Test Title" />
    )
    const wrapper = container.firstChild as HTMLElement
    expect(wrapper.className).toContain('custom-class')
  })

  it('renders all features together', () => {
    const { container } = render(
      <SectionHeader
        align="left"
        className="custom"
        gradient={true}
        subtitle="Subtitle"
        title="Title"
        underline={true}
      />
    )

    // Check title
    expect(screen.getByText('Title')).toBeDefined()

    // Check subtitle
    expect(screen.getByText('Subtitle')).toBeDefined()

    // Check gradient
    const heading = container.querySelector('h2')
    expect(heading?.className).toContain('bg-gradient-to-r')

    // Check underline
    const underline = container.querySelector('.h-1\\.5')
    expect(underline).toBeDefined()

    // Check alignment
    const wrapper = container.firstChild as HTMLElement
    expect(wrapper.className).toContain('text-left')

    // Check custom class
    expect(wrapper.className).toContain('custom')
  })
})
