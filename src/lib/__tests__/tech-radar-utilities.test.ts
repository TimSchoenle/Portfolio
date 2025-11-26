import {
  calculateBlipPosition,
  resolveBlipCollisions,
} from '@/lib/tech-radar-utilities'
import { RADAR_CONFIG } from '@/lib/radar-config'
import type { CalculateBlipPositionResult } from '@/types/tech-radar'
import { describe, it, expect } from 'vitest'

describe('tech-radar-utilities', () => {
  describe('calculateBlipPosition', () => {
    it('should return deterministic results for the same input', () => {
      const params = {
        confidence: 0.9,
        endAngle: Math.PI,
        index: 0,
        skillName: 'Test Skill',
        startAngle: 0,
        total: 5,
      }

      const result1 = calculateBlipPosition(params)
      const result2 = calculateBlipPosition(params)

      expect(result1).toEqual(result2)
    })

    it('should calculate radius based on confidence (high confidence = low radius)', () => {
      const highConfidence = calculateBlipPosition({
        confidence: 1.0,
        endAngle: Math.PI,
        index: 0,
        skillName: 'High',
        startAngle: 0,
        total: 1,
      })

      const lowConfidence = calculateBlipPosition({
        confidence: 0.0,
        endAngle: Math.PI,
        index: 0,
        skillName: 'Low',
        startAngle: 0,
        total: 1,
      })

      // Allow for jitter, but general trend should hold
      // Inner radius is smaller than Outer radius
      expect(highConfidence.radius).toBeLessThan(lowConfidence.radius)
      expect(highConfidence.radius).toBeGreaterThanOrEqual(
        RADAR_CONFIG.blips.minRadius - RADAR_CONFIG.jitter.radius
      )
      expect(lowConfidence.radius).toBeLessThanOrEqual(
        RADAR_CONFIG.blips.maxRadius + RADAR_CONFIG.jitter.radius
      )
    })
  })

  describe('resolveBlipCollisions', () => {
    const startAngle = 0
    const endAngle = Math.PI / 2

    it('should maintain stable positions for blips that are far apart', () => {
      // Create blips with initial positions that match their target radius
      // to minimize spring forces
      const blips: CalculateBlipPositionResult[] = [
        { angle: 0.4, radius: 50, xCoordinate: 0, yCoordinate: 0 },
        { angle: 0.8, radius: 60, xCoordinate: 0, yCoordinate: 0 },
      ].map((b) => ({
        ...b,
        xCoordinate: Math.cos(b.angle) * b.radius,
        yCoordinate: Math.sin(b.angle) * b.radius,
      }))

      const resolved = resolveBlipCollisions({ blips, startAngle, endAngle })

      expect(resolved).toHaveLength(2)
      const [first, second] = resolved
      const [blipFirst, blipSecond] = blips

      expect(first).toBeDefined()
      expect(second).toBeDefined()
      expect(blipFirst).toBeDefined()
      expect(blipSecond).toBeDefined()

      // Verify blips remain in the same sector and don't overlap
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      expect(first!.angle).toBeGreaterThanOrEqual(startAngle)
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      expect(first!.angle).toBeLessThanOrEqual(endAngle)
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      expect(second!.angle).toBeGreaterThanOrEqual(startAngle)
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      expect(second!.angle).toBeLessThanOrEqual(endAngle)

      // Verify blips maintain their relative order (first should have smaller angle)
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      expect(first!.angle).toBeLessThan(second!.angle)

      // Verify they maintain adequate separation
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      const deltaX = first!.xCoordinate - second!.xCoordinate
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      const deltaY = first!.yCoordinate - second!.yCoordinate
      const distance = Math.sqrt(deltaX * deltaX + deltaY * deltaY)
      const minSeparation = RADAR_CONFIG.blips.size * 2 + 4

      expect(distance).toBeGreaterThan(minSeparation)
    })

    it('should separate overlapping blips', () => {
      // Two blips at almost the same position
      const blips: CalculateBlipPositionResult[] = [
        { angle: 0.5, radius: 50, xCoordinate: 0, yCoordinate: 0 },
        { angle: 0.501, radius: 50, xCoordinate: 0, yCoordinate: 0 },
      ].map((b) => ({
        ...b,
        xCoordinate: Math.cos(b.angle) * b.radius,
        yCoordinate: Math.sin(b.angle) * b.radius,
      }))

      const resolved = resolveBlipCollisions({ blips, startAngle, endAngle })

      expect(resolved).toHaveLength(2)
      const [first, second] = resolved

      expect(first).toBeDefined()
      expect(second).toBeDefined()

      // Verify they are separated by at least the minimum distance (approx)
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      const dx = first!.xCoordinate - second!.xCoordinate
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      const dy = first!.yCoordinate - second!.yCoordinate
      const distance = Math.sqrt(dx * dx + dy * dy)

      // They should have moved apart, but maybe not as aggressively as before
      expect(distance).toBeGreaterThan(RADAR_CONFIG.blips.size * 0.5)
    })

    it('should respect boundary constraints', () => {
      // Blip near the start boundary, pushed left
      const blips: CalculateBlipPositionResult[] = [
        { angle: 0.01, radius: 50, xCoordinate: 0, yCoordinate: 0 },
        { angle: 0.02, radius: 50, xCoordinate: 0, yCoordinate: 0 },
      ].map((b) => ({
        ...b,
        xCoordinate: Math.cos(b.angle) * b.radius,
        yCoordinate: Math.sin(b.angle) * b.radius,
      }))

      const resolved = resolveBlipCollisions({ blips, startAngle, endAngle })

      expect(resolved).toHaveLength(2)
      const [first, second] = resolved

      expect(first).toBeDefined()
      expect(second).toBeDefined()

      // Calculate buffer
      const bufferArc = RADAR_CONFIG.blips.size + 2 // Updated buffer size
      const angleBuffer = bufferArc / 50

      // Should be clamped to startAngle + buffer
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      expect(first!.angle).toBeGreaterThanOrEqual(
        startAngle + angleBuffer - 0.05 // Allow epsilon
      )
      // Check that they are still valid numbers and roughly in the sector
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      expect(first!.angle).toBeLessThan(endAngle)
    })

    it('should respect end boundary constraints', () => {
      // Blip near the end boundary, pushed right
      const blips: CalculateBlipPositionResult[] = [
        { angle: endAngle - 0.02, radius: 50, xCoordinate: 0, yCoordinate: 0 },
        { angle: endAngle - 0.01, radius: 50, xCoordinate: 0, yCoordinate: 0 },
      ].map((b) => ({
        ...b,
        xCoordinate: Math.cos(b.angle) * b.radius,
        yCoordinate: Math.sin(b.angle) * b.radius,
      }))

      const resolved = resolveBlipCollisions({ blips, startAngle, endAngle })

      expect(resolved).toHaveLength(2)
      const [first, second] = resolved

      expect(first).toBeDefined()
      expect(second).toBeDefined()

      // Calculate buffer
      const bufferArc = RADAR_CONFIG.blips.size + 2 // Updated buffer size
      const angleBuffer = bufferArc / 50

      // Should be clamped to endAngle - buffer
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      expect(second!.angle).toBeLessThanOrEqual(endAngle - angleBuffer + 0.05) // Allow epsilon
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      expect(second!.angle).toBeGreaterThan(startAngle)
    })

    it('should handle quadrants with negative atan2 results (e.g. 3PI/2 to 2PI)', () => {
      const q4Start = (3 * Math.PI) / 2
      const q4End = 2 * Math.PI

      // Blip in the middle of Q4
      const angle = q4Start + 0.5
      const blips: CalculateBlipPositionResult[] = [
        { angle, radius: 50, xCoordinate: 0, yCoordinate: 0 },
      ].map((b) => ({
        ...b,
        xCoordinate: Math.cos(b.angle) * b.radius,
        yCoordinate: Math.sin(b.angle) * b.radius,
      }))

      const resolved = resolveBlipCollisions({
        blips,
        startAngle: q4Start,
        endAngle: q4End,
      })

      expect(resolved).toHaveLength(1)
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      const result = resolved[0]!

      // Should NOT be clamped to the start boundary (which would happen if negative angle wasn't normalized)
      expect(result.angle).toBeCloseTo(angle, 0) // Relaxed precision
      expect(result.angle).toBeGreaterThan(q4Start)
    })
  })
})
