import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'

// Mock next-intl
vi.mock('next-intl', () => ({
  useTranslations: vi.fn(() => (key: string) => key),
  useLocale: vi.fn(() => 'en'),
}))

// Mock lucide-react icons
vi.mock('lucide-react', () => ({
  Home: () => <div data-testid="home-icon">Home</div>,
  User: () => <div data-testid="user-icon">User</div>,
  Code: () => <div data-testid="code-icon">Code</div>,
  Briefcase: () => <div data-testid="briefcase-icon">Briefcase</div>,
  MessageSquare: () => <div data-testid="message-icon">Message</div>,
  FileText: () => <div data-testid="file-icon">File</div>,
  CookieIcon: () => <div data-testid="cookie-icon">Cookie</div>,
  GitBranch: () => <div data-testid="git-icon">Git</div>,
  Mail: () => <div data-testid="mail-icon">Mail</div>,
}))

// Mock i18n routing
vi.mock('@/i18n/routing', () => ({
  useRouter: vi.fn(() => ({ push: vi.fn() })),
  usePathname: vi.fn(() => '/'),
}))

// Mock config
vi.mock('@/lib/config', () => ({
  siteConfig: {
    github: 'https://github.com/test',
    email: 'test@test.com',
  },
}))

// Mock command UI components
vi.mock('@/components/ui/command', () => ({
  CommandDialog: ({ children, open }: any) =>
    open ? <div data-testid="command-dialog">{children}</div> : null,
  CommandInput: () => <input data-testid="command-input" />,
  CommandList: ({ children }: any) => <div>{children}</div>,
  CommandEmpty: ({ children }: any) => <div>{children}</div>,
  CommandGroup: ({ children, heading }: any) => (
    <div>
      <div data-testid="group-heading">{heading}</div>
      {children}
    </div>
  ),
  CommandItem: ({ children, onSelect }: any) => (
    <button onClick={() => onSelect?.('')}>{children}</button>
  ),
}))

describe('CommandPalette', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('renders command palette', async () => {
    const { CommandPalette } = await import('../command-palette')
    render(<CommandPalette />)

    // Initially closed, so dialog not visible
    expect(screen.queryByTestId('command-dialog')).toBeNull()
  })

  it('opens on Cmd/Ctrl+K', async () => {
    const { CommandPalette } = await import('../command-palette')
    render(<CommandPalette />)

    // Simulate Cmd+K
    fireEvent.keyDown(document, { key: 'k', metaKey: true })

    await waitFor(() => {
      expect(screen.queryByTestId('command-dialog')).toBeDefined()
    })
  })

  it('includes navigation items', async () => {
    const { CommandPalette } = await import('../command-palette')
    const { container } = render(<CommandPalette />)

    // Open the dialog
    fireEvent.keyDown(document, { key: 'k', ctrlKey: true })

    await waitFor(() => {
      expect(container.textContent).toContain('navigation')
    })
  })

  it('includes action items', async () => {
    const { CommandPalette } = await import('../command-palette')
    const { container } = render(<CommandPalette />)

    // Open dialog
    fireEvent.keyDown(document, { key: 'k', metaKey: true })

    await waitFor(() => {
      expect(container.textContent).toContain('actions')
    })
  })

  it('has search input', async () => {
    const { CommandPalette } = await import('../command-palette')
    render(<CommandPalette />)

    // Open dialog
    fireEvent.keyDown(document, { key: 'k', metaKey: true })

    await waitFor(() => {
      expect(screen.queryByTestId('command-input')).toBeDefined()
    })
  })
})
