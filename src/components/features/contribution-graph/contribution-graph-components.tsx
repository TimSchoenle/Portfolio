import { type JSX } from 'react'

import { type Locale } from 'next-intl'

import {
  type MonthLabel,
  type WeekModel,
} from '@/lib/github/contribution-calendar'
import {
  CONTRIBUTION_LEVELS,
  type ContributionLevel,
  type ContributionPoint,
} from '@/models/github'
import type { FCStrict } from '@/types/fc'

import styles from './contribution-graph.module.css'

/* =============================== Subviews =============================== */

export const levelClass: (level: ContributionLevel) => string = (
  level: ContributionLevel
): string => {
  if (level <= 0) {
    return 'bg-muted/40 dark:bg-muted/20 hover:bg-muted/60 dark:hover:bg-muted/40 border border-border/50 dark:border-border/30'
  }
  if (level === 1) {
    return 'bg-primary/20 dark:bg-primary/20 hover:bg-primary/30 dark:hover:bg-primary/30 border border-primary/20 dark:border-primary/20'
  }
  if (level === 2) {
    return 'bg-primary/40 dark:bg-primary/40 hover:bg-primary/50 dark:hover:bg-primary/50 border border-primary/30 dark:border-primary/30'
  }
  if (level === 3) {
    return 'bg-primary/60 dark:bg-primary/60 hover:bg-primary/70 dark:hover:bg-primary/70 border border-primary/40 dark:border-primary/40'
  }
  return 'bg-primary/80 dark:bg-primary/80 hover:bg-primary/90 dark:hover:bg-primary/90 border border-primary/50 dark:border-primary/50'
}

interface LegendSquareProperties {
  readonly level: ContributionLevel
}

const LegendSquare: FCStrict<LegendSquareProperties> = ({
  level,
}: LegendSquareProperties): JSX.Element => {
  const cls: string = levelClass(level).split(' ').slice(0, 3).join(' ')
  return <div className={`h-4 w-4 rounded-sm ${cls}`} />
}

interface LegendBarProperties {
  readonly less: string
  readonly more: string
}

export const LegendBar: FCStrict<LegendBarProperties> = ({
  less,
  more,
}: LegendBarProperties): JSX.Element => {
  return (
    <div className="flex items-center gap-2 text-xs text-muted-foreground">
      <span>{less}</span>
      {CONTRIBUTION_LEVELS.map(
        (lvl: ContributionLevel): JSX.Element => (
          <LegendSquare key={`lg-${String(lvl)}`} level={lvl} />
        )
      )}
      <span>{more}</span>
    </div>
  )
}

interface MonthLabelsRowProperties {
  readonly labels: readonly MonthLabel[]
}

export const MonthLabelsRow: FCStrict<MonthLabelsRowProperties> = ({
  labels,
}: MonthLabelsRowProperties): JSX.Element => {
  return (
    <div className="relative mb-3 h-5">
      {labels.map(
        (monthLabel: MonthLabel): JSX.Element => (
          <div
            className="absolute text-sm font-medium text-muted-foreground"
            key={`${monthLabel.month}-${String(monthLabel.weekIndex)}`}
            style={{ left: String(48 + monthLabel.weekIndex * 20) + 'px' }}
          >
            {monthLabel.month}
          </div>
        )
      )}
    </div>
  )
}

// one-time module constant (tree-shakable)
const WEEKDAY_INDICES: readonly [0, 1, 2, 3, 4, 5, 6] = [
  0, 1, 2, 3, 4, 5, 6,
] as const

interface WeekdayLabelColProperties {
  readonly dayFive: string
  readonly dayOne: string
  readonly dayThree: string
}

export const WeekdayLabelCol: FCStrict<WeekdayLabelColProperties> = ({
  dayFive,
  dayOne,
  dayThree,
}: WeekdayLabelColProperties): JSX.Element => {
  return (
    <div className="flex flex-shrink-0 flex-col gap-1 pr-3">
      {WEEKDAY_INDICES.map(
        (index: (typeof WEEKDAY_INDICES)[number]): JSX.Element => {
          let label: string

          switch (index) {
            case 1: {
              label = dayOne
              break
            }
            case 3: {
              label = dayThree
              break
            }
            case 5: {
              label = dayFive
              break
            }
            default: {
              label = ''
            }
          }

          return (
            <div
              className="flex h-4 w-8 items-center text-xs font-medium text-muted-foreground"
              key={`wd-${String(index)}`}
            >
              {label}
            </div>
          )
        }
      )}
    </div>
  )
}

interface DayCellProperties {
  readonly day: ContributionPoint
  readonly dayIndex: number
  readonly locale: Locale
  readonly weekIndex: number
}

const DayCell: FCStrict<DayCellProperties> = ({
  day,
  dayIndex,
  locale,
  weekIndex,
}: DayCellProperties): JSX.Element => {
  const delayMs: string = String(weekIndex * 7 + dayIndex) + 'ms'

  const commitsText: string = day.count.toLocaleString(locale)

  return (
    <div
      className={`h-4 w-4 rounded-sm transition-all duration-200 ${levelClass(day.level)} ${styles['cell'] ?? ''}`}
      data-commits={commitsText}
      data-date={day.date}
      data-row={dayIndex}
      key={day.date}
      style={{ animationDelay: delayMs }}
    />
  )
}

const EmptyCell: FCStrict = (): JSX.Element => (
  <div className="h-4 w-4 rounded-sm border border-border/30 bg-muted/20 dark:border-border/20 dark:bg-muted/10" />
)

interface WeekColumnProperties {
  readonly locale: Locale
  readonly week: WeekModel
  readonly weekIndex: number
}

export const WeekColumn: FCStrict<WeekColumnProperties> = ({
  locale,
  week,
  weekIndex,
}: WeekColumnProperties): JSX.Element => {
  return (
    <div className="flex flex-shrink-0 flex-col gap-1">
      {week.days.map(
        (
          contributionPoint: ContributionPoint | null,
          dayIndex: number
        ): JSX.Element =>
          contributionPoint === null ? (
            <EmptyCell key={`${week.key}-${String(dayIndex)}`} />
          ) : (
            <DayCell
              day={contributionPoint}
              dayIndex={dayIndex}
              key={contributionPoint.date}
              locale={locale}
              weekIndex={weekIndex}
            />
          )
      )}
    </div>
  )
}

interface WeeksGridProperties {
  readonly dayFive: string
  readonly dayOne: string
  readonly dayThree: string
  readonly locale: Locale
  readonly weeks: readonly WeekModel[]
}

export const WeeksGrid: FCStrict<WeeksGridProperties> = ({
  dayFive,
  dayOne,
  dayThree,
  locale,
  weeks,
}: WeeksGridProperties): JSX.Element => {
  return (
    <div className="flex w-full justify-between gap-1">
      <WeekdayLabelCol dayFive={dayFive} dayOne={dayOne} dayThree={dayThree} />
      <div className="flex flex-1 justify-between gap-1">
        {weeks.map(
          (weekModel: WeekModel, index: number): JSX.Element => (
            <WeekColumn
              key={weekModel.key}
              locale={locale}
              week={weekModel}
              weekIndex={index}
            />
          )
        )}
      </div>
    </div>
  )
}
