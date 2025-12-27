import { render, screen, fireEvent } from '@testing-library/react'
import { vi, describe, it, expect, beforeEach } from 'vitest'
import { ActionsGroup } from '../actions-group'

// Mock dependencies
vi.mock('@/components/ui/command', () => ({
  CommandGroup: ({ children, heading }: any) => (
    <div data-testid="group" title={heading}>
      {children}
    </div>
  ),
  CommandItem: ({ children, onSelect, value }: any) => (
    <div data-testid={`item-${value}`} onClick={() => onSelect(value)}>
      {children}
    </div>
  ),
  CommandShortcut: ({ children }: any) => <span>{children}</span>,
}))

vi.mock('../../utils/actions', () => ({
  openNewTab: vi.fn(),
  sendMailTo: vi.fn(),
}))

vi.mock('@/data/config', () => ({
  siteConfig: {
    socials: {
      linkedin: 'https://linkedin.com/test',
      github: 'https://github.com/test',
    },
    email: 'test@example.com',
  },
}))

import { openNewTab, sendMailTo } from '../../utils/actions'

describe('ActionsGroup', () => {
  const mockRun = vi.fn((cb) => cb())
  const mockT = vi.fn((key) => key)

  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('should render all actions', () => {
    render(<ActionsGroup run={mockRun} tPalette={mockT as any} />)
    expect(screen.getByTestId('item-email')).toBeInTheDocument()
    expect(screen.getByTestId('item-linkedin')).toBeInTheDocument()
    expect(screen.getByTestId('item-github')).toBeInTheDocument()
  })

  it('should trigger email action', () => {
    render(<ActionsGroup run={mockRun} tPalette={mockT as any} />)
    fireEvent.click(screen.getByTestId('item-email'))
    expect(mockRun).toHaveBeenCalled()
    expect(sendMailTo).toHaveBeenCalledWith('test@example.com')
  })

  it('should trigger linkedin action', () => {
    render(<ActionsGroup run={mockRun} tPalette={mockT as any} />)
    fireEvent.click(screen.getByTestId('item-linkedin'))
    expect(mockRun).toHaveBeenCalled()
    expect(openNewTab).toHaveBeenCalledWith('https://linkedin.com/test')
  })

  it('should trigger github action', () => {
    render(<ActionsGroup run={mockRun} tPalette={mockT as any} />)
    fireEvent.click(screen.getByTestId('item-github'))
    expect(mockRun).toHaveBeenCalled()
    expect(openNewTab).toHaveBeenCalledWith('https://github.com/test')
  })
})
