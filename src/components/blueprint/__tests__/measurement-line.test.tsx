import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'

import { MeasurementLine } from '../measurement-line'

describe('MeasurementLine', () => {
  it('renders correctly', () => {
    const { container } = render(<MeasurementLine />)
    expect(container.firstChild).toBeInTheDocument()
  })

  it('applies aria-hidden="true" to container', () => {
    const { container } = render(<MeasurementLine />)
    expect(container.firstChild).toHaveAttribute('aria-hidden', 'true')
  })

  it('applies role="img" to container', () => {
    const { container } = render(<MeasurementLine />)
    expect(container.firstChild).toHaveAttribute('role', 'img') // Or inherited from BlueprintLabel logic if wrapper
  })

  it('renders SVG element', () => {
    const { container } = render(<MeasurementLine />)
    const svg = container.querySelector('svg')
    expect(svg).toBeInTheDocument()
  })

  it('renders label text', () => {
    render(<MeasurementLine label="TEST_LABEL" />)
    expect(screen.getByText('TEST_LABEL')).toBeInTheDocument()
  })

  it('applies dynamic style for dimensions (horizontal)', () => {
    const { container } = render(
      <MeasurementLine orientation="horizontal" width="500px" />
    )
    // Note: The structure is <div style={{width: 500px, height: 10px}}>...</div>
    expect(container.firstChild).toHaveStyle('width: 500px')
    expect(container.firstChild).toHaveStyle('height: 10px')
  })

  it('applies dynamic style for dimensions (vertical)', () => {
    const { container } = render(
      <MeasurementLine orientation="vertical" width="300px" />
    )
    expect(container.firstChild).toHaveStyle('height: 300px')
    expect(container.firstChild).toHaveStyle('width: 10px')
  })
})
