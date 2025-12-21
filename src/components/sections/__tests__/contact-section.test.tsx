import { readFile } from 'node:fs/promises'

import { render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

import { ContactSection } from '../contact-section'

// Mock node:fs/promises
// Mock node:fs/promises
vi.mock('node:fs/promises', () => {
  const readFile = vi.fn()
  return {
    __esModule: true,
    readFile,
    default: { readFile },
  }
})

// Mock next-intl/server
vi.mock('next-intl/server', () => ({
  getTranslations: vi.fn().mockImplementation(async () => (key: string) => key),
}))

// Mock lucide-react
vi.mock('lucide-react', () => ({
  Send: () => null,
  Mail: () => null,
  Github: () => null,
  Linkedin: () => null,
  Eye: () => null,
  FileText: () => null,
  GitBranch: () => null,
  MapPin: () => null,
  ShieldCheck: () => <div data-testid="shield-check" />,
}))

// Mock ResumeVerificationDialog (since it's a client component used in server component)
vi.mock('@/components/resume/resume-verification-dialog', () => ({
  ResumeVerificationDialog: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="verification-dialog">{children}</div>
  ),
}))

// Mock Blueprint components to avoid nested complexity
vi.mock('@/components/blueprint/blueprint-card', () => ({
  BlueprintCard: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="blueprint-card">{children}</div>
  ),
}))
vi.mock('@/components/blueprint/blueprint-container', () => ({
  BlueprintContainer: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="blueprint-container">{children}</div>
  ),
}))

describe('ContactSection', () => {
  it('renders verify button when fingerprint exists', async () => {
    // Mock successful file read with fingerprint
    vi.mocked(readFile).mockResolvedValue(
      JSON.stringify({ fingerprint: 'VALID:FINGERPRINT' })
    )

    const Component = await ContactSection({ locale: 'en' })
    render(Component)

    // Check for verification dialog wrapper and shield icon
    expect(screen.getByTestId('verification-dialog')).toBeInTheDocument()
    expect(screen.getByTestId('shield-check')).toBeInTheDocument()
  })

  it('hides verify button when fingerprint file is missing/invalid', async () => {
    // Mock file read failure
    vi.mocked(readFile).mockRejectedValue(new Error('File not found'))

    const Component = await ContactSection({ locale: 'en' })
    render(Component)

    // Should NOT have verification dialog
    expect(screen.queryByTestId('verification-dialog')).not.toBeInTheDocument()
    expect(screen.queryByTestId('shield-check')).not.toBeInTheDocument()
  })

  it('renders contact links', async () => {
    // Mock file read failure (default case for links check)
    vi.mocked(readFile).mockRejectedValue(new Error('File not found'))

    const Component = await ContactSection({ locale: 'en' })
    render(Component)

    const githubLink = screen.getByRole('link', { name: /github/i })
    expect(githubLink).toBeDefined()
    expect(githubLink.getAttribute('href')).toContain('github.com')
  })
})
