import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

import { ResumeVerificationDialog } from '../resume-verification-dialog'

// Mock next-intl
vi.mock('next-intl', () => ({
  useTranslations:
    () =>
    (key: string): string =>
      key,
}))

// Mock lucide-react icons
vi.mock('lucide-react', () => ({
  ShieldCheck: () => <div data-testid="shield-check" />,
  Check: () => <div data-testid="check" />,
  Copy: () => <div data-testid="copy" />,
  Download: () => <div data-testid="download" />,
  X: () => <div data-testid="close" />,
  XIcon: () => <div data-testid="close" />,
}))

// Mock clipboard API
const mockWriteText = vi.fn()
Object.defineProperty(navigator, 'clipboard', {
  value: {
    writeText: mockWriteText,
  },
  writable: true,
})

describe('ResumeVerificationDialog', () => {
  const defaultProps = {
    children: <button>Open Dialog</button>,
    fingerprint: 'AB:CD:EF:12:34:56',
  }

  it('renders the trigger child', () => {
    render(<ResumeVerificationDialog {...defaultProps} />)
    expect(
      screen.getByRole('button', { name: /open dialog/i })
    ).toBeInTheDocument()
  })

  it('opens the dialog and displays fingerprint', async () => {
    render(<ResumeVerificationDialog {...defaultProps} />)

    // Open dialog
    fireEvent.click(screen.getByRole('button', { name: /open dialog/i }))

    // Check title and fingerprint
    expect(screen.getByText('title')).toBeInTheDocument()
    expect(
      screen.getByRole('textbox', { name: 'fingerprintLabel' })
    ).toHaveValue(defaultProps.fingerprint)
  })

  it('copies fingerprint to clipboard', async () => {
    mockWriteText.mockResolvedValue(undefined)
    render(<ResumeVerificationDialog {...defaultProps} />)

    // Open dialog
    fireEvent.click(screen.getByRole('button', { name: /open dialog/i }))

    // Click copy button
    fireEvent.click(screen.getByRole('button', { name: 'copyFingerprint' }))

    await waitFor(() => {
      expect(mockWriteText).toHaveBeenCalledWith(defaultProps.fingerprint)
    })

    // Check for success state (Check icon)
    await waitFor(() => {
      expect(screen.getByTestId('check')).toBeInTheDocument()
    })
  })

  it('renders download button with correct link', async () => {
    render(<ResumeVerificationDialog {...defaultProps} />)

    // Open dialog
    fireEvent.click(screen.getByRole('button', { name: /open dialog/i }))

    const downloadLink = screen.getByRole('link', {
      name: 'downloadCertificate',
    })
    expect(downloadLink).toHaveAttribute('href', '/resume/certificate.crt')
    expect(downloadLink).toHaveAttribute('download', 'certificate.crt')
  })
})
