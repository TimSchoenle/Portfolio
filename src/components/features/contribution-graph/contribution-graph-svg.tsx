'use client'

import React, {
  type CSSProperties,
  type Dispatch,
  type JSX,
  type RefObject,
  type SetStateAction,
  useEffect,
  useRef,
  useState,
} from 'react'

import { type Locale, useFormatter, useTranslations } from 'next-intl'

import { createPortal } from 'react-dom'

import { type WeekModel } from '@/lib/github/contribution-calendar'
import type { ContributionPoint } from '@/models/github'
import type { FCStrict } from '@/types/fc'
import type { Translations } from '@/types/i18n'

import styles from './contribution-graph.module.css'

interface ContributionGraphSvgProperties {
  readonly dayFive: string
  readonly dayOne: string
  readonly dayThree: string
  readonly locale: Locale
  readonly weeks: readonly WeekModel[]
}

interface HoveredData {
  count: number
  date: string
  xCoord: number
  yCoord: number
}

interface TooltipStyleProperties {
  xCoord: number
  yCoord: number
}

const CONSTANTS: {
  CELL_BORDER_RADIUS: number
  CELL_GAP: number
  CELL_SIZE: number
  LABEL_FONT_SIZE: number
  LABEL_PADDING_X: number
  LABEL_PADDING_Y: number
  WEEK_LABEL_WIDTH: number
} = {
  CELL_BORDER_RADIUS: 2,
  CELL_GAP: 3,
  CELL_SIZE: 11,
  LABEL_FONT_SIZE: 10,
  LABEL_PADDING_X: 10,
  LABEL_PADDING_Y: 20,
  WEEK_LABEL_WIDTH: 30,
} as const

// Helper to render month labels
const renderMonthLabels: (
  weeks: readonly WeekModel[],
  locale: Locale
) => JSX.Element[] = (
  weeks: readonly WeekModel[],
  locale: Locale
): JSX.Element[] => {
  const monthLabels: JSX.Element[] = []
  let lastMonthLabelIndex: number = -1

  for (const [index, week] of weeks.entries()) {
    const firstDay: ContributionPoint | null | undefined = week.days.find(
      (day: ContributionPoint | null): boolean => day !== null
    )
    if (!firstDay) {
      continue
    }

    const date: Date = new Date(firstDay.date)
    const month: number = date.getMonth()

    if (month !== lastMonthLabelIndex) {
      const xPosition: number =
        CONSTANTS.WEEK_LABEL_WIDTH +
        index * (CONSTANTS.CELL_SIZE + CONSTANTS.CELL_GAP)
      monthLabels.push(
        <text
          className="fill-muted-foreground text-[10px]"
          dominantBaseline="hanging"
          key={`month-${String(index)}`}
          x={xPosition}
          y={0}
        >
          {date.toLocaleDateString(locale, { month: 'short' })}
        </text>
      )
      lastMonthLabelIndex = month
    }
  }
  return monthLabels
}

// 3-Zone Tooltip Helper for Portal (Fixed Position)
const getTooltipStyle: (
  properties: TooltipStyleProperties
) => CSSProperties = ({
  xCoord,
  yCoord,
}: TooltipStyleProperties): CSSProperties => {
  const viewportWidth: number =
    typeof window === 'undefined' ? 1000 : window.innerWidth
  const isLeft: boolean = xCoord < 80
  const isRight: boolean = xCoord > viewportWidth - 80
  const isTop: boolean = yCoord < 80 // Threshold for top flip

  let xTranslate: string = '-50%'
  if (isLeft) {
    xTranslate = '-15%'
  } else if (isRight) {
    xTranslate = '-85%'
  }

  const yTranslate: string = isTop ? '20px' : '-100%'

  return {
    left: `${String(xCoord)}px`,
    position: 'fixed',
    top: `${String(yCoord)}px`,
    transform: `translate(${xTranslate}, ${yTranslate})`,
    zIndex: 9999, // Ensure it's on top of everything
  }
}

const getTooltipArrowStyle: (
  xCoord: number,
  yCoord: number
) => CSSProperties = (xCoord: number, yCoord: number): CSSProperties => {
  const viewportWidth: number =
    typeof window === 'undefined' ? 1000 : window.innerWidth
  const isLeft: boolean = xCoord < 80
  const isRight: boolean = xCoord > viewportWidth - 80
  const isTop: boolean = yCoord < 80

  const arrowStyle: CSSProperties = {
    bottom: isTop ? 'auto' : '-4px',
    left: '50%',
    position: 'absolute',
    top: isTop ? '-4px' : 'auto',
    transform: isTop
      ? 'translateX(-50%) rotate(225deg)'
      : 'translateX(-50%) rotate(45deg)',
  }

  if (isLeft) {
    arrowStyle.left = '15%'
  } else if (isRight) {
    arrowStyle.left = '85%'
  }

  return arrowStyle
}

interface TooltipReferenceProperties {
  data: HoveredData
}

// Extracted Tooltip Component
const Tooltip: FCStrict<TooltipReferenceProperties> = ({
  data,
}: TooltipReferenceProperties): JSX.Element => {
  // Cast to internal interfaces to avoid 'any'
  const translate: Translations<'projects.contributions'> = useTranslations(
    'projects.contributions'
  )
  const formatter: ReturnType<typeof useFormatter> = useFormatter()

  return (
    <div
      className={styles['tooltipContainer'] ?? ''}
      style={getTooltipStyle({
        xCoord: data.xCoord,
        yCoord: data.yCoord,
      })}
    >
      <div className={styles['tooltipContent'] ?? ''}>
        <span className={styles['tooltipCount'] ?? ''}>
          {translate('tooltip', { count: data.count })}
        </span>
        <span className={styles['tooltipDate'] ?? ''}>
          {formatter.dateTime(new Date(data.date), {
            dateStyle: 'medium',
          })}
        </span>
      </div>
      <div
        className={styles['tooltipArrow'] ?? ''}
        style={getTooltipArrowStyle(data.xCoord, data.yCoord)}
      />
    </div>
  )
}

// Extracted renderGrid function
const renderGrid: (
  weeks: readonly WeekModel[],
  onHover: (data: HoveredData | null, event: React.MouseEvent) => void
) => JSX.Element = (
  weeks: readonly WeekModel[],
  onHover: (data: HoveredData | null, event: React.MouseEvent) => void
): JSX.Element => (
  <g
    transform={`translate(${String(CONSTANTS.WEEK_LABEL_WIDTH)}, ${String(CONSTANTS.LABEL_PADDING_Y)})`}
  >
    {weeks.map(
      (week: WeekModel, wIndex: number): JSX.Element => (
        <g
          key={week.key}
          transform={`translate(${String(
            wIndex * (CONSTANTS.CELL_SIZE + CONSTANTS.CELL_GAP)
          )}, 0)`}
        >
          {week.days.map(
            (
              day: ContributionPoint | null,
              dIndex: number
            ): JSX.Element | null => {
              if (day) {
                const yPos: number =
                  dIndex * (CONSTANTS.CELL_SIZE + CONSTANTS.CELL_GAP)
                const levelClass: string | undefined =
                  styles[`level${String(day.level)}`]

                return (
                  <rect
                    className={`${styles['rect'] ?? ''} ${levelClass ?? ''}`}
                    height={CONSTANTS.CELL_SIZE}
                    key={day.date}
                    rx={CONSTANTS.CELL_BORDER_RADIUS}
                    ry={CONSTANTS.CELL_BORDER_RADIUS}
                    width={CONSTANTS.CELL_SIZE}
                    x={0}
                    y={yPos}
                    onMouseEnter={(event: React.MouseEvent): void => {
                      onHover(
                        {
                          count: day.count,
                          date: day.date,
                          xCoord: 0,
                          yCoord: 0,
                        },
                        event
                      )
                    }}
                  />
                )
              }
              return null
            }
          )}
        </g>
      )
    )}
  </g>
)

// eslint-disable-next-line max-lines-per-function
export const ContributionGraphSvg: FCStrict<ContributionGraphSvgProperties> = ({
  dayFive,
  dayOne,
  dayThree,
  locale,
  weeks,
}: ContributionGraphSvgProperties): JSX.Element => {
  const containerReference: RefObject<HTMLDivElement | null> =
    useRef<HTMLDivElement>(null)
  const [hoveredData, setHoveredData]: [
    HoveredData | null,
    Dispatch<SetStateAction<HoveredData | null>>,
  ] = useState<HoveredData | null>(null)
  const [portalContainer, setPortalContainer]: [
    HTMLElement | null,
    Dispatch<SetStateAction<HTMLElement | null>>,
  ] = useState<HTMLElement | null>(null)

  useEffect((): void => {
    setPortalContainer(document.body)
  }, [])

  const handleHover: (
    data: HoveredData | null,
    event?: React.MouseEvent
  ) => void = (data: HoveredData | null, event?: React.MouseEvent): void => {
    if (data && event) {
      const rect: DOMRect = event.currentTarget.getBoundingClientRect()
      // Calculate center of the cell in viewport coordinates
      const viewportX: number = rect.left + rect.width / 2
      const viewportY: number = rect.top

      setHoveredData({
        ...data,
        xCoord: viewportX,
        yCoord: viewportY,
      })
    } else {
      setHoveredData(null)
    }
  }

  // 1. Calculate Dimensions
  const weekCount: number = weeks.length
  // 7 days + gaps
  const graphHeight: number =
    7 * (CONSTANTS.CELL_SIZE + CONSTANTS.CELL_GAP) - CONSTANTS.CELL_GAP
  const graphWidth: number =
    weekCount * (CONSTANTS.CELL_SIZE + CONSTANTS.CELL_GAP) - CONSTANTS.CELL_GAP

  const totalHeight: number = graphHeight + CONSTANTS.LABEL_PADDING_Y
  const totalWidth: number = graphWidth + CONSTANTS.WEEK_LABEL_WIDTH

  // Add padding to viewBox to prevent layout shift from strokes
  const padding: number = 4
  const viewBox: string = `${String(-padding / 2)} ${String(-padding / 2)} ${String(totalWidth + padding)} ${String(totalHeight + padding)}`

  // 2. Render content
  const monthLabels: JSX.Element[] = renderMonthLabels(weeks, locale)

  // 3. Main Render
  return (
    <div className={styles['graphWrapper'] ?? ''} ref={containerReference}>
      <div className={styles['graphContainer'] ?? ''}>
        <svg
          className="h-auto w-full overflow-visible"
          preserveAspectRatio="xMidYMid meet"
          viewBox={viewBox}
          onMouseLeave={(): void => {
            setHoveredData(null)
          }}
        >
          {/* Month Labels */}
          {monthLabels}

          {/* Weekday Labels (Mon, Wed, Fri) */}
          <text
            className="fill-muted-foreground text-[10px]"
            dominantBaseline="middle"
            x={0}
            y={
              CONSTANTS.LABEL_PADDING_Y +
              (CONSTANTS.CELL_SIZE + CONSTANTS.CELL_GAP) +
              CONSTANTS.CELL_SIZE / 2
            }
          >
            {dayOne}
          </text>
          <text
            className="fill-muted-foreground text-[10px]"
            dominantBaseline="middle"
            x={0}
            y={
              CONSTANTS.LABEL_PADDING_Y +
              3 * (CONSTANTS.CELL_SIZE + CONSTANTS.CELL_GAP) +
              CONSTANTS.CELL_SIZE / 2
            }
          >
            {dayThree}
          </text>
          <text
            className="fill-muted-foreground text-[10px]"
            dominantBaseline="middle"
            x={0}
            y={
              CONSTANTS.LABEL_PADDING_Y +
              5 * (CONSTANTS.CELL_SIZE + CONSTANTS.CELL_GAP) +
              CONSTANTS.CELL_SIZE / 2
            }
          >
            {dayFive}
          </text>

          {/* Grid */}
          {renderGrid(weeks, handleHover)}
        </svg>

        {/* Portal Tooltip Overlay - delegated to sub-component */}
        {hoveredData &&
          portalContainer &&
          createPortal(<Tooltip data={hoveredData} />, portalContainer)}
      </div>
    </div>
  )
}
