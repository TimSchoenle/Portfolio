import { describe, it, expect, vi } from 'vitest'
import { mapExperienceData } from '../experience-utilities'

describe('experience-utilities', () => {
  const mockFormat = {
    dateTime: vi.fn((_date, _options) => 'Formatted Date'),
  }

  it('should return empty array if raw input is not an array', () => {
    // @ts-ignore
    const result = mapExperienceData({
      raw: null as any,
      format: mockFormat as any,
      presentLabel: 'Present',
    })
    expect(result).toEqual([])
  })

  it('should map experience data correctly', () => {
    const raw = [
      {
        company: 'Company A',
        title: 'Developer',
        location: 'City A',
        start: { year: 2020, month: 1 },
        end: { year: 2021, month: 1 },
        achievements: ['Achieved X'],
      },
    ]

    const result = mapExperienceData({
      raw: raw as any,
      format: mockFormat as any,
      presentLabel: 'Present',
    })

    expect(result).toHaveLength(1)
    expect(result[0]).toEqual(
      expect.objectContaining({
        company: 'Company A',
        role: 'Developer',
        location: 'City A',
        achievements: ['Achieved X'],
        duration: 'Formatted Date - Formatted Date',
        index: 0,
      })
    )
  })

  it('should handle present jobs (null end date)', () => {
    const raw = [
      {
        company: 'Company B',
        title: 'Senior Developer',
        start: { year: 2022, month: 1 },
        end: null,
        achievements: [],
      },
    ]

    const result = mapExperienceData({
      raw: raw as any,
      format: mockFormat as any,
      presentLabel: 'Present',
    })

    expect(result).toHaveLength(1)
    if (result[0]) {
      expect(result[0].duration).toBe('Formatted Date - Present')
    }
  })
})
