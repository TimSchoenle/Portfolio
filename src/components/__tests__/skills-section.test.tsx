import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import { SkillsSection } from '../skills-section'

// Mock next-intl
vi.mock('next-intl/server', () => ({
  getTranslations: vi.fn(async () => (key: string) => key),
}))

// Mock config
vi.mock('@/lib/config', () => ({
  siteConfig: {
    skills: {
      expertise: ['TypeScript', 'React', 'Next.js'],
      learning: ['Rust', 'Go'],
      tools: ['Git', 'Docker', 'VS Code'],
    },
  },
}))

describe('SkillsSection', () => {
  it('renders section with title', async () => {
    const Component = await SkillsSection({ locale: 'en' })
    render(Component)

    expect(screen.getByText('title')).toBeDefined()
  })

  it('renders expertise skills', async () => {
    const Component = await SkillsSection({ locale: 'en' })
    render(Component)

    expect(screen.getByText('expertise')).toBeDefined()
    expect(screen.getByText('TypeScript')).toBeDefined()
    expect(screen.getByText('React')).toBeDefined()
    expect(screen.getByText('Next.js')).toBeDefined()
  })

  it('renders learning skills', async () => {
    const Component = await SkillsSection({ locale: 'en' })
    render(Component)

    expect(screen.getByText('learning')).toBeDefined()
    expect(screen.getByText('Rust')).toBeDefined()
    expect(screen.getByText('Go')).toBeDefined()
  })

  it('renders tools', async () => {
    const Component = await SkillsSection({ locale: 'en' })
    render(Component)

    expect(screen.getByText('tools')).toBeDefined()
    expect(screen.getByText('Git')).toBeDefined()
    expect(screen.getByText('Docker')).toBeDefined()
    expect(screen.getByText('VS Code')).toBeDefined()
  })
})
