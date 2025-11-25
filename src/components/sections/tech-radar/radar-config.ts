/**
 * Tech Radar Configuration
 * All dimensions, radii, and visual constants for easy customization
 */

import type {
  LabelPathsType,
  QuadrantAnglesType,
  QuadrantStylesType,
  RadarConfigType,
} from '@/types/tech-radar'

/**
 * Radar dimensions and layout
 */
export const RADAR_CONFIG: RadarConfigType = {
  /** Animation settings */
  animation: {
    sonarSweepAngle: 45, // degrees
    sonarSweepRadius: 100,
  },

  /** Blip positioning constraints */
  blips: {
    hoverScale: 1.5,
    maxRadius: 85,
    minRadius: 25,
    size: 3,
    strokeWidth: 0.5,
  },

  /** Background circle radii */
  circles: {
    inner: 32.5,
    middle: 67.5,
    outer: 97.5,
  },

  /** Jitter amounts for position randomization */
  jitter: {
    angle: 0.2,
    radius: 5,
  },

  /** Quadrant label positioning */
  labels: {
    bottomRadius: 114,
    fontSize: 8,
    topRadius: 105,
  },

  /** Size of the SVG viewBox */
  viewBox: {
    height: 200,
    max: 100,
    min: -100,
    width: 200,
  },
} as const

/**
 * Quadrant angle definitions (in radians)
 */
export const QUADRANT_ANGLES: QuadrantAnglesType = {
  buildTools: {
    end: Math.PI,
    start: Math.PI / 2,
  },
  frameworks: {
    end: 2 * Math.PI,
    start: (3 * Math.PI) / 2,
  },
  infrastructure: {
    end: Math.PI / 2,
    start: 0,
  },
  languages: {
    end: (3 * Math.PI) / 2,
    start: Math.PI,
  },
} as const

/**
 * Quadrant visual styling
 */
export const QUADRANT_STYLES: QuadrantStylesType = {
  buildTools: {
    blipColor: 'fill-slate-500 stroke-slate-100',
    labelColor: 'fill-slate-500',
  },
  frameworks: {
    blipColor: 'fill-sky-500 stroke-sky-100',
    labelColor: 'fill-sky-500',
  },
  infrastructure: {
    blipColor: 'fill-violet-500 stroke-violet-100',
    labelColor: 'fill-violet-500',
  },
  languages: {
    blipColor: 'fill-primary stroke-primary-foreground',
    labelColor: 'fill-primary',
  },
} as const

/**
 * SVG path definitions for curved text
 */
const numberToString: (number: number) => string = String

export const LABEL_PATHS: LabelPathsType = {
  buildTools: `M -${numberToString(RADAR_CONFIG.labels.bottomRadius)},0 A ${numberToString(RADAR_CONFIG.labels.bottomRadius)},${numberToString(RADAR_CONFIG.labels.bottomRadius)} 0 0,0 0,${numberToString(RADAR_CONFIG.labels.bottomRadius)}`,
  frameworks: `M 0,-${numberToString(RADAR_CONFIG.labels.topRadius)} A ${numberToString(RADAR_CONFIG.labels.topRadius)},${numberToString(RADAR_CONFIG.labels.topRadius)} 0 0,1 ${numberToString(RADAR_CONFIG.labels.topRadius)},0`,
  infrastructure: `M 0,${numberToString(RADAR_CONFIG.labels.bottomRadius)} A ${numberToString(RADAR_CONFIG.labels.bottomRadius)},${numberToString(RADAR_CONFIG.labels.bottomRadius)} 0 0,0 ${numberToString(RADAR_CONFIG.labels.bottomRadius)},0`,
  languages: `M -${numberToString(RADAR_CONFIG.labels.topRadius)},0 A ${numberToString(RADAR_CONFIG.labels.topRadius)},${numberToString(RADAR_CONFIG.labels.topRadius)} 0 0,1 0,-${numberToString(RADAR_CONFIG.labels.topRadius)}`,
} as const
