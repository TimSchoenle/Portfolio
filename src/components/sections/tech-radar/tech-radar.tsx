'use server'

import { type JSX } from 'react'

import type { Locale } from 'next-intl'

import { getTranslations } from 'next-intl/server'

import styles from '@/components/sections/tech-radar/tech-radar.module.css'
import { type Skill } from '@/lib/config'
import {
  LABEL_PATHS,
  QUADRANT_ANGLES,
  QUADRANT_STYLES,
  RADAR_CONFIG,
} from '@/lib/radar-config'
import {
  calculateBlipPosition,
  resolveBlipCollisions,
} from '@/lib/tech-radar-utilities'
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
import { TechRadarInteractive } from './tech-radar-interactive'
import { TechRadarTooltip } from './tech-radar-tooltip'

const quadrantToPathId: (quadrant: TechRadarQuadrant) => string = (
  quadrant: TechRadarQuadrant
): string => {
  return `${quadrant}Path`
}

interface TechRadarProperties {
  readonly buildTools: readonly Skill[]
  readonly frameworks: readonly Skill[]
  readonly infrastructure: readonly Skill[]
  readonly languages: readonly Skill[]
  readonly locale: Locale
}

interface GenerateBlipsForCategoryParameters {
  readonly config: QuadrantAngleType
  readonly items: readonly Skill[]
  readonly quadrantKey: TechRadarQuadrant
}

const generateBlipsForCategory: (
  parameter: GenerateBlipsForCategoryParameters
) => Blip[] = ({
  config,
  items,
  quadrantKey,
}: GenerateBlipsForCategoryParameters): Blip[] => {
  // 1. Calculate initial positions
  const initialBlips: Blip[] = items.map(
    (skill: Skill, index: number): Blip => {
      const position: CalculateBlipPositionResult = calculateBlipPosition({
        confidence: skill.confidence,
        endAngle: config.end,
        index,
        skillName: skill.name,
        startAngle: config.start,
        total: items.length,
      })
      return {
        ...position,
        iconName: skill.name,
        id: `${quadrantKey}-${skill.name}`,
        name: skill.name,
        quadrant: quadrantKey,
      }
    }
  )

  // 2. Resolve collisions
  const resolvedPositions: CalculateBlipPositionResult[] =
    resolveBlipCollisions({
      blips: initialBlips,
      endAngle: config.end,
      startAngle: config.start,
    })

  // 3. Merge resolved positions back into blips
  return initialBlips.map((blip: Blip, index: number): Blip => {
    const resolved: CalculateBlipPositionResult | undefined =
      resolvedPositions.at(index)

    if (!resolved) {
      // This should never happen since resolvedPositions has same length as initialBlips
      // but we need to handle it for type safety
      return blip
    }

    return {
      ...blip,
      angle: resolved.angle,
      radius: resolved.radius,
      xCoordinate: resolved.xCoordinate,
      yCoordinate: resolved.yCoordinate,
    }
  })
}

interface RadarBackgroundProperties {
  readonly animation: RadarConfigType['animation']
  readonly circles: RadarConfigType['circles']
  readonly viewBox: RadarConfigType['viewBox']
}

const RadarBackground: FCStrict<RadarBackgroundProperties> = ({
  animation,
  circles,
  viewBox,
}: RadarBackgroundProperties): JSX.Element => {
  const sweepRadius: number = animation.sonarSweepRadius
  const sweepAngle: number = (animation.sonarSweepAngle * Math.PI) / 180
  const sweepEndX: number = Math.cos(sweepAngle) * sweepRadius
  const sweepEndY: number = Math.sin(sweepAngle) * sweepRadius

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
          d={`M 0 0 L ${String(sweepRadius)} 0 A ${String(sweepRadius)} ${String(sweepRadius)} 0 0 1 ${String(sweepEndX)} ${String(sweepEndY)} Z`}
        />
      </g>

      {/* Axes - clipped to circle */}
      <g clipPath="url(#radarClip)">
        <line
          className="stroke-border/50 stroke-1"
          x1={String(viewBox.min)}
          x2={String(viewBox.max)}
          y1="0"
          y2="0"
        />
        <line
          className="stroke-border/50 stroke-1"
          x1="0"
          x2="0"
          y1={String(viewBox.min)}
          y2={String(viewBox.max)}
        />
      </g>
    </>
  )
}

interface RadarDefsProperties {
  readonly circles: RadarConfigType['circles']
}

const RadarDefs: FCStrict<RadarDefsProperties> = ({
  circles,
}: RadarDefsProperties): JSX.Element => {
  return (
    <defs>
      <clipPath id="radarClip">
        <circle cx="0" cy="0" r={circles.outer} />
      </clipPath>

      {/* Curved text paths for quadrant labels */}
      <path
        d={LABEL_PATHS.languages}
        fill="none"
        id={quadrantToPathId('languages')}
      />
      <path
        d={LABEL_PATHS.frameworks}
        fill="none"
        id={quadrantToPathId('frameworks')}
      />
      <path
        d={LABEL_PATHS.buildTools}
        fill="none"
        id={quadrantToPathId('buildTools')}
      />
      <path
        d={LABEL_PATHS.infrastructure}
        fill="none"
        id={quadrantToPathId('infrastructure')}
      />
    </defs>
  )
}

interface RadarLabelProperties {
  readonly color: string
  readonly fontSize: number
  readonly label: string
  readonly quadrant: TechRadarQuadrant
}

const RadarLabel: FCStrict<RadarLabelProperties> = ({
  color,
  fontSize,
  label,
  quadrant,
}: RadarLabelProperties): JSX.Element => {
  return (
    <text
      className={`font-bold tracking-wider uppercase ${color}`}
      style={{ fontSize: `${String(fontSize)}px` }}
    >
      <textPath
        href={`#${quadrantToPathId(quadrant)}`}
        startOffset="50%"
        textAnchor="middle"
      >
        {label}
      </textPath>
    </text>
  )
}

interface RadarLabelsProperties {
  readonly labels: RadarConfigType['labels']
  readonly translations: Translations<'skills'>
}

const RadarLabels: FCStrict<RadarLabelsProperties> = ({
  labels,
  translations,
}: RadarLabelsProperties): JSX.Element => {
  return (
    <>
      {/* Curved Quadrant Labels - Top quadrants */}
      <RadarLabel
        color={QUADRANT_STYLES.languages.labelColor}
        fontSize={labels.fontSize}
        label={translations('languages')}
        quadrant="languages"
      />
      <RadarLabel
        color={QUADRANT_STYLES.frameworks.labelColor}
        fontSize={labels.fontSize}
        label={translations('frameworks')}
        quadrant="frameworks"
      />

      {/* Curved Quadrant Labels - Bottom quadrants */}
      <RadarLabel
        color={QUADRANT_STYLES.buildTools.labelColor}
        fontSize={labels.fontSize}
        label={translations('buildTools')}
        quadrant="buildTools"
      />
      <RadarLabel
        color={QUADRANT_STYLES.infrastructure.labelColor}
        fontSize={labels.fontSize}
        label={translations('infrastructure')}
        quadrant="infrastructure"
      />
    </>
  )
}

// eslint-disable-next-line max-lines-per-function
export const TechRadar: AsyncPageFC<TechRadarProperties> = async ({
  buildTools,
  frameworks,
  infrastructure,
  languages,
  locale,
}: TechRadarProperties): Promise<JSX.Element> => {
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
    ...generateBlipsForCategory({
      config: quadrants.languages,
      items: languages,
      quadrantKey: 'languages',
    }),
    ...generateBlipsForCategory({
      config: quadrants.frameworks,
      items: frameworks,
      quadrantKey: 'frameworks',
    }),
    ...generateBlipsForCategory({
      config: quadrants.buildTools,
      items: buildTools,
      quadrantKey: 'buildTools',
    }),
    ...generateBlipsForCategory({
      config: quadrants.infrastructure,
      items: infrastructure,
      quadrantKey: 'infrastructure',
    }),
  ]

  const { animation, circles, labels, viewBox }: typeof RADAR_CONFIG =
    RADAR_CONFIG

  return (
    <HoverProvider blips={allBlips}>
      <div className="relative mx-auto aspect-square w-full max-w-2xl">
        <svg
          className="h-full w-full overflow-visible"
          viewBox={`${String(viewBox.min)} ${String(viewBox.min)} ${String(viewBox.width)} ${String(viewBox.height)}`}
        >
          <RadarDefs circles={circles} />
          <RadarBackground
            animation={animation}
            circles={circles}
            viewBox={viewBox}
          />
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
