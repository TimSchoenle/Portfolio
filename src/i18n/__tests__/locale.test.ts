import { describe, it, expect, vi } from 'vitest'
import { ensureLocale, maybeLocale, assertLocale } from '../locale'

// Mock next/navigation
vi.mock('next/navigation', () => ({
  notFound: vi.fn(() => {
    throw new Error('notFound')
  }),
}))

// Mock routing
vi.mock('@/i18n/routing', () => ({
  routing: {
    locales: ['en', 'de'],
  },
}))

describe('locale utilities', () => {
  describe('ensureLocale', () => {
    it('returns valid locale', () => {
      expect(ensureLocale('en')).toBe('en')
      expect(ensureLocale('de')).toBe('de')
    })

    it('throws notFound for invalid locale', () => {
      expect(() => ensureLocale('fr')).toThrow('notFound')
    })
  })

  describe('maybeLocale', () => {
    it('returns valid locale', () => {
      expect(maybeLocale('en')).toBe('en')
      expect(maybeLocale('de')).toBe('de')
    })

    it('returns null for invalid locale', () => {
      expect(maybeLocale('fr')).toBe(null)
      expect(maybeLocale('invalid')).toBe(null)
    })
  })

  describe('assertLocale', () => {
    it('does not throw for valid locale', () => {
      expect(() => assertLocale('en')).not.toThrow()
      expect(() => assertLocale('de')).not.toThrow()
    })

    it('throws notFound for invalid locale', () => {
      expect(() => assertLocale('fr')).toThrow('notFound')
    })
  })
})
