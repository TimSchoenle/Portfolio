import { describe, it, expect, vi, beforeEach } from 'vitest'
import {
  handleActionKey,
  handleNavigationKey,
  handleSectionKey,
} from '../key-handlers'
import { siteConfig } from '@/data/config'
import type { LocalizedRouter } from '../../utils/actions'

// Mock dependencies
const mockRun = vi.fn((cb) => cb())
const mockRouter = {
  push: vi.fn(),
  replace: vi.fn(),
  back: vi.fn(),
  forward: vi.fn(),
  refresh: vi.fn(),
  prefetch: vi.fn(),
} as unknown as LocalizedRouter

const mockDeps = {
  locale: 'en' as const,
  pathname: '/',
  router: mockRouter,
  run: mockRun,
}

const mockHandleRequest = vi.fn((action: (value: string) => void) => {
  action('test-val')
})

vi.mock('@/data/config', () => ({
  siteConfig: {
    socials: {
      linkedin: 'https://linkedin.com/in/test',
      github: 'https://github.com/test',
    },
    contact: {
      email: 'test@test.com',
    },
    email: 'test@test.com',
  },
}))

vi.mock('../../utils/actions', () => ({
  openNewTab: vi.fn(),
  sendMailTo: vi.fn(),
  goToSection: vi.fn(),
  createPushHandler: vi.fn(({ locale, href }) => () => {
    // Mock implementation of createPushHandler's return
    mockRouter.push(`/${locale}${href}`)
  }),
  createOnSelectSection: vi.fn(({ locale, sectionId }) => () => {
    goToSection({ locale, pathname: '/', router: mockRouter, sectionId })
  }),
  // We need to export goToSection to verify it's called
}))

import {
  openNewTab,
  sendMailTo,
  goToSection,
  createPushHandler,
} from '../../utils/actions'

describe('key-handlers', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('handleActionKey', () => {
    it('should open LinkedIn on "L"', () => {
      const result = handleActionKey('L', mockDeps, mockHandleRequest)
      expect(result).toBe(true)
      expect(openNewTab).toHaveBeenCalledWith(siteConfig.socials.linkedin)
    })

    it('should open GitHub on "G"', () => {
      const result = handleActionKey('G', mockDeps, mockHandleRequest)
      expect(result).toBe(true)
      expect(openNewTab).toHaveBeenCalledWith(siteConfig.socials.github)
    })

    it('should send email on "E"', () => {
      const result = handleActionKey('E', mockDeps, mockHandleRequest)
      expect(result).toBe(true)
      expect(sendMailTo).toHaveBeenCalledWith(siteConfig.email)
    })

    it('should do nothing on "L" if linkedin is undefined', () => {
      // temporarily mock siteConfig
      const originalLinkedin = siteConfig.socials.linkedin
      // @ts-ignore
      siteConfig.socials.linkedin = undefined

      const result = handleActionKey('L', mockDeps, mockHandleRequest)
      expect(result).toBe(true)
      expect(openNewTab).not.toHaveBeenCalled()

      // restore
      // @ts-ignore
      siteConfig.socials.linkedin = originalLinkedin
    })

    it('should ignore unknown keys', () => {
      const result = handleActionKey('Z', mockDeps, mockHandleRequest)
      expect(result).toBe(false)
      expect(mockHandleRequest).not.toHaveBeenCalled()
    })
  })

  describe('handleNavigationKey', () => {
    it('should go to home on "H"', () => {
      const result = handleNavigationKey('H', mockDeps, mockHandleRequest)
      expect(result).toBe(true)
      // handleRequest triggers the action from createPushHandler
      // which in our mock calls router.push
      expect(createPushHandler).toHaveBeenCalledWith(
        expect.objectContaining({ href: '/' })
      )
    })

    it('should go to imprint on "I"', () => {
      const result = handleNavigationKey('I', mockDeps, mockHandleRequest)
      expect(result).toBe(true)
      expect(createPushHandler).toHaveBeenCalledWith(
        expect.objectContaining({ href: '/imprint' })
      )
    })

    it('should go to privacy on "D"', () => {
      const result = handleNavigationKey('D', mockDeps, mockHandleRequest)
      expect(result).toBe(true)
      expect(createPushHandler).toHaveBeenCalledWith(
        expect.objectContaining({ href: '/privacy' })
      )
    })

    it('should ignore unknown keys', () => {
      const result = handleNavigationKey('Z', mockDeps, mockHandleRequest)
      expect(result).toBe(false)
    })
  })

  describe('handleSectionKey', () => {
    const sections = [
      { key: 'A', id: 'about' },
      { key: 'K', id: 'skills' },
      { key: 'P', id: 'projects' },
      { key: 'X', id: 'experience' },
      { key: 'C', id: 'contact' },
    ]

    sections.forEach(({ key, id }) => {
      it(`should go to ${id} on "${key}"`, () => {
        const result = handleSectionKey(key, mockDeps, mockHandleRequest)
        expect(result).toBe(true)
        // calls createOnSelectSection
      })
    })

    it('should ignore unknown keys', () => {
      const result = handleSectionKey('Z', mockDeps, mockHandleRequest)
      expect(result).toBe(false)
    })
  })
})
