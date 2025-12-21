import { describe, expect, it, vi } from 'vitest'

import { ExperienceSection } from '@/components/resume/templates/modern/experience-section'
import type { ResumeExperience, ResumeTranslations } from '@/types/resume'

describe('ExperienceSection', () => {
  const mockExperience: ResumeExperience[] = [
    {
      achievements: [
        'Led development of microservices',
        'Reduced load time by 60%',
        'Mentored 5 junior developers',
      ],
      company: 'Tech Company Inc.',
      end: null,
      location: 'San Francisco, CA',
      start: {
        month: 1,
        year: 2021,
      },
      title: 'Senior Software Developer',
    },
    {
      achievements: ['Built real-time messaging system', 'Implemented CI/CD'],
      company: 'Startup LLC',
      end: {
        month: 12,
        year: 2021,
      },
      location: 'Remote',
      start: {
        month: 6,
        year: 2019,
      },
      title: 'Software Developer',
    },
  ]

  const mockTranslations: ResumeTranslations = Object.assign(
    vi.fn((key: string) => {
      const translations: Record<string, string> = {
        'resume.sectionTitles.experience': 'Experience',
      }
      return translations[key] ?? key
    }),
    {
      raw: vi.fn((key: string) => {
        if (key === 'resume.experience') {
          return mockExperience
        }
        return null
      }),
    }
  ) as unknown as ResumeTranslations

  it('can be imported', async () => {
    const module =
      await import('@/components/resume/templates/modern/experience-section')
    expect(module.ExperienceSection).toBeDefined()
  })

  it('renders without errors', () => {
    expect(() => {
      ExperienceSection({
        formatDate: {
          dateTime: (date: Date | number) => {
            if (date instanceof Date) {
              return date.getFullYear().toString()
            }
            return date.toString()
          },
        } as any,
        translations: mockTranslations,
      })
    }).not.toThrow()
  })

  it('displays section title', () => {
    const result = ExperienceSection({
      formatDate: {
        dateTime: (date: Date | number) => {
          if (date instanceof Date) {
            return date.getFullYear().toString()
          }
          return date.toString()
        },
      } as any,
      translations: mockTranslations,
    })

    const resultString = JSON.stringify(result)
    expect(resultString.includes('Experience')).toBe(true)
  })

  it('renders all experience entries', () => {
    const result = ExperienceSection({
      formatDate: {
        dateTime: (date: Date | number) => {
          if (date instanceof Date) {
            return date.getFullYear().toString()
          }
          return date.toString()
        },
      } as any,
      translations: mockTranslations,
    })

    // Convert to string to verify experience data
    const resultString = JSON.stringify(result)

    // Check that experience data is present
    expect(resultString.includes('Senior Software Developer')).toBe(true)
    expect(resultString.includes('Tech Company Inc.')).toBe(true)
  })

  it('calls translations.raw with correct key', () => {
    ExperienceSection({
      formatDate: {
        dateTime: (date: Date | number) => {
          if (date instanceof Date) {
            return date.getFullYear().toString()
          }
          return date.toString()
        },
      } as any,
      translations: mockTranslations,
    })

    expect(mockTranslations.raw).toHaveBeenCalledWith('resume.experience')
  })

  it('displays job title, company, and dates', () => {
    const result = ExperienceSection({
      formatDate: {
        dateTime: (date: Date | number) => {
          if (date instanceof Date) {
            return date.getFullYear().toString()
          }
          return date.toString()
        },
      } as any,
      translations: mockTranslations,
    })

    const resultString = JSON.stringify(result)
    expect(resultString.includes('Senior Software Developer')).toBe(true)
    expect(resultString.includes('Tech Company Inc.')).toBe(true)
    expect(resultString.includes('2021')).toBe(true)
  })

  it('renders achievements for each experience', () => {
    const result = ExperienceSection({
      formatDate: {
        dateTime: (date: Date | number) => {
          if (date instanceof Date) {
            return date.getFullYear().toString()
          }
          return date.toString()
        },
      } as any,
      translations: mockTranslations,
    })

    // Convert to string to verify achievements
    const resultString = JSON.stringify(result)

    // Check that achievements are present
    expect(resultString.includes('Led development of microservices')).toBe(true)
    expect(resultString.includes('Reduced load time by 60%')).toBe(true)
  })

  it('handles empty experience array', () => {
    const emptyTranslations: ResumeTranslations = Object.assign(
      vi.fn(() => 'Experience'),
      {
        raw: vi.fn(() => []),
      }
    ) as unknown as ResumeTranslations

    expect(() => {
      ExperienceSection({
        formatDate: {
          dateTime: (date: Date | number) => {
            if (date instanceof Date) {
              return date.getFullYear().toString()
            }
            return date.toString()
          },
        } as any,
        translations: emptyTranslations,
      })
    }).not.toThrow()

    const result = ExperienceSection({
      formatDate: {
        dateTime: (date: Date | number) => {
          if (date instanceof Date) {
            return date.getFullYear().toString()
          }
          return date.toString()
        },
      } as any,
      translations: emptyTranslations,
    })
    expect(result).toBeTruthy()
  })

  it('renders section divider', () => {
    const result = ExperienceSection({
      formatDate: {
        dateTime: (date: Date | number) => {
          if (date instanceof Date) {
            return date.getUTCFullYear().toString()
          }
          return date.toString()
        },
      } as any,
      translations: mockTranslations,
    })

    const resultString = JSON.stringify(result)
    // Check for style object which implies View components are present
    expect(resultString.includes('"style":')).toBe(true)
  })
})
