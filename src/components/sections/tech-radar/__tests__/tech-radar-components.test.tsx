import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { TechRadarTooltip } from '../tech-radar-tooltip'
import { TechRadarBlips } from '../tech-radar-blips'
import * as HoverContextModule from '../hover-context' // Import the module to spy on it

// Mock lucide icons
vi.mock('lucide-react', () => ({
  Code: () => <div data-testid="icon-code">Code</div>,
  Circle: () => <div data-testid="icon-circle">Circle</div>,
}))

// Mock skill-icons
vi.mock('@/components/sections/skill-icons', () => ({
  getSkillIcon: () => () => <div data-testid="mock-icon">Icon</div>,
}))

describe('TechRadarTooltip', () => {
  const mockBlips = [
    {
      id: '1',
      name: 'TypeScript',
      quadrant: 'languages',
      xCoordinate: 100,
      yCoordinate: 100,
      iconName: 'typescript',
    },
  ] as any[]

  it('renders nothing when no blip is hovered', () => {
    vi.spyOn(HoverContextModule, 'useHover').mockReturnValue({
      hoveredBlip: null,
      setHoveredBlip: vi.fn(),
    })
    const { container } = render(<TechRadarTooltip blips={mockBlips} />)
    expect(container.firstChild).toBeNull()
  })

  it('renders nothing when hovered blip id is not found', () => {
    vi.spyOn(HoverContextModule, 'useHover').mockReturnValue({
      hoveredBlip: '999',
      setHoveredBlip: vi.fn(),
    })
    const { container } = render(<TechRadarTooltip blips={mockBlips} />)
    expect(container.firstChild).toBeNull()
  })

  it('renders tooltip when blip is hovered', () => {
    // Mock getBoundingClientRect
    const originalGetBoundingClientRect =
      Element.prototype.getBoundingClientRect
    Element.prototype.getBoundingClientRect = vi.fn(() => ({
      width: 500,
      height: 500,
      top: 0,
      left: 0,
      bottom: 500,
      right: 500,
      x: 0,
      y: 0,
      toJSON: () => {},
    }))

    vi.spyOn(HoverContextModule, 'useHover').mockReturnValue({
      hoveredBlip: '1',
      setHoveredBlip: vi.fn(),
    })

    // Wrap in a structure that satisfies the component's DOM requirements
    render(
      <div style={{ width: '500px', height: '500px' }}>
        <svg
          style={{ width: '100%', height: '100%' }}
          viewBox="-220 -220 440 440"
        />
        <TechRadarTooltip blips={mockBlips} />
      </div>
    )

    expect(screen.getByText('TypeScript')).toBeDefined()
    expect(screen.getByText('languages')).toBeDefined()
    expect(screen.getByTestId('mock-icon')).toBeDefined()

    Element.prototype.getBoundingClientRect = originalGetBoundingClientRect
  })
})

describe('TechRadarBlips', () => {
  const mockBlips = [
    {
      id: '1',
      name: 'TypeScript',
      quadrant: 'languages',
      xCoordinate: 10,
      yCoordinate: 10,
    },
    {
      id: '2',
      name: 'React',
      quadrant: 'frameworks',
      xCoordinate: -10,
      yCoordinate: -10,
    },
  ] as any[]

  it('renders all blips', () => {
    vi.spyOn(HoverContextModule, 'useHover').mockReturnValue({
      hoveredBlip: null,
      setHoveredBlip: vi.fn(),
    })
    const { container } = render(<TechRadarBlips blips={mockBlips} />)
    expect(container.querySelectorAll('circle').length).toBe(2)
  })

  it('handles mouse interactions', () => {
    const setHoveredBlip = vi.fn()
    vi.spyOn(HoverContextModule, 'useHover').mockReturnValue({
      hoveredBlip: null,
      setHoveredBlip,
    })

    const { container } = render(<TechRadarBlips blips={mockBlips} />)
    const blipGroups = container.querySelectorAll('g')
    const firstBlip = blipGroups[0]

    expect(firstBlip).toBeDefined()
    if (!firstBlip) return

    // Mouse enter
    fireEvent.mouseEnter(firstBlip)
    expect(setHoveredBlip).toHaveBeenCalledWith('1')

    // Mouse leave
    fireEvent.mouseLeave(firstBlip)
    expect(setHoveredBlip).toHaveBeenCalledWith(null)
  })

  it('applies hover styles', () => {
    vi.spyOn(HoverContextModule, 'useHover').mockReturnValue({
      hoveredBlip: '1',
      setHoveredBlip: vi.fn(),
    })

    const { container } = render(<TechRadarBlips blips={mockBlips} />)
    const blipGroups = container.querySelectorAll('g')

    const firstBlip = blipGroups[0]
    const secondBlip = blipGroups[1]

    expect(firstBlip).toBeDefined()
    expect(secondBlip).toBeDefined()

    if (!firstBlip || !secondBlip) return

    // First blip should be scaled
    const style = firstBlip.getAttribute('style')
    expect(style).toContain('scale(1.5)')

    // Second blip should not be scaled
    const style2 = secondBlip.getAttribute('style')
    expect(style2).toContain('scale(1)')
  })
})
