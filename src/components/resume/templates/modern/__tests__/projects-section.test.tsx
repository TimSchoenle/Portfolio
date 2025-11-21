import { describe, expect, it, vi } from 'vitest'

import { ProjectsSection } from '@/components/resume/templates/modern/projects-section'
import type { ResumeProject, ResumeTranslations } from '@/types/resume'

describe('ProjectsSection', () => {
  const mockProjects: ResumeProject[] = [
    {
      description: 'Full-stack e-commerce platform',
      name: 'E-Commerce Platform',
      technologies: ['React', 'Node.js', 'MongoDB'],
      url: 'https://github.com/user/ecommerce',
    },
    {
      description: 'Task management application',
      name: 'Task Manager',
      technologies: ['Next.js', 'TypeScript', 'PostgreSQL'],
    },
  ]

  const mockTranslations: ResumeTranslations = Object.assign(
    vi.fn((key: string) => {
      const translations: Record<string, string> = {
        'resume.sectionTitles.projects': 'Projects',
      }
      return translations[key] ?? key
    }),
    {
      raw: vi.fn((key: string) => {
        if (key === 'resume.projects') {
          return mockProjects
        }
        return null
      }),
    }
  ) as unknown as ResumeTranslations

  it('can be imported', async () => {
    const module = await import(
      '@/components/resume/templates/modern/projects-section'
    )
    expect(module.ProjectsSection).toBeDefined()
  })

  it('renders without errors', () => {
    expect(() => {
      ProjectsSection({ translations: mockTranslations })
    }).not.toThrow()
  })

  it('displays section title', () => {
    const result = ProjectsSection({
      translations: mockTranslations,
    })

    const resultString = JSON.stringify(result)
    expect(resultString.includes('Projects')).toBe(true)
  })

  it('renders all project entries', () => {
    const result = ProjectsSection({
      translations: mockTranslations,
    })

    const resultString = JSON.stringify(result)
    expect(resultString.includes('E-Commerce Platform')).toBe(true)
    expect(resultString.includes('Task Manager')).toBe(true)
  })

  it('calls translations.raw with correct key', () => {
    ProjectsSection({ translations: mockTranslations })

    expect(mockTranslations.raw).toHaveBeenCalledWith('resume.projects')
  })

  it('displays project name and description', () => {
    const result = ProjectsSection({
      translations: mockTranslations,
    })

    const resultString = JSON.stringify(result)
    expect(resultString.includes('Full-stack e-commerce platform')).toBe(true)
    expect(resultString.includes('Task management application')).toBe(true)
  })

  it('renders technologies joined with separator', () => {
    const result = ProjectsSection({
      translations: mockTranslations,
    })

    // Convert result to string to check for separator
    const resultString = JSON.stringify(result)

    // Check that technologies are rendered with bullet separator
    expect(resultString.includes('•')).toBe(true)
    // Also check that at least one technology is present
    expect(
      resultString.includes('React') || resultString.includes('Node.js')
    ).toBe(true)
  })

  it('handles optional URL field', () => {
    const projectsWithoutUrl: ResumeProject[] = [
      {
        description: 'Test project',
        name: 'Test',
        technologies: ['React'],
      },
    ]

    const customTranslations: ResumeTranslations = Object.assign(
      vi.fn(() => 'Projects'),
      {
        raw: vi.fn(() => projectsWithoutUrl),
      }
    ) as unknown as ResumeTranslations

    expect(() => {
      ProjectsSection({ translations: customTranslations })
    }).not.toThrow()
  })

  it('handles empty projects array', () => {
    const emptyTranslations: ResumeTranslations = Object.assign(
      vi.fn(() => 'Projects'),
      {
        raw: vi.fn(() => []),
      }
    ) as unknown as ResumeTranslations

    expect(() => {
      ProjectsSection({ translations: emptyTranslations })
    }).not.toThrow()

    const result = ProjectsSection({
      translations: emptyTranslations,
    })
    expect(result).toBeTruthy()
  })

  it('renders section divider', () => {
    const result = ProjectsSection({
      translations: mockTranslations,
    })

    const resultString = JSON.stringify(result)
    // Check for style object which implies View components are present
    expect(resultString.includes('"style":')).toBe(true)
  })
})
