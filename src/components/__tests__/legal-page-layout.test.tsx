import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import { LegalPageLayout } from '../legal-page-layout'

// Mock BackToHome
vi.mock('../back-to-home', () => ({
  BackToHome: () => <div data-testid="back-to-home">Back</div>,
}))

describe('LegalPageLayout', () => {
  it('renders title', () => {
    render(
      <LegalPageLayout locale="en" title="Test Title">
        Content
      </LegalPageLayout>
    )

    expect(screen.getByText('Test Title')).toBeDefined()
  })

  it('renders children', () => {
    render(
      <LegalPageLayout locale="en" title="Title">
        Test Content
      </LegalPageLayout>
    )

    expect(screen.getByText('Test Content')).toBeDefined()
  })

  it('renders BackToHome component', () => {
    render(
      <LegalPageLayout locale="en" title="Title">
        Content
      </LegalPageLayout>
    )

    expect(screen.getByTestId('back-to-home')).toBeDefined()
  })
})
