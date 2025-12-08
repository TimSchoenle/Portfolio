import { describe, it, expect } from 'vitest'
import {
  buildCalendar,
  dayLabelTriple,
  isoDate,
  makeDataMap,
  sumCount,
  sundayOfWeekUTC,
} from '../contribution-calendar'
import type { ContributionPoint } from '@/models/github'

describe('contribution-calendar', () => {
  describe('isoDate', () => {
    it('formats date as YYYY-MM-DD', () => {
      const date = new Date('2023-01-01T12:00:00Z')
      expect(isoDate(date)).toBe('2023-01-01')
    })
  })

  describe('sundayOfWeekUTC', () => {
    it('returns the Sunday of the week for a given date', () => {
      // Wednesday
      const date = new Date('2023-11-22T00:00:00Z')
      const sunday = sundayOfWeekUTC(date)
      expect(isoDate(sunday)).toBe('2023-11-19')
    })

    it('returns the same day if it is already Sunday', () => {
      const date = new Date('2023-11-19T00:00:00Z')
      const sunday = sundayOfWeekUTC(date)
      expect(isoDate(sunday)).toBe('2023-11-19')
    })
  })

  describe('makeDataMap', () => {
    it('creates a map from contribution points', () => {
      const points: ContributionPoint[] = [
        { date: '2023-01-01', count: 5, level: 1 },
        { date: '2023-01-02', count: 10, level: 2 },
      ]
      const map = makeDataMap(points)
      expect(map.size).toBe(2)
      expect(map.get('2023-01-01')).toEqual(points[0])
      expect(map.get('2023-01-02')).toEqual(points[1])
    })
  })

  describe('sumCount', () => {
    it('sums contribution counts', () => {
      const points: ContributionPoint[] = [
        { date: '2023-01-01', count: 5, level: 1 },
        { date: '2023-01-02', count: 10, level: 2 },
      ]
      expect(sumCount(points)).toBe(15)
    })

    it('returns 0 for empty array', () => {
      expect(sumCount([])).toBe(0)
    })
  })

  describe('buildCalendar', () => {
    it('returns empty model for empty data', () => {
      const result = buildCalendar({ data: [], locale: 'en-US' })
      expect(result.weeks).toHaveLength(0)
      expect(result.monthLabels).toHaveLength(0)
    })

    it('builds a calendar for a range of data', () => {
      const data: ContributionPoint[] = [
        { date: '2023-01-01', count: 1, level: 1 }, // Sunday
        { date: '2023-01-08', count: 1, level: 1 }, // Next Sunday
      ]
      // Should span at least 2 weeks
      const result = buildCalendar({ data, locale: 'en-US' })
      expect(result.weeks.length).toBeGreaterThanOrEqual(2)
      expect(result.weeks[0]!.days[0]?.date).toBe('2023-01-01')
    })
    it('handles gaps in data (empty weeks)', () => {
      const data: ContributionPoint[] = [
        { date: '2023-01-01', count: 1, level: 1 }, // Sunday
        { date: '2023-01-15', count: 1, level: 1 }, // 2 weeks later
      ]
      const result = buildCalendar({ data, locale: 'en-US' })
      // Should have 3 weeks: Jan 1-7, Jan 8-14 (empty), Jan 15-21
      expect(result.weeks.length).toBeGreaterThanOrEqual(3)
      // Middle week should be empty of contributions
      const middleWeek = result.weeks[1]
      const hasContributions = middleWeek!.days.some((d) => d !== null)
      expect(hasContributions).toBe(false)
    })
  })

  describe('dayLabelTriple', () => {
    it('returns empty strings for empty data', () => {
      const result = dayLabelTriple({ data: [], locale: 'en-US' })
      expect(result).toEqual({ dayFive: '', dayOne: '', dayThree: '' })
    })

    it('returns correct weekday labels', () => {
      const data: ContributionPoint[] = [
        { date: '2023-01-01', count: 1, level: 1 },
      ]
      const result = dayLabelTriple({ data, locale: 'en-US' })
      // Sunday is day 0.
      // dayOne = Mon, dayThree = Wed, dayFive = Fri
      expect(result.dayOne).toBe('Mon')
      expect(result.dayThree).toBe('Wed')
      expect(result.dayFive).toBe('Fri')
    })
  })
})
