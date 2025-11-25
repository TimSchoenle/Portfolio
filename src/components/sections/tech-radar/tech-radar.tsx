'use server'

import { type JSX } from 'react'

import type { Locale } from 'next-intl'

import { getTranslations } from 'next-intl/server'

import styles from '@/components/sections/tech-radar/tech-radar.module.css'
import { type Skill } from '@/lib/config'
import { calculateBlipPosition } from '@/lib/tech-radar-utilities'
import type { AsyncPageFC, FCStrict } from '@/types/fc'
import type { Translations } from '@/types/i18n'
import type {
  Blip,
  CalculateBlipPositionResult,
  QuadrantAngleType,
  RadarConfigType,
  TechRadarQuadrant,
} from '@/types/tech-radar'

import { HoverProvider } from './hover-context'
import {
  LABEL_PATHS,
  QUADRANT_ANGLES,
  QUADRANT_STYLES,
  RADAR_CONFIG,
} from './radar-config'
import { TechRadarInteractive } from './tech-radar-interactive'
import { TechRadarTooltip } from './tech-radar-tooltip'

interface TechRadarProperties {
  readonly buildTools: readonly Skill[]
  readonly frameworks: readonly Skill[]
  readonly infrastructure: readonly Skill[]
  readonly languages: readonly Skill[]
  readonly locale: Locale
}

const generateBlipsForCategory: (
  items: readonly Skill[],
  quadrantKey: TechRadarQuadrant,
  config: QuadrantAngleType
) => Blip[] = (
  items: readonly Skill[],
  quadrantKey: TechRadarQuadrant,
  config: QuadrantAngleType
): Blip[] => {
  return items.map((skill: Skill, index: number): Blip => {
    const {
      angle,
      radius,
      xCoordinate,
      yCoordinate,
    }: CalculateBlipPositionResult = calculateBlipPosition({
      confidence: skill.confidence,
      endAngle: config.end,
      index,
      skillName: skill.name,
      startAngle: config.start,
      total: items.length,
    })
    return {
      angle,
      iconName: skill.name,
      id: `${quadrantKey}-${skill.name}`,
      name: skill.name,
      quadrant: quadrantKey,
      radius,
      xCoordinate,
      yCoordinate,
    }
  })
}

interface RadarBackgroundProperties {
  readonly circles: RadarConfigType['circles']
}

/**
 * Server component rendering the static background circles
 */
const RadarBackground: FCStrict<RadarBackgroundProperties> = ({
  circles,
}: RadarBackgroundProperties): JSX.Element => {
  return (
    <>
      {/* Background circles */}
      <circle
        className="fill-muted/20 stroke-border stroke-1"
        cx="0"
        cy="0"
        r={circles.inner}
      />
      <circle
        className="fill-muted/10 stroke-border stroke-1"
        cx="0"
        cy="0"
        r={circles.middle}
      />
      <circle
        className="fill-transparent stroke-border stroke-1"
        cx="0"
        cy="0"
        r={circles.outer}
      />

      {/* Sonar sweep animation */}
      <g className={styles['radarSpin']}>
        <path
          className="fill-primary/20"
          d="M 0 0 L 100 0 A 100 100 0 0 1 70.7 70.7 Z"
        />
      </g>

      {/* Axes - clipped to circle */}
      <g clipPath="url(#radarClip)">
        <line
          className="stroke-border/50 stroke-1"
          x1="-100"
          x2="100"
          y1="0"
          y2="0"
        />
        <line
          className="stroke-border/50 stroke-1"
          x1="0"
          x2="0"
          y1="-100"
          y2="100"
        />
      </g>
    </>
  )
}

interface RadarDefsProperties {
  readonly circles: RadarConfigType['circles']
}

/**
 * Server component rendering SVG definitions (clip paths and text paths)
 */
const RadarDefs: FCStrict<RadarDefsProperties> = ({
  circles,
}: RadarDefsProperties): JSX.Element => {
  return (
    <defs>
      <clipPath id="radarClip">
        <circle cx="0" cy="0" r={circles.outer} />
      </clipPath>

      {/* Curved text paths for quadrant labels */}
      <path d={LABEL_PATHS.languages} fill="none" id="languagesPath" />
      <path d={LABEL_PATHS.frameworks} fill="none" id="frameworksPath" />
      <path d={LABEL_PATHS.buildTools} fill="none" id="buildToolsPath" />
      <path
        d={LABEL_PATHS.infrastructure}
        fill="none"
        id="infrastructurePath"
      />
    </defs>
  )
}

interface RadarLabelsProperties {
  readonly labels: RadarConfigType['labels']
  readonly translations: Translations<'skills'>
}

/**
 * Server component rendering curved quadrant labels
 */
const RadarLabels: FCStrict<RadarLabelsProperties> = ({
  labels,
  translations,
}: RadarLabelsProperties): JSX.Element => {
  return (
    <>
      {/* Curved Quadrant Labels - Top quadrants */}
      <text
        className={`font-bold tracking-wider uppercase ${QUADRANT_STYLES.languages.labelColor}`}
        style={{ fontSize: `${String(labels.fontSize)}px` }}
      >
        <textPath href="#languagesPath" startOffset="50%" textAnchor="middle">
          {translations('languages')}
        </textPath>
      </text>
      <text
        className={`font-bold tracking-wider uppercase ${QUADRANT_STYLES.frameworks.labelColor}`}
        style={{ fontSize: `${String(labels.fontSize)}px` }}
      >
        <textPath href="#frameworksPath" startOffset="50%" textAnchor="middle">
          {translations('frameworks')}
        </textPath>
      </text>

      {/* Curved Quadrant Labels - Bottom quadrants */}
      <text
        className={`font-bold tracking-wider uppercase ${QUADRANT_STYLES.buildTools.labelColor}`}
        style={{ fontSize: `${String(labels.fontSize)}px` }}
      >
        <textPath href="#buildToolsPath" startOffset="50%" textAnchor="middle">
          {translations('buildTools')}
        </textPath>
      </text>
      <text
        className={`font-bold tracking-wider uppercase ${QUADRANT_STYLES.infrastructure.labelColor}`}
        style={{ fontSize: `${String(labels.fontSize)}px` }}
      >
        <textPath
          href="#infrastructurePath"
          startOffset="50%"
          textAnchor="middle"
        >
          {translations('infrastructure')}
        </textPath>
      </text>
    </>
  )
}

export const TechRadar: AsyncPageFC<TechRadarProperties> = async ({
  buildTools,
  frameworks,
  infrastructure,
  languages,
  locale,
}: TechRadarProperties): Promise<JSX.Element> => {
  // Fetch translations on server
  const translations: Translations<'skills'> = await getTranslations({
    locale,
    namespace: 'skills',
  })

  const quadrants: Record<TechRadarQuadrant, QuadrantAngleType> = {
    buildTools: QUADRANT_ANGLES.buildTools,
    frameworks: QUADRANT_ANGLES.frameworks,
    infrastructure: QUADRANT_ANGLES.infrastructure,
    languages: QUADRANT_ANGLES.languages,
  }

  const allBlips: Blip[] = [
    ...generateBlipsForCategory(languages, 'languages', quadrants.languages),
    ...generateBlipsForCategory(frameworks, 'frameworks', quadrants.frameworks),
    ...generateBlipsForCategory(buildTools, 'buildTools', quadrants.buildTools),
    ...generateBlipsForCategory(
      infrastructure,
      'infrastructure',
      quadrants.infrastructure
    ),
  ]

  const { circles, labels, viewBox }: typeof RADAR_CONFIG = RADAR_CONFIG

  return (
    <HoverProvider blips={allBlips}>
      <div className="relative mx-auto aspect-square w-full max-w-2xl">
        <svg
          className="h-full w-full overflow-visible"
          viewBox={`${String(viewBox.min)} ${String(viewBox.min)} ${String(viewBox.width)} ${String(viewBox.height)}`}
        >
          <RadarDefs circles={circles} />
          <RadarBackground circles={circles} />
          <RadarLabels labels={labels} translations={translations} />

          {/* Interactive blips - client component */}
          <TechRadarInteractive blips={allBlips} />
        </svg>

        {/* Tooltip - rendered outside SVG for proper positioning */}
        <TechRadarTooltip blips={allBlips} />

        {/* Center hub */}
        <div className="pointer-events-none absolute inset-0 flex items-center justify-center">
          <div className="h-2 w-2 animate-pulse rounded-full bg-primary" />
        </div>
      </div>
    </HoverProvider>
  )
}
