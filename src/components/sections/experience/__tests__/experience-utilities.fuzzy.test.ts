import { test, fc } from '@fast-check/vitest'
import { describe, expect, vi } from 'vitest'

import { mapExperienceData } from '../experience-utilities'
import type { ResumeExperience } from '@/types/resume'

// Mock createFormatter result
const mockFormat = {
  dateTime: vi.fn(
    (
      date: Date,
      _options?: {
        readonly month: string
        readonly year: string
      }
    ): string => date.toISOString().split('T')[0]!
  ),
}

describe('Experience Utilities Fuzzy Tests', () => {
  describe('mapExperienceData', () => {
    test.prop([
      // ResumeExperience[]
      fc.array(
        fc.record({
          achievements: fc.array(fc.string()),
          company: fc.string(),
          // Start date: year/month
          start: fc.record({
            year: fc.integer({ min: 1, max: 3000 }),
            month: fc.integer({ min: 1, max: 12 }),
          }),
          // End date: nullable record
          end: fc.option(
            fc.record({
              year: fc.integer({ min: 1, max: 3000 }),
              month: fc.integer({ min: 1, max: 12 }),
            })
          ),
          location: fc.string(),
          title: fc.string(),
          website: fc.string(),
        })
      ),
      // Present label constant (must be non-empty to validly test endsWith)
      fc.string({ minLength: 1 }),
    ])(
      'should correctly map raw data to UI properties',
      (raw, presentLabel) => {
        const result = mapExperienceData({
          format: mockFormat as any,
          presentLabel,
          raw: raw as ResumeExperience[],
        })

        expect(result).toHaveLength(raw.length)

        result.forEach((item, index) => {
          const source = raw[index]!

          // Check direct mapping
          expect(item.index).toBe(index)
          expect(item.company).toBe(source.company)
          expect(item.role).toBe(source.title)
          expect(item.location).toBe(source.location)
          expect(item.achievements).toEqual(source.achievements)

          const expectedStart = mockFormat.dateTime(
            new Date(Date.UTC(source.start.year, source.start.month - 1, 1)),
            {
              month: 'short',
              year: 'numeric',
            }
          )

          const expectedEnd =
            source.end === null || source.end === undefined
              ? presentLabel
              : mockFormat.dateTime(
                  new Date(Date.UTC(source.end.year, source.end.month - 1, 1)),
                  {
                    month: 'short',
                    year: 'numeric',
                  }
                )

          expect(item.duration).toBe(`${expectedStart} - ${expectedEnd}`)
        })
      }
    )
  })
})
