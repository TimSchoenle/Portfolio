import { describe, expect, it, vi } from 'vitest'

import type { ResumeData } from '@/types/resume-types'
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
  const mockResumeData: ResumeData = {
    education: [
      {
        degree: 'B.S. Computer Science',
        institution: 'University of California',
        year: '2016',
      },
    ],
    experience: [
      {
        achievements: ['Built web apps', 'Improved performance'],
        company: 'Tech Corp',
        endDate: 'Present',
        location: 'San Francisco, CA',
        startDate: '2020',
        title: 'Senior Developer',
      },
    ],
    personalInfo: {
      email: 'john@example.com',
      github: 'https://github.com/johndoe',
      linkedin: 'https://linkedin.com/in/johndoe',
      location: 'San Francisco, CA',
      name: 'John Doe',
      title: 'Senior Software Engineer',
    },
    projects: [
      {
        description: 'A cool project',
        name: 'Project Alpha',
        technologies: ['React', 'Node.js'],
        url: 'https://github.com/johndoe/project-alpha',
      },
    ],
    skills: {
      expertise: ['JavaScript', 'TypeScript', 'React'],
      learning: ['Rust', 'Go'],
      tools: ['Git', 'Docker'],
    },
    summary: 'Experienced software engineer',
  }

  it('can be imported', async () => {
    const module = await import('@/components/resume/resume-pdf-document')
    expect(module.ResumePDFDocument).toBeDefined()
  })

  it('is a function', async () => {
    const { ResumePDFDocument } = await import(
      '@/components/resume/resume-pdf-document'
    )
    expect(typeof ResumePDFDocument).toBe('function')
  })

  it('accepts resume data as a prop', async () => {
    const { ResumePDFDocument } = await import(
      '@/components/resume/resume-pdf-document'
    )
    expect(() => {
      ResumePDFDocument({ data: mockResumeData })
    }).not.toThrow()
  })
})
