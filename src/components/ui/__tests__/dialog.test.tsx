import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import {
  Dialog,
  DialogTrigger,
  DialogContent,
  DialogHeader,
  DialogFooter,
  DialogTitle,
  DialogDescription,
} from '../dialog'

// Mock Radix UI Dialog
vi.mock('@radix-ui/react-dialog', () => ({
  Root: ({ children, ...props }: any) => (
    <div data-dialog="root" {...props}>
      {children}
    </div>
  ),
  Trigger: ({ children, ...props }: any) => (
    <button data-dialog="trigger" {...props}>
      {children}
    </button>
  ),
  Portal: ({ children }: any) => <div data-dialog="portal">{children}</div>,
  Overlay: ({ ...props }: any) => <div data-dialog="overlay" {...props} />,
  Content: ({ children, ...props }: any) => (
    <div data-dialog="content" {...props}>
      {children}
    </div>
  ),
  Close: ({ children, ...props }: any) => (
    <button data-dialog="close" {...props}>
      {children}
    </button>
  ),
  Title: ({ children, ...props }: any) => (
    <h2 data-dialog="title" {...props}>
      {children}
    </h2>
  ),
  Description: ({ children, ...props }: any) => (
    <p data-dialog="description" {...props}>
      {children}
    </p>
  ),
}))

// Mock lucide icons
vi.mock('lucide-react', () => ({
  XIcon: () => <div data-testid="x-icon">X</div>,
}))

describe('Dialog components', () => {
  it('renders Dialog root', () => {
    render(<Dialog>Content</Dialog>)
    expect(screen.getByText('Content')).toBeDefined()
  })

  it('renders DialogTrigger', () => {
    render(<DialogTrigger>Open</DialogTrigger>)
    expect(screen.getByText('Open')).toBeDefined()
  })

  it('renders DialogContent with title', () => {
    render(<DialogContent title="Test Title">Content</DialogContent>)
    expect(screen.getByText('Test Title')).toBeDefined()
  })

  it('renders DialogContent with description', () => {
    render(
      <DialogContent description="Test description">Content</DialogContent>
    )
    expect(screen.getByText('Test description')).toBeDefined()
  })

  it('renders close button by default', () => {
    render(<DialogContent>Content</DialogContent>)
    expect(screen.getByTestId('x-icon')).toBeDefined()
  })

  it('hides close button when showCloseButton is false', () => {
    render(<DialogContent showCloseButton={false}>Content</DialogContent>)
    expect(screen.queryByTestId('x-icon')).toBeNull()
  })

  it('renders DialogHeader', () => {
    render(<DialogHeader>Header</DialogHeader>)
    expect(screen.getByText('Header')).toBeDefined()
  })

  it('renders DialogFooter', () => {
    render(<DialogFooter>Footer</DialogFooter>)
    expect(screen.getByText('Footer')).toBeDefined()
  })

  it('renders DialogTitle', () => {
    render(<DialogTitle>Title</DialogTitle>)
    expect(screen.getByText('Title')).toBeDefined()
  })

  it('renders DialogDescription', () => {
    render(<DialogDescription>Description</DialogDescription>)
    expect(screen.getByText('Description')).toBeDefined()
  })
})
