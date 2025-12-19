import '@testing-library/jest-dom/vitest'
import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'

import { BlueprintLabel } from '../blueprint-label'

describe('BlueprintLabel', () => {
  it('renders children correctly', () => {
    render(<BlueprintLabel>Test Content</BlueprintLabel>)
    const element = screen.getByText('Test Content')
    expect(element).toBeInTheDocument()
  })

  it('applies aria-hidden="true"', () => {
    render(<BlueprintLabel>Hidden Content</BlueprintLabel>)
    const element = screen.getByText('Hidden Content')
    expect(element).toHaveAttribute('aria-hidden', 'true')
  })

  it('applies role="img"', () => {
    render(<BlueprintLabel>Image Content</BlueprintLabel>)
    const element = screen.getByText('Image Content')
    expect(element).toHaveAttribute('role', 'img')
  })

  it('renders as a different element', () => {
    render(<BlueprintLabel as="span">Span Content</BlueprintLabel>)
    const element = screen.getByText('Span Content')
    expect(element.tagName).toBe('SPAN')
  })

  it('applies custom className', () => {
    render(
      <BlueprintLabel className="custom-class">Class Content</BlueprintLabel>
    )
    const element = screen.getByText('Class Content')
    expect(element).toHaveClass('custom-class')
  })

  it('forwards style prop', () => {
    const style = { color: 'red' }
    render(<BlueprintLabel style={style}>Style Content</BlueprintLabel>)
    const element = screen.getByText('Style Content')
    expect(element).toHaveStyle('color: rgb(255, 0, 0)') // JSDOM computed style
  })
})
