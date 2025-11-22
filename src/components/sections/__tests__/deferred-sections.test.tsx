import { describe, it, expect, vi } from 'vitest'

// Mock server-only to allow importing
vi.mock('server-only', () => ({}))

// Mock all section components BEFORE importing DeferredSections
vi.mock('../about-section', () => ({
  default: () => null,
}))

vi.mock('../skills-section', () => ({
  SkillsSection: () => null,
}))

vi.mock('../projects-section', () => ({
  ProjectsSection: () => null,
}))

vi.mock('../experience-section', () => ({
  ExperienceSection: () => null,
}))

vi.mock('../testimonials-section', () => ({
  TestimonialsSection: () => null,
}))

vi.mock('../contact-section', () => ({
  ContactSection: () => null,
}))

vi.mock('@/lib/config', () => ({
  siteConfig: {
    githubUsername: 'testuser',
  },
}))

describe('DeferredSections', () => {
  it('can be imported', async () => {
    const { DeferredSections } = await import('../deferred-sections')
    expect(DeferredSections).toBeDefined()
  })

  it('is a function', async () => {
    const { DeferredSections } = await import('../deferred-sections')
    expect(typeof DeferredSections).toBe('function')
  })
})
