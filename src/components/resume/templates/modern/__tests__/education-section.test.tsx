import { describe, expect, it, vi } from 'vitest'

import { EducationSection } from '@/components/resume/templates/modern/education-section'
import type { ResumeEducation, ResumeTranslations } from '@/types/resume'

describe('EducationSection', () => {
  const mockEducation: ResumeEducation[] = [
    {
      degree: 'B.S. Computer Science',
      end: { month: 5, year: 2019 },
      institution: 'University of California',
      start: { month: 9, year: 2015 },
    },
    {
      degree: 'M.S. Software Developering',
      end: null,
      institution: 'Stanford University',
      start: { month: 9, year: 2020 },
    },
  ]

  const mockTranslations: ResumeTranslations = Object.assign(
    vi.fn((key: string) => {
      const translations: Record<string, string> = {
        'resume.sectionTitles.education': 'Education',
        'resume.present': 'Present',
      }
      return translations[key] ?? key
    }),
    {
      raw: vi.fn((key: string) => {
        if (key === 'resume.education') {
          return mockEducation
        }
        return null
      }),
    }
  ) as unknown as ResumeTranslations

  const mockFormatter = {
    dateTime: vi.fn((date, _options) => {
      if (date instanceof Date) {
        return date.getUTCFullYear().toString()
      }
      return 'Date'
    }),
  } as any

  it('can be imported', async () => {
    const module =
      await import('@/components/resume/templates/modern/education-section')
    expect(module.EducationSection).toBeDefined()
  })

  it('renders without errors', () => {
    expect(() => {
      EducationSection({
        formatDate: mockFormatter,
        translations: mockTranslations,
      })
    }).not.toThrow()
  })

  it('displays section title', () => {
    const result = EducationSection({
      formatDate: mockFormatter,
      translations: mockTranslations,
    })

    const resultString = JSON.stringify(result)
    expect(resultString.includes('Education')).toBe(true)
  })

  it('renders all education entries', () => {
    const result = EducationSection({
      formatDate: mockFormatter,
      translations: mockTranslations,
    })

    const resultString = JSON.stringify(result)
    expect(resultString.includes('B.S. Computer Science')).toBe(true)
    expect(resultString.includes('M.S. Software Developering')).toBe(true)
  })

  it('handles empty education array', () => {
    const emptyTranslations: ResumeTranslations = Object.assign(
      vi.fn(() => 'Education'),
      {
        raw: vi.fn(() => []),
      }
    ) as unknown as ResumeTranslations

    expect(() => {
      EducationSection({
        formatDate: mockFormatter,
        translations: emptyTranslations,
      })
    }).not.toThrow()

    const result = EducationSection({
      formatDate: mockFormatter,
      translations: emptyTranslations,
    })
    expect(result).toBeTruthy()
  })

  it('renders section divider', () => {
    const result = EducationSection({
      formatDate: mockFormatter,
      translations: mockTranslations,
    })

    const resultString = JSON.stringify(result)
    // Check for style object which implies View components are present
    expect(resultString.includes('"style":')).toBe(true)
  })

  it('displays degree, institution, and dates for each entry', () => {
    const result = EducationSection({
      formatDate: mockFormatter,
      translations: mockTranslations,
    })

    // Convert to string to verify education data is rendered
    const resultString = JSON.stringify(result)

    // Check that education data is present
    expect(resultString.includes('B.S. Computer Science')).toBe(true)
    expect(resultString.includes('University of California')).toBe(true)
    // 2019 is the graduation year in mock data
    expect(resultString.includes('2019')).toBe(true)
  })
})
