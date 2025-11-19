import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import { Badge } from '../ui/badge'

describe('Badge', () => {
  it('renders children', () => {
    render(<Badge>Test Badge</Badge>)
    expect(screen.getByText('Test Badge')).toBeDefined()
  })

  it('renders with default variant', () => {
    const { container } = render(<Badge>Default</Badge>)
    expect(container.querySelector('[class*="bg-primary"]')).toBeDefined()
  })

  it('renders with secondary variant', () => {
    const { container } = render(<Badge variant="secondary">Secondary</Badge>)
    expect(container.querySelector('[class*="bg-secondary"]')).toBeDefined()
  })

  it('renders with destructive variant', () => {
    const { container } = render(
      <Badge variant="destructive">Destructive</Badge>
    )
    expect(container.querySelector('[class*="bg-destructive"]')).toBeDefined()
  })

  it('renders with outline variant', () => {
    const { container } = render(<Badge variant="outline">Outline</Badge>)
    expect(container.querySelector('[class*="border"]')).toBeDefined()
  })
})
