import { describe, expect, it, vi } from 'vitest'

import type { Messages } from 'next-intl'
import React from 'react'

// Mock @react-pdf/renderer before importing component
vi.mock('@react-pdf/renderer', () => ({
  Document: vi.fn(
    ({ children }: { readonly children: React.ReactNode }) => children
  ),
  Link: vi.fn(
    ({ children }: { readonly children: React.ReactNode }) => children
  ),
  Page: vi.fn(
    ({ children }: { readonly children: React.ReactNode }) => children
  ),
  StyleSheet: {
    create: vi.fn((styles: Record<string, unknown>) => styles),
  },
  Text: vi.fn(
    ({ children }: { readonly children: React.ReactNode }) => children
  ),
  View: vi.fn(
    ({ children }: { readonly children: React.ReactNode }) => children
  ),
}))

describe('ResumePDFDocument', () => {
  it('can be imported', async () => {
    const module = await import('@/components/resume/resume-pdf-document')
    expect(module.ResumePDFDocument).toBeDefined()
  })

  it('is a function', async () => {
    const { ResumePDFDocument } =
      await import('@/components/resume/resume-pdf-document')
    expect(typeof ResumePDFDocument).toBe('function')
  })

  const mockTranslations = {
    contact: 'Contact',
    education: 'Education',
    email: 'Email',
    experience: 'Experience',
    github: 'GitHub',
    linkedin: 'LinkedIn',
    location: 'Location',
    projects: 'Projects',
    skills: 'Skills',
    summary: 'Summary',
  } as unknown as Messages['resume']['sectionTitles']

  it('accepts resume data as a prop', async () => {
    const { ResumePDFDocument } =
      await import('@/components/resume/resume-pdf-document')
    expect(() => {
      ResumePDFDocument({
        formatDate: {
          dateTime: (date: Date | number) => date.toString(),
        } as any,
        translations: mockTranslations,
      })
    }).not.toThrow()
  })
})
