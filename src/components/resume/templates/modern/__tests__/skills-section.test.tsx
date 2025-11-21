import { describe, expect, it, vi } from 'vitest'

import { SkillsSection } from '@/components/resume/templates/modern/skills-section'
import type { ResumeTranslations } from '@/types/resume'

// Mock site config
vi.mock('@/lib/config', () => ({
  siteConfig: {
    skills: {
      expertise: ['JavaScript', 'TypeScript', 'React'],
      learning: ['Rust', 'Go'],
      tools: ['Git', 'Docker', 'VS Code'],
    },
  },
}))

describe('SkillsSection', () => {
  const mockTranslations: ResumeTranslations = vi.fn((key: string) => {
    const translations: Record<string, string> = {
      'resume.sectionTitles.skills': 'Technical Skills',
      'resume.sectionTitles.skillsSubTypes.expertise': 'Expertise',
      'resume.sectionTitles.skillsSubTypes.learning': 'Currently Learning',
      'resume.sectionTitles.skillsSubTypes.tools': 'Tools & Platforms',
    }
    return translations[key] ?? key
  }) as unknown as ResumeTranslations

  it('can be imported', async () => {
    const module = await import(
      '@/components/resume/templates/modern/skills-section'
    )
    expect(module.SkillsSection).toBeDefined()
  })

  it('renders without errors', () => {
    expect(() => {
      SkillsSection({ translations: mockTranslations })
    }).not.toThrow()
  })

  it('displays section title', () => {
    const result = SkillsSection({
      translations: mockTranslations,
    })

    const resultString = JSON.stringify(result)
    expect(resultString.includes('Technical Skills')).toBe(true)
  })

  it('renders all expertise skills', () => {
    const result = SkillsSection({
      translations: mockTranslations,
    })

    const resultString = JSON.stringify(result)
    expect(resultString.includes('Expertise')).toBe(true)
    expect(resultString.includes('JavaScript')).toBe(true)
    expect(resultString.includes('TypeScript')).toBe(true)
    expect(resultString.includes('React')).toBe(true)
  })

  it('renders all learning skills', () => {
    const result = SkillsSection({
      translations: mockTranslations,
    })

    const resultString = JSON.stringify(result)
    expect(resultString.includes('Currently Learning')).toBe(true)
    expect(resultString.includes('Rust')).toBe(true)
    expect(resultString.includes('Go')).toBe(true)
  })

  it('renders all tools', () => {
    const result = SkillsSection({
      translations: mockTranslations,
    })

    const resultString = JSON.stringify(result)
    expect(resultString.includes('Tools & Platforms')).toBe(true)
    expect(resultString.includes('Git')).toBe(true)
    expect(resultString.includes('Docker')).toBe(true)
  })

  it('renders section divider', () => {
    const result = SkillsSection({
      translations: mockTranslations,
    })

    const resultString = JSON.stringify(result)
    // Check for style object which implies View components are present
    expect(resultString.includes('"style":')).toBe(true)
  })

  it('renders skill tags in containers', () => {
    const result = SkillsSection({
      translations: mockTranslations,
    })

    // Convert to string to verify skills are rendered
    const resultString = JSON.stringify(result)

    // Check for some known skills
    expect(resultString.includes('JavaScript')).toBe(true)
    expect(resultString.includes('Rust')).toBe(true)
    expect(resultString.includes('Docker')).toBe(true)
  })
})
