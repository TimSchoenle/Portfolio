import { describe, expect, it, vi } from 'vitest'

import { EducationSection } from '@/components/resume/templates/modern/education-section'
import type { ResumeEducation, ResumeTranslations } from '@/types/resume'

describe('EducationSection', () => {
  const mockEducation: ResumeEducation[] = [
    {
      degree: 'B.S. Computer Science',
      institution: 'University of California',
      year: '2019',
    },
    {
      degree: 'M.S. Software Engineering',
      institution: 'Stanford University',
      year: '2021',
    },
  ]

  const mockTranslations: ResumeTranslations = Object.assign(
    vi.fn((key: string) => {
      const translations: Record<string, string> = {
        'resume.sectionTitles.education': 'Education',
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

  it('can be imported', async () => {
    const module = await import(
      '@/components/resume/templates/modern/education-section'
    )
    expect(module.EducationSection).toBeDefined()
  })

  it('renders without errors', () => {
    expect(() => {
      EducationSection({ translations: mockTranslations })
    }).not.toThrow()
  })

  it('displays section title', () => {
    const result = EducationSection({
      translations: mockTranslations,
    })

    const resultString = JSON.stringify(result)
    expect(resultString.includes('Education')).toBe(true)
  })

  it('renders all education entries', () => {
    const result = EducationSection({
      translations: mockTranslations,
    })

    const resultString = JSON.stringify(result)
    expect(resultString.includes('B.S. Computer Science')).toBe(true)
    expect(resultString.includes('M.S. Software Engineering')).toBe(true)
  })

  it('handles empty education array', () => {
    const emptyTranslations: ResumeTranslations = Object.assign(
      vi.fn(() => 'Education'),
      {
        raw: vi.fn(() => []),
      }
    ) as unknown as ResumeTranslations

    expect(() => {
      EducationSection({ translations: emptyTranslations })
    }).not.toThrow()

    const result = EducationSection({
      translations: emptyTranslations,
    })
    expect(result).toBeTruthy()
  })

  it('renders section divider', () => {
    const result = EducationSection({
      translations: mockTranslations,
    })

    const resultString = JSON.stringify(result)
    // Check for style object which implies View components are present
    expect(resultString.includes('"style":')).toBe(true)
  })

  it('displays degree, institution, and year for each entry', () => {
    const result = EducationSection({
      translations: mockTranslations,
    })

    // Convert to string to verify education data is rendered
    const resultString = JSON.stringify(result)

    // Check that education data is present
    expect(resultString.includes('B.S. Computer Science')).toBe(true)
    expect(resultString.includes('University of California')).toBe(true)
    expect(resultString.includes('2019')).toBe(true)
  })
})
