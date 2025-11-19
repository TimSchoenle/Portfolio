import { describe, it, expect } from 'vitest'
import { cn, panic } from '../utilities'

describe('utilities', () => {
  describe('cn', () => {
    it('merges class names correctly', () => {
      expect(cn('foo', 'bar')).toBe('foo bar')
      expect(cn('foo', { bar: true })).toBe('foo bar')
      expect(cn('foo', { bar: false })).toBe('foo')
    })

    it('handles tailwind conflicts', () => {
      expect(cn('p-4', 'p-2')).toBe('p-2')
    })
  })

  describe('panic', () => {
    it('throws an error with the message', () => {
      expect(() => panic('Something went wrong')).toThrow(
        'Something went wrong'
      )
    })
  })
})
