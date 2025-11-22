import { describe, it, expect, vi } from 'vitest'

// Mock server-only
vi.mock('server-only', () => ({}))

// Mock next-intl server
vi.mock('next-intl/server', () => ({
  getTranslations: vi.fn(async () => (key: string) => key),
}))

// Mock lucide-react
vi.mock('lucide-react', () => ({
  Send: () => null,
  Mail: () => null,
  Github: () => null,
  Linkedin: () => null,
  Download: () => null,
  FileText: () => null,
  GitBranch: () => null,
  MapPin: () => null,
}))

// Mock config
vi.mock('@/lib/config', () => ({
  siteConfig: {
    email: 'test@example.com',
    github: 'https://github.com/testuser',
    linkedin: 'https://linkedin.com/in/testuser',
    location: 'Test Location',
  },
}))

// Mock fs/promises correctly
vi.mock('node:fs/promises', () => ({
  default: {
    access: vi.fn(async () => undefined),
    constants: { R_OK: 4 },
  },
  access: vi.fn(async () => undefined),
  constants: { R_OK: 4 },
}))

describe('ContactSection', () => {
  it('can be imported', async () => {
    const { ContactSection } = await import('../contact-section')
    expect(ContactSection).toBeDefined()
  })

  it('is a server component', async () => {
    const { ContactSection } = await import('../contact-section')
    expect(typeof ContactSection).toBe('function')
  })
})
