import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import Page from '../page'

// Mock next/navigation
vi.mock('next/navigation', () => ({
  useRouter: vi.fn(),
  usePathname: vi.fn(),
  useSearchParams: vi.fn(),
  redirect: vi.fn(),
  notFound: vi.fn(),
}))

// Mock i18n routing
vi.mock('@/i18n/routing', () => ({
  Link: ({ children, ...props }: any) => <a {...props}>{children}</a>,
  getPathname: vi.fn(),
  routing: {
    locales: ['en', 'de'],
    defaultLocale: 'en',
  },
}))

// Mock next-intl
vi.mock('next-intl/server', () => ({
  setRequestLocale: vi.fn(),
  getTranslations: vi.fn(() => (key: string) => key),
}))

// Mock github data fetching
vi.mock('@/lib/github/client', () => ({
  getGithubUser: vi.fn().mockResolvedValue({
    contributionData: [],
    projects: [],
    stats: { stars: 0, forks: 0, repositories: 0 },
  }),
}))

// Mock config
vi.mock('@/lib/config', () => ({
  siteConfig: {
    email: 'test@example.com',
    github: 'https://github.com/test',
    githubUsername: 'testuser',
  },
}))

// Mock components to avoid deep rendering complexity in integration test
vi.mock('@/components/sections/hero-section', () => ({
  HeroSection: () => <div data-testid="hero-section">Hero</div>,
}))

vi.mock(
  '@/components/features/scroll-snap/scroll-snap-pair-controller',
  () => ({
    ScrollSnapPairController: () => null,
  })
)

// Mock all section components with correct export names
vi.mock('@/components/sections/about-section', () => ({
  default: () => <div data-testid="about-section">About</div>,
}))

vi.mock('@/components/sections/skills-section', () => ({
  SkillsSection: () => <div data-testid="skills-section">Skills</div>,
}))

vi.mock('@/components/sections/projects-section', () => ({
  ProjectsSection: () => <div data-testid="projects-section">Projects</div>,
}))

vi.mock('@/components/sections/experience-section', () => ({
  ExperienceSection: () => (
    <div data-testid="experience-section">Experience</div>
  ),
}))

vi.mock('@/components/sections/testimonials-section', () => ({
  TestimonialsSection: () => (
    <div data-testid="testimonials-section">Testimonials</div>
  ),
}))

vi.mock('@/components/sections/contact-section', () => ({
  ContactSection: () => <div data-testid="contact-section">Contact</div>,
}))

describe('Page', () => {
  it('renders all sections', async () => {
    const Component = await Page({
      params: Promise.resolve({ locale: 'en-US' }),
    })
    render(Component)

    expect(screen.getByTestId('hero-section')).toBeDefined()
    expect(screen.getByRole('main')).toBeDefined()

    // Check that main-section wrapper exists
    const mainSection = document.querySelector('#main-section')
    expect(mainSection).toBeDefined()

    // All section mocks should be rendered
    expect(screen.getByTestId('about-section')).toBeDefined()
    expect(screen.getByTestId('skills-section')).toBeDefined()
    expect(screen.getByTestId('projects-section')).toBeDefined()
    expect(screen.getByTestId('experience-section')).toBeDefined()
    expect(screen.getByTestId('testimonials-section')).toBeDefined()
    expect(screen.getByTestId('contact-section')).toBeDefined()
  })
})
