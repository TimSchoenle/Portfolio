/* eslint-disable max-lines */
'use client'

import React, {
  type Dispatch,
  type JSX,
  type SetStateAction,
  useMemo,
  useState,
} from 'react'

import { type Locale, useTranslations } from 'next-intl'

import { Card } from '@/components/ui/card'
import { panic } from '@/lib/utilities'
import type { FCStrict } from '@/types/fc'
import {
  CONTRIBUTION_LEVELS,
  type ContributionLevel,
  type ContributionPoint,
} from '@/types/github'
import type { LocalePageProperties, Translations } from '@/types/i18n'

/* =============================== Types =============================== */

interface ContributionGraphProperties extends LocalePageProperties {
  readonly data: readonly ContributionPoint[]
}

interface MousePos {
  readonly x: number
  readonly y: number
}

interface MonthLabel {
  readonly month: string
  readonly weekIndex: number
}

interface WeekModel {
  readonly days: readonly (ContributionPoint | null)[]
  readonly key: string
}

interface CalendarModel {
  readonly monthLabels: readonly MonthLabel[]
  readonly weeks: readonly WeekModel[]
}

/* ============================ Pure Helpers =========================== */

const levelClass: (level: ContributionLevel) => string = (
  level: ContributionLevel
): string => {
  if (level <= 0) {
    return 'bg-muted/40 dark:bg-muted/20 hover:bg-muted/60 dark:hover:bg-muted/40 border border-border/50 dark:border-border/30'
  }
  if (level === 1) {
    return 'bg-emerald-100 dark:bg-emerald-900/40 hover:bg-emerald-200 dark:hover:bg-emerald-900/60 border border-emerald-200/50 dark:border-emerald-800/30'
  }
  if (level === 2) {
    return 'bg-emerald-300 dark:bg-emerald-700/60 hover:bg-emerald-400 dark:hover:bg-emerald-700/80 border border-emerald-400/50 dark:border-emerald-600/30'
  }
  if (level === 3) {
    return 'bg-emerald-500 dark:bg-emerald-600/80 hover:bg-emerald-600 dark:hover:bg-emerald-600 border border-emerald-600/50 dark:border-emerald-500/30'
  }
  return 'bg-emerald-700 dark:bg-emerald-500 hover:bg-emerald-800 dark:hover:bg-emerald-400 border border-emerald-800/50 dark:border-emerald-400/30'
}

const isoDate: (date: Date) => string = (date: Date): string =>
  date.toISOString().split('T')[0] ?? date.toISOString()

const sundayOfWeekUTC: (date: Date) => Date = (date: Date): Date => {
  const utcDate: Date = new Date(
    Date.UTC(date.getUTCFullYear(), date.getUTCMonth(), date.getUTCDate())
  )
  const day: number = utcDate.getUTCDay() // 0..6 (Sun..Sat)
  utcDate.setUTCDate(utcDate.getUTCDate() - day)
  return utcDate
}

const makeDataMap: (
  input: readonly ContributionPoint[]
) => Map<string, ContributionPoint> = (
  input: readonly ContributionPoint[]
): Map<string, ContributionPoint> => {
  const entries: readonly (readonly [string, ContributionPoint])[] = input.map(
    (
      contributionPoint: ContributionPoint
    ): readonly [string, ContributionPoint] => [
      contributionPoint.date,
      contributionPoint,
    ]
  )
  return new Map<string, ContributionPoint>(entries)
}

interface MonthContext {
  readonly currentMonth: string
  readonly locale: string
  readonly weekDays: readonly (ContributionPoint | null)[]
  readonly weeksLength: number
}

interface ComputeMonthLabelResult {
  readonly label: MonthLabel | null
  readonly next: string
}

const computeMonthLabel: (context: MonthContext) => ComputeMonthLabelResult = ({
  currentMonth,
  locale,
  weekDays,
  weeksLength,
}: MonthContext): ComputeMonthLabelResult => {
  const first: ContributionPoint | undefined = weekDays.find(
    (
      contributionPoint: ContributionPoint | null
    ): contributionPoint is ContributionPoint => contributionPoint !== null
  )
  if (first === undefined) {
    return { label: null, next: currentMonth }
  }

  const monthName: string = new Date(first.date).toLocaleDateString(locale, {
    month: 'short',
  })
  if (monthName === currentMonth) {
    return { label: null, next: currentMonth }
  }

  return {
    label: { month: monthName, weekIndex: weeksLength },
    next: monthName,
  }
}

interface BuildCalendarProperties {
  readonly data: readonly ContributionPoint[]
  readonly locale: string
}

const buildCalendar: (properties: BuildCalendarProperties) => CalendarModel = ({
  data,
  locale,
  // eslint-disable-next-line complexity
}: BuildCalendarProperties): CalendarModel => {
  if (data.length === 0) {
    return { monthLabels: [], weeks: [] }
  }

  const dataMap: Map<string, ContributionPoint> = makeDataMap(data)
  const sorted: readonly ContributionPoint[] = [...data].sort(
    (pointOne: ContributionPoint, pointTwo: ContributionPoint): number =>
      new Date(pointOne.date).getTime() - new Date(pointTwo.date).getTime()
  )

  // normalize to UTC midnight to avoid TZ/DST drift
  const firstISO: string =
    sorted[0]?.date ?? panic('buildCalendar: empty contributions array')

  const lastISO: string =
    sorted[sorted.length - 1]?.date ?? panic('buildCalendar: missing last item')

  const firstDate: Date = new Date(`${firstISO}T00:00:00Z`)
  const lastDate: Date = new Date(`${lastISO}T00:00:00Z`)

  const cursor: Date = sundayOfWeekUTC(firstDate)
  const weeks: WeekModel[] = []
  const monthLabels: MonthLabel[] = []
  let currentMonth: string = ''

  while (cursor.getTime() <= lastDate.getTime()) {
    const start: Date = new Date(cursor.getTime())
    const days: (ContributionPoint | null)[] = []

    for (let index: number = 0; index < 7; index++) {
      const key: string = isoDate(cursor) // YYYY-MM-DD (UTC)
      const day: ContributionPoint | null = dataMap.get(key) ?? null
      days.push(day)
      // 🔧 advance in UTC, not local time
      cursor.setUTCDate(cursor.getUTCDate() + 1)
    }

    const response: ComputeMonthLabelResult = computeMonthLabel({
      currentMonth,
      locale,
      weekDays: days,
      weeksLength: weeks.length,
    })
    currentMonth = response.next
    if (response.label !== null) {
      monthLabels.push(response.label)
    }

    weeks.push({ days, key: isoDate(start) })
  }

  return { monthLabels, weeks }
}

interface DayLabelTripleProperties {
  readonly data: readonly ContributionPoint[]
  readonly locale: Locale
}

interface DayLabelTripleResult {
  readonly dayFive: string
  readonly dayOne: string
  readonly dayThree: string
}
const dayLabelTriple: (p: DayLabelTripleProperties) => DayLabelTripleResult = ({
  data,
  locale,
}: DayLabelTripleProperties): DayLabelTripleResult => {
  if (data.length === 0) {
    return { dayFive: '', dayOne: '', dayThree: '' }
  }

  const iso: string =
    data[0]?.date ?? panic('dayLabelTriple: empty contributions array')
  const firstUTC: Date = new Date(`${iso}T00:00:00Z`)
  const start: Date = sundayOfWeekUTC(firstUTC)

  const labelUTC: (offset: number) => string = (offset: number): string => {
    const date: Date = new Date(start)
    date.setUTCDate(date.getUTCDate() + offset)
    return date.toLocaleDateString(locale, {
      timeZone: 'UTC',
      weekday: 'short',
    })
  }

  return { dayFive: labelUTC(5), dayOne: labelUTC(1), dayThree: labelUTC(3) }
}

const sumCount: (input: readonly ContributionPoint[]) => number = (
  input: readonly ContributionPoint[]
): number => {
  let total: number = 0
  for (const contributionPoint of input) {
    total += contributionPoint.count
  }
  return total
}

/* =============================== Subviews =============================== */

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

const LegendBar: FCStrict<LegendBarProperties> = ({
  less,
  more,
}: LegendBarProperties): JSX.Element => {
  return (
    <div className="text-muted-foreground flex items-center gap-2 text-xs">
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

const MonthLabelsRow: FCStrict<MonthLabelsRowProperties> = ({
  labels,
}: MonthLabelsRowProperties): JSX.Element => {
  return (
    <div className="relative mb-3 h-5">
      {labels.map(
        (monthLabel: MonthLabel): JSX.Element => (
          <div
            className="text-muted-foreground absolute text-sm font-medium"
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

const WeekdayLabelCol: FCStrict<WeekdayLabelColProperties> = ({
  dayFive,
  dayOne,
  dayThree,
}: WeekdayLabelColProperties): JSX.Element => {
  return (
    <div className="flex flex-shrink-0 flex-col gap-1 pr-3">
      {WEEKDAY_INDICES.map(
        (index: (typeof WEEKDAY_INDICES)[number]): JSX.Element => {
          const label: string =
            index === 1
              ? dayOne
              : index === 3
                ? dayThree
                : index === 5
                  ? dayFive
                  : ''
          return (
            <div
              className="text-muted-foreground flex h-4 w-8 items-center text-xs font-medium"
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
  readonly onEnter: (properties: CellEnterProperties) => void
  readonly onLeave: () => void
  readonly weekIndex: number
}

const DayCell: FCStrict<DayCellProperties> = ({
  day,
  dayIndex,
  onEnter,
  onLeave,
  weekIndex,
}: DayCellProperties): JSX.Element => {
  const delayMs: string = String(weekIndex * 7 + dayIndex) + 'ms'
  const aria: string = String(day.count) + ' on ' + day.date
  return (
    <div
      aria-label={aria}
      className={`h-4 w-4 cursor-pointer rounded-sm transition-all duration-200 ${levelClass(
        day.level
      )}`}
      key={day.date}
      role="button"
      style={{ animationDelay: delayMs }}
      tabIndex={0}
      onBlur={onLeave}
      onFocus={(error: React.FocusEvent<HTMLDivElement>): void => {
        onEnter({
          element: error as unknown as React.MouseEvent<HTMLDivElement>,
          point: day,
        })
      }}
      onKeyDown={(error: React.KeyboardEvent<HTMLDivElement>): void => {
        if (error.key === 'Enter' || error.key === ' ') {
          onEnter({
            element: error as unknown as React.MouseEvent<HTMLDivElement>,
            point: day,
          })
        }
      }}
      onMouseEnter={(error: React.MouseEvent<HTMLDivElement>): void => {
        onEnter({ element: error, point: day })
      }}
      onMouseLeave={onLeave}
    />
  )
}

const EmptyCell: FCStrict = (): JSX.Element => (
  <div className="border-border/30 bg-muted/20 dark:border-border/20 dark:bg-muted/10 h-4 w-4 rounded-sm border" />
)

interface CellEnterProperties {
  readonly element: React.MouseEvent<HTMLDivElement>
  readonly point: ContributionPoint
}

interface WeekColumnProperties {
  readonly onEnter: (properties: CellEnterProperties) => void
  readonly onLeave: () => void
  readonly week: WeekModel
  readonly weekIndex: number
}

const WeekColumn: FCStrict<WeekColumnProperties> = ({
  onEnter,
  onLeave,
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
              weekIndex={weekIndex}
              onEnter={onEnter}
              onLeave={onLeave}
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
  readonly onEnter: (properties: CellEnterProperties) => void
  readonly onLeave: () => void
  readonly weeks: readonly WeekModel[]
}

const WeeksGrid: FCStrict<WeeksGridProperties> = ({
  dayFive,
  dayOne,
  dayThree,
  onEnter,
  onLeave,
  weeks,
}: WeeksGridProperties): JSX.Element => {
  return (
    <div className="flex gap-1">
      <WeekdayLabelCol dayFive={dayFive} dayOne={dayOne} dayThree={dayThree} />
      <div className="flex gap-1">
        {weeks.map(
          (weekModel: WeekModel, index: number): JSX.Element => (
            <WeekColumn
              key={weekModel.key}
              week={weekModel}
              weekIndex={index}
              onEnter={onEnter}
              onLeave={onLeave}
            />
          )
        )}
      </div>
    </div>
  )
}

interface TooltipProperties {
  readonly day: ContributionPoint
  readonly locale: string
  readonly mouse: MousePos
}

const Tooltip: FCStrict<TooltipProperties> = ({
  day,
  locale,
  mouse,
}: TooltipProperties): JSX.Element => {
  const cnt: string = day.count.toLocaleString(locale)
  const dateTxt: string = new Date(day.date).toLocaleDateString(locale, {
    day: 'numeric',
    month: 'short',
    timeZone: 'UTC',
    weekday: 'short',
    year: 'numeric',
  })
  return (
    <div
      className="bg-popover dark:border-border/50 dark:bg-popover/95 pointer-events-none fixed z-50 -translate-x-1/2 -translate-y-full rounded-lg border-2 px-3 py-2 text-sm shadow-xl dark:shadow-2xl"
      style={{ left: mouse.x, top: mouse.y - 8 }}
    >
      <p className="font-semibold">{cnt}</p>
      <p className="text-muted-foreground text-xs">{dateTxt}</p>
    </div>
  )
}

/* ================================ Main FC ================================ */

// eslint-disable-next-line max-lines-per-function
export const ContributionGraph: FCStrict<ContributionGraphProperties> = ({
  data,
  locale,
}: ContributionGraphProperties): JSX.Element => {
  const translations: Translations<'projects.contributions'> = useTranslations(
    'projects.contributions'
  )
  const [hoveredDay, setHoveredDay]: [
    ContributionPoint | null,
    Dispatch<SetStateAction<ContributionPoint | null>>,
  ] = useState<ContributionPoint | null>(null)
  const [mouse, setMouse]: [MousePos, Dispatch<SetStateAction<MousePos>>] =
    useState<MousePos>({ x: 0, y: 0 })

  const calendar: CalendarModel = useMemo<CalendarModel>(
    (): CalendarModel => buildCalendar({ data, locale }),
    [data, locale]
  )
  const labels: DayLabelTripleResult = useMemo<DayLabelTripleResult>(
    (): DayLabelTripleResult => dayLabelTriple({ data, locale }),
    [data, locale]
  )

  const total: number = useMemo<number>((): number => sumCount(data), [data])

  const lessText: string = translations('less')
  const moreText: string = translations('more')
  const titleText: string = translations('title')
  const totalText: string = translations('totalAmount', {
    count: total.toLocaleString(locale),
  })

  const handleEnter: (properties: CellEnterProperties) => void = ({
    element,
    point,
  }: CellEnterProperties): void => {
    const rect: DOMRect = element.currentTarget.getBoundingClientRect()
    const mouseX: number = rect.left + rect.width / 2
    const mouseY: number = rect.top
    setMouse({ x: mouseX, y: mouseY })
    setHoveredDay(point)
  }

  const handleLeave: () => void = (): void => {
    setHoveredDay(null)
  }

  return (
    <Card className="hover:border-primary/50 dark:bg-card/50 w-full overflow-hidden border-2 p-6 transition-all duration-300 hover:shadow-lg">
      <div className="mb-6 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h3 className="text-2xl font-bold">{titleText}</h3>
          <p className="text-muted-foreground text-sm">{totalText}</p>
        </div>
        <LegendBar less={lessText} more={moreText} />
      </div>

      <div className="relative w-full overflow-x-auto pb-2">
        <div className="inline-block min-w-full">
          <MonthLabelsRow labels={calendar.monthLabels} />
          <WeeksGrid
            dayFive={labels.dayFive}
            dayOne={labels.dayOne}
            dayThree={labels.dayThree}
            weeks={calendar.weeks}
            onEnter={handleEnter}
            onLeave={handleLeave}
          />
        </div>
      </div>

      {hoveredDay !== null ? (
        <Tooltip day={hoveredDay} locale={locale} mouse={mouse} />
      ) : null}
    </Card>
  )
}
