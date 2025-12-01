import { describe, expect, it, vi } from 'vitest'

import { SkillsSection } from '@/components/resume/templates/modern/skills-section'
import type { ResumeTranslations } from '@/types/resume'

vi.mock('@/lib/config', () => ({
  siteConfig: {
    skills: {
      languages: [
        { name: 'JavaScript', confidence: 0.9 },
        { name: 'TypeScript', confidence: 0.85 },
      ],
      frameworks: [
        { name: 'React', confidence: 0.8 },
        { name: 'Next.js', confidence: 0.8 },
      ],
      infrastructure: [
        { name: 'Docker', confidence: 0.7 },
        { name: 'Linux', confidence: 0.8 },
      ],
      buildTools: [
        { name: 'Git', confidence: 0.85 },
        { name: 'VS Code', confidence: 0.9 },
      ],
      sectionSideMinimumConfidence: 0,
      resumeMinimumConfidence: 0,
    },
  },
  SKILL_RENDER_AREAS: {
    RESUME: 'resume',
  },
}))

describe('SkillsSection', () => {
  const mockTranslations: ResumeTranslations = vi.fn((key: string) => {
    const translations: Record<string, string> = {
      'resume.sectionTitles.skills': 'Technical Skills',
      'resume.sectionTitles.skillsSubTypes.languages': 'Languages',
      'resume.sectionTitles.skillsSubTypes.frameworks': 'Frameworks',
      'resume.sectionTitles.skillsSubTypes.infrastructure': 'Infrastructure',
      'resume.sectionTitles.skillsSubTypes.buildTools': 'Build Tools',
    }
    return translations[key] ?? key
  }) as unknown as ResumeTranslations

  it('can be imported', async () => {
    const module =
      await import('@/components/resume/templates/modern/skills-section')
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

  it('renders languages', () => {
    const result = SkillsSection({
      translations: mockTranslations,
    })

    const resultString = JSON.stringify(result)
    expect(resultString.includes('Languages')).toBe(true)
    expect(resultString.includes('JavaScript')).toBe(true)
    expect(resultString.includes('TypeScript')).toBe(true)
  })

  it('renders frameworks', () => {
    const result = SkillsSection({
      translations: mockTranslations,
    })

    const resultString = JSON.stringify(result)
    expect(resultString.includes('Frameworks')).toBe(true)
    expect(resultString.includes('React')).toBe(true)
    expect(resultString.includes('Next.js')).toBe(true)
  })

  it('renders infrastructure', () => {
    const result = SkillsSection({
      translations: mockTranslations,
    })

    const resultString = JSON.stringify(result)
    expect(resultString.includes('Infrastructure')).toBe(true)
    expect(resultString.includes('Docker')).toBe(true)
    expect(resultString.includes('Linux')).toBe(true)
  })

  it('renders build tools', () => {
    const result = SkillsSection({
      translations: mockTranslations,
    })

    const resultString = JSON.stringify(result)
    expect(resultString.includes('Build Tools')).toBe(true)
    expect(resultString.includes('Git')).toBe(true)
    expect(resultString.includes('VS Code')).toBe(true)
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
    expect(resultString.includes('React')).toBe(true)
    expect(resultString.includes('Docker')).toBe(true)
  })
})
