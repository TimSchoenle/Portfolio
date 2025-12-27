import { describe, it, expect, vi, beforeEach } from 'vitest'
import {
  scrollToSection,
  goToSection,
  openNewTab,
  sendMailTo,
  createPushHandler,
  createOnSelectSection,
  type LocalizedRouter,
} from '../actions'

const mockRouter = {
  push: vi.fn(),
  replace: vi.fn(),
  back: vi.fn(),
  forward: vi.fn(),
  refresh: vi.fn(),
  prefetch: vi.fn(),
} as unknown as LocalizedRouter

describe('actions', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('scrollToSection', () => {
    it('should scroll to element if found', () => {
      const mockElement = { scrollIntoView: vi.fn() }
      vi.spyOn(document, 'querySelector').mockReturnValue(mockElement as any)

      scrollToSection('test-id')
      expect(document.querySelector).toHaveBeenCalledWith('#test-id')
      expect(mockElement.scrollIntoView).toHaveBeenCalledWith({
        behavior: 'smooth',
      })
    })

    it('should do nothing if element not found', () => {
      vi.spyOn(document, 'querySelector').mockReturnValue(null)
      // Should not throw
      scrollToSection('test-id')
    })
  })

  describe('goToSection', () => {
    it('should scroll if on homepage', () => {
      // Mock scrollToSection indirect logic or just spy on querySelector again
      const mockElement = { scrollIntoView: vi.fn() }
      vi.spyOn(document, 'querySelector').mockReturnValue(mockElement as any)

      goToSection({
        locale: 'en',
        pathname: '/en',
        router: mockRouter,
        sectionId: 'about',
      })

      expect(document.querySelector).toHaveBeenCalledWith('#about')
      expect(mockRouter.push).not.toHaveBeenCalled()
    })

    it('should push route if not on homepage', () => {
      goToSection({
        locale: 'en',
        pathname: '/en/imprint',
        router: mockRouter,
        sectionId: 'about',
      })

      expect(mockRouter.push).toHaveBeenCalledWith('/en#about')
    })
  })

  describe('openNewTab', () => {
    it('should open url in new tab', () => {
      const openSpy = vi.spyOn(window, 'open').mockImplementation(() => null)
      openNewTab('https://example.com')
      expect(openSpy).toHaveBeenCalledWith(
        'https://example.com',
        '_blank',
        'noopener,noreferrer'
      )
    })
  })

  describe('sendMailTo', () => {
    it('should set window location', () => {
      Object.defineProperty(window, 'location', {
        value: { href: '' },
        writable: true,
      })
      sendMailTo('test@example.com')
      expect(window.location.href).toBe('mailto:test@example.com')
    })
  })

  describe('createPushHandler', () => {
    it('should return a function that calls run -> router.push', () => {
      const mockRun = vi.fn((cb) => cb())
      const handler = createPushHandler({
        href: '/imprint',
        locale: 'en',
        router: mockRouter,
        run: mockRun,
      })

      handler('val')
      expect(mockRun).toHaveBeenCalled()
      expect(mockRouter.push).toHaveBeenCalledWith('/en/imprint')
    })
  })

  describe('createOnSelectSection', () => {
    it('should return a function that calls run -> goToSection', () => {
      const mockRun = vi.fn((cb) => cb())
      // GoToSection calls querySelector or push
      const mockElement = { scrollIntoView: vi.fn() }
      vi.spyOn(document, 'querySelector').mockReturnValue(mockElement as any)

      const handler = createOnSelectSection({
        locale: 'en',
        pathname: '/en',
        router: mockRouter,
        run: mockRun,
        sectionId: 'contact',
      })

      handler('val')
      expect(mockRun).toHaveBeenCalled()
      expect(document.querySelector).toHaveBeenCalledWith('#contact')
    })
  })
})
