import { RADAR_CONFIG } from '@/lib/radar-config'
import type {
  CalculateBlipPositionParameters,
  CalculateBlipPositionResult,
} from '@/types/tech-radar'

/**
 * Simple hash function to convert a string to a deterministic number.
 * This ensures the same skill name always produces the same seed.
 */
export const hashString: (input: string) => number = (
  input: string
): number => {
  let hash: number = 0
  for (let index: number = 0; index < input.length; index++) {
    const char: number = input.codePointAt(index) ?? 0
    hash = (hash << 5) - hash + char
    hash = hash & hash // Convert to 32-bit integer
  }
  return Math.abs(hash)
}

// Seeded random number generator for deterministic layout
export const seededRandom: (seed: number) => number = (
  seed: number
): number => {
  const temporary: number = Math.sin(seed) * 10_000
  return temporary - Math.floor(temporary)
}

/**
 * Calculate position for a blip within a quadrant.
 * `startAngle` and `endAngle` define the angular range (radians).
 * Uses skillName for deterministic seeding to ensure consistent rendering.
 */
export const calculateBlipPosition: (
  parameters: CalculateBlipPositionParameters
) => CalculateBlipPositionResult = ({
  confidence,
  endAngle,
  index,
  skillName,
  startAngle,
  total,
}: CalculateBlipPositionParameters): CalculateBlipPositionResult => {
  // Generate deterministic seeds from skill name
  const baseSeed: number = hashString(skillName)
  const angleSeed: number = baseSeed
  const radiusSeed: number = baseSeed + 100

  const angleStep: number = (endAngle - startAngle) / (total + 1)
  const angleJitter: number =
    (seededRandom(angleSeed) - 0.5) * RADAR_CONFIG.jitter.angle
  const angle: number = startAngle + (index + 1) * angleStep + angleJitter

  // Map confidence (0-1) to radius (inner to outer)
  // High confidence -> closer to center
  // Low confidence -> closer to edge
  const radiusJitter: number =
    (seededRandom(radiusSeed) - 0.5) * RADAR_CONFIG.jitter.radius
  const radius: number =
    RADAR_CONFIG.blips.minRadius +
    (1 - confidence) *
      (RADAR_CONFIG.blips.maxRadius - RADAR_CONFIG.blips.minRadius) +
    radiusJitter

  const xCoordinate: number = Math.cos(angle) * radius
  const yCoordinate: number = Math.sin(angle) * radius
  return { angle, radius, xCoordinate, yCoordinate }
}

interface HandleBlipCollisionParameters {
  readonly blip1: CalculateBlipPositionResult
  readonly blip2: CalculateBlipPositionResult
  readonly endAngle: number
  readonly minDistance: number
  readonly startAngle: number
}

interface HandleBlipCollisionResult {
  readonly newBlip1: CalculateBlipPositionResult
  readonly newBlip2: CalculateBlipPositionResult
}

/**
 * Helper function to handle collision between two blips
 */
const handleBlipCollision: (
  parameters: HandleBlipCollisionParameters
  // eslint-disable-next-line max-lines-per-function
) => HandleBlipCollisionResult | null = ({
  blip1,
  blip2,
  endAngle,
  minDistance,
  startAngle,
}: HandleBlipCollisionParameters): HandleBlipCollisionResult | null => {
  const deltaX: number = blip1.xCoordinate - blip2.xCoordinate
  const deltaY: number = blip1.yCoordinate - blip2.yCoordinate
  const distance: number = Math.hypot(deltaX, deltaY)

  if (distance >= minDistance) {
    return null
  }

  // Collision detected - calculate overlap amount
  const overlap: number = minDistance - distance

  // Determine direction based on relative angles
  let angleDifference: number = blip1.angle - blip2.angle
  if (Math.abs(angleDifference) < 0.0001) {
    angleDifference = -1
  }
  const direction: number = Math.sign(angleDifference)

  // Calculate angle adjustment
  const totalMoveAngle: number = overlap / ((blip1.radius + blip2.radius) / 2)
  const moveAngle: number = (totalMoveAngle / 2) * 1.1

  // Nudge blip1
  const bufferArc1: number = RADAR_CONFIG.blips.size + 4
  const angleBuffer1: number = bufferArc1 / blip1.radius
  let newAngle1: number = blip1.angle + direction * moveAngle
  newAngle1 = Math.max(
    startAngle + angleBuffer1,
    Math.min(endAngle - angleBuffer1, newAngle1)
  )

  // Nudge blip2
  const bufferArc2: number = RADAR_CONFIG.blips.size + 4
  const angleBuffer2: number = bufferArc2 / blip2.radius
  let newAngle2: number = blip2.angle - direction * moveAngle
  newAngle2 = Math.max(
    startAngle + angleBuffer2,
    Math.min(endAngle - angleBuffer2, newAngle2)
  )

  return {
    newBlip1: {
      ...blip1,
      angle: newAngle1,
      xCoordinate: Math.cos(newAngle1) * blip1.radius,
      yCoordinate: Math.sin(newAngle1) * blip1.radius,
    },
    newBlip2: {
      ...blip2,
      angle: newAngle2,
      xCoordinate: Math.cos(newAngle2) * blip2.radius,
      yCoordinate: Math.sin(newAngle2) * blip2.radius,
    },
  }
}

interface ProcessBlipPairParameters {
  readonly endAngle: number
  readonly index1: number
  readonly index2: number
  readonly minDistance: number
  readonly resolvedBlips: CalculateBlipPositionResult[]
  readonly startAngle: number
}

/**
 * Process a single blip pair for collision resolution
 */
const processBlipPair: (parameters: ProcessBlipPairParameters) => void = ({
  endAngle,
  index1,
  index2,
  minDistance,
  resolvedBlips,
  startAngle,
}: ProcessBlipPairParameters): void => {
  const blip1: CalculateBlipPositionResult | undefined =
    resolvedBlips.at(index1)
  const blip2: CalculateBlipPositionResult | undefined =
    resolvedBlips.at(index2)

  if (!blip1 || !blip2) {
    return
  }

  const result: HandleBlipCollisionResult | null = handleBlipCollision({
    blip1,
    blip2,
    endAngle,
    minDistance,
    startAngle,
  })

  if (result) {
    resolvedBlips.splice(index1, 1, result.newBlip1)
    resolvedBlips.splice(index2, 1, result.newBlip2)
  }
}

/**
 * Resolves collisions between blips by adjusting their angles.
 * Preserves radius (confidence) but nudges angles to prevent overlap.
 */
export const resolveBlipCollisions: (
  blips: CalculateBlipPositionResult[],
  startAngle: number,
  endAngle: number
) => CalculateBlipPositionResult[] = (
  blips: CalculateBlipPositionResult[],
  startAngle: number,
  endAngle: number
): CalculateBlipPositionResult[] => {
  const resolvedBlips: CalculateBlipPositionResult[] = [...blips]
  const minDistance: number = RADAR_CONFIG.blips.size * 2 + 14
  const iterations: number = 20

  for (let iteration: number = 0; iteration < iterations; iteration++) {
    for (let index: number = 0; index < resolvedBlips.length; index++) {
      for (
        let index_: number = index + 1;
        index_ < resolvedBlips.length;
        index_++
      ) {
        processBlipPair({
          endAngle,
          index1: index,
          index2: index_,
          minDistance,
          resolvedBlips,
          startAngle,
        })
      }
    }
  }

  return resolvedBlips
}
