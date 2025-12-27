import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'

import { Heading } from '../heading'

describe('Heading', () => {
  it('renders a default h2 heading', () => {
    render(<Heading>Default Heading</Heading>)
    const heading = screen.getByRole('heading', { level: 2 })
    expect(heading).toBeInTheDocument()
    expect(heading).toHaveTextContent('Default Heading')
    expect(heading).toHaveAttribute('data-heading-tag', 'H2')
  })

  it('renders an h1 when as="h1" is passed', () => {
    render(<Heading as="h1">H1 Heading</Heading>)
    const heading = screen.getByRole('heading', { level: 1 })
    expect(heading).toBeInTheDocument()
    expect(heading).toHaveTextContent('H1 Heading')
    expect(heading).toHaveAttribute('data-heading-tag', 'H1')
  })

  it('renders an h3 when as="h3" is passed', () => {
    render(<Heading as="h3">H3 Heading</Heading>)
    const heading = screen.getByRole('heading', { level: 3 })
    expect(heading).toBeInTheDocument()
    expect(heading).toHaveAttribute('data-heading-tag', 'H3')
  })

  it('passes through additional props', () => {
    render(
      <Heading as="h4" className="custom-class" id="test-id">
        Custom Heading
      </Heading>
    )
    const heading = screen.getByRole('heading', { level: 4 })
    expect(heading).toHaveClass('custom-class')
    expect(heading).toHaveAttribute('id', 'test-id')
  })
})
