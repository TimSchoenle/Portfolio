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

    it('should not modify blips that are far apart', () => {
      const blips: CalculateBlipPositionResult[] = [
        { angle: 0.2, radius: 50, xCoordinate: 0, yCoordinate: 0 },
        { angle: 0.8, radius: 60, xCoordinate: 100, yCoordinate: 100 },
      ].map((b) => ({
        ...b,
        xCoordinate: Math.cos(b.angle) * b.radius,
        yCoordinate: Math.sin(b.angle) * b.radius,
      }))

      const resolved = resolveBlipCollisions(blips, startAngle, endAngle)

      expect(resolved).toHaveLength(2)
      const [first, second] = resolved
      const [blipFirst, blipSecond] = blips

      expect(first).toBeDefined()
      expect(second).toBeDefined()
      expect(blipFirst).toBeDefined()
      expect(blipSecond).toBeDefined()

      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      expect(first!.angle).toBeCloseTo(blipFirst!.angle)
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      expect(second!.angle).toBeCloseTo(blipSecond!.angle)
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

      const resolved = resolveBlipCollisions(blips, startAngle, endAngle)

      expect(resolved).toHaveLength(2)
      const [first, second] = resolved
      const [blipFirst, blipSecond] = blips

      expect(first).toBeDefined()
      expect(second).toBeDefined()
      expect(blipFirst).toBeDefined()
      expect(blipSecond).toBeDefined()

      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      const angleDiff = Math.abs(first!.angle - second!.angle)
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      const initialDiff = Math.abs(blipFirst!.angle - blipSecond!.angle)

      expect(angleDiff).toBeGreaterThan(initialDiff)

      // Verify they are separated by at least the minimum distance (approx)
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      const dx = first!.xCoordinate - second!.xCoordinate
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      const dy = first!.yCoordinate - second!.yCoordinate
      const distance = Math.sqrt(dx * dx + dy * dy)

      expect(distance).toBeGreaterThan(RADAR_CONFIG.blips.size * 2)
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

      const resolved = resolveBlipCollisions(blips, startAngle, endAngle)

      expect(resolved).toHaveLength(2)
      const [first, second] = resolved

      expect(first).toBeDefined()
      expect(second).toBeDefined()

      // Calculate buffer
      const bufferArc = RADAR_CONFIG.blips.size + 4
      const angleBuffer = bufferArc / 50

      // Should be clamped to startAngle + buffer
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      expect(first!.angle).toBeGreaterThanOrEqual(
        startAngle + angleBuffer - 0.0001
      )
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      expect(first!.angle).toBeLessThan(second!.angle)
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

      const resolved = resolveBlipCollisions(blips, startAngle, endAngle)

      expect(resolved).toHaveLength(2)
      const [first, second] = resolved

      expect(first).toBeDefined()
      expect(second).toBeDefined()

      // Calculate buffer
      const bufferArc = RADAR_CONFIG.blips.size + 4
      const angleBuffer = bufferArc / 50

      // Should be clamped to endAngle - buffer
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      expect(second!.angle).toBeLessThanOrEqual(endAngle - angleBuffer + 0.0001)
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      expect(second!.angle).toBeGreaterThan(first!.angle)
    })
  })
})
