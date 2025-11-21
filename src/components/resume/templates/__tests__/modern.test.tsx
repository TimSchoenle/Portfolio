import { Page } from '@react-pdf/renderer'
import { describe, expect, it, vi } from 'vitest'

import { ModernTemplate } from '@/components/resume/templates/modern'
import type { ResumeTranslations } from '@/types/resume'

// Mock siteConfig
vi.mock('@/lib/config', () => ({
  siteConfig: {
    email: 'test@example.com',
    fullName: 'John Doe',
    github: 'https://github.com/johndoe',
    linkedin: 'https://linkedin.com/in/johndoe',
    name: 'John Doe',
    skills: {
      expertise: ['JavaScript', 'TypeScript'],
      learning: ['Rust'],
      tools: ['Git', 'Docker'],
    },
  },
}))

describe('ModernTemplate', () => {
  const mockTranslations: ResumeTranslations = Object.assign(
    vi.fn((key: string) => {
      const translations: Record<string, string> = {
        'personalInfo.jobTitle': 'Senior Software Engineer',
        'personalInfo.country': 'Germany',
        'resume.summary': 'Experienced developer with 5+ years',
        'resume.sectionTitles.experience': 'Experience',
        'resume.sectionTitles.projects': 'Projects',
        'resume.sectionTitles.education': 'Education',
        'resume.sectionTitles.skills': 'Technical Skills',
        'resume.sectionTitles.skillsSubTypes.expertise': 'Expertise',
        'resume.sectionTitles.skillsSubTypes.learning': 'Currently Learning',
        'resume.sectionTitles.skillsSubTypes.tools': 'Tools & Platforms',
        'contact.title': 'Contact',
        'contact.email': 'Email',
        'contact.location': 'Location',
        'common.socials.github': 'GitHub',
        'common.socials.linkedin': 'LinkedIn',
      }
      return translations[key] ?? key
    }),
    {
      raw: vi.fn((key: string) => {
        const data: Record<string, unknown> = {
          'resume.education': [
            {
              degree: 'B.S. Computer Science',
              institution: 'University',
              year: '2019',
            },
          ],
          'resume.experience': [
            {
              achievements: ['Built features', 'Improved performance'],
              company: 'Tech Corp',
              endDate: 'Present',
              location: 'SF',
              startDate: '2020',
              title: 'Developer',
            },
          ],
          'resume.projects': [
            {
              description: 'Cool project',
              name: 'Project Alpha',
              technologies: ['React', 'Node.js'],
            },
          ],
        }
        return data[key] ?? null
      }),
    }
  ) as unknown as ResumeTranslations

  it('can be imported', async () => {
    const module = await import('@/components/resume/templates/modern')
    expect(module.ModernTemplate).toBeDefined()
  })

  it('renders without errors', () => {
    expect(() => {
      ModernTemplate({ translations: mockTranslations })
    }).not.toThrow()
  })

  it('returns a Page component', () => {
    const result = ModernTemplate({
      translations: mockTranslations,
    })

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    expect((result as any).type).toBe(Page)
  })

  it('displays name from siteConfig', () => {
    const result = ModernTemplate({
      translations: mockTranslations,
    })

    const resultString = JSON.stringify(result)
    expect(resultString.includes('John Doe')).toBe(true)
  })

  it('displays job title from translations', () => {
    const result = ModernTemplate({
      translations: mockTranslations,
    })

    const resultString = JSON.stringify(result)
    expect(resultString.includes('Senior Software Engineer')).toBe(true)
  })

  it('displays summary from translations', () => {
    const result = ModernTemplate({
      translations: mockTranslations,
    })

    const resultString = JSON.stringify(result)
    expect(resultString.includes('Experienced developer with 5+ years')).toBe(
      true
    )
  })
})
